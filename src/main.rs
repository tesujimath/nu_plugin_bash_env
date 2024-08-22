use nu_plugin::{
    serve_plugin, EngineInterface, EvaluatedCall, JsonSerializer, Plugin, PluginCommand,
};
use nu_protocol::{
    Category, IntoPipelineData, LabeledError, PipelineData, Record, Signature, Span, SyntaxShape,
    Type, Value,
};
use once_cell::sync::OnceCell;
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use shellexpand::tilde;
use std::{env, fs, io::Write, os::unix::fs::PermissionsExt, path::PathBuf};
use subprocess::{Popen, PopenConfig};
use tempfile::TempDir;
use tracing::debug;
use tracing_subscriber::EnvFilter;

struct BashEnvPlugin;

impl Plugin for BashEnvPlugin {
    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(BashEnv)]
    }

    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

struct BashEnv;

impl PluginCommand for BashEnv {
    type Plugin = BashEnvPlugin;

    fn name(&self) -> &str {
        "bash-env"
    }

    fn usage(&self) -> &str {
        "get environment variables from Bash format file and/or stdin"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .usage("get environment variables from Bash format file and/or stdin")
            .category(Category::Env)
            .optional("path", SyntaxShape::String, "path to environment file")
            .named(
                "export",
                SyntaxShape::List(Box::new(SyntaxShape::String)),
                "shell variables to export",
                None,
            )
            .input_output_types(vec![(Type::Nothing, Type::Any), (Type::String, Type::Any)])
            .filter()
            .allow_variants_without_examples(true)
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: nu_protocol::PipelineData,
    ) -> Result<nu_protocol::PipelineData, LabeledError> {
        let cwd = engine.get_current_dir()?;

        let span = input.span();
        let path = match call.positional.first() {
            Some(value @ Value::String { val: path, .. }) => {
                let path = PathBuf::from(tilde(path).into_owned());
                if path.exists() {
                    Some(path.into_os_string().into_string().unwrap())
                } else {
                    Err(create_error(
                        format!("no such file {:?}", path),
                        value.span(),
                    ))?
                }
            }
            None => None,
            Some(value) => Err(create_error(
                format!("positional requires string; got {}", value.get_type()),
                call.head,
            ))?,
        };
        let stdin = match input {
            // TODO: pipe the stream into the subprocess rather than via a string
            PipelineData::ByteStream(bytes, _metadata) => Some(bytes.into_string()?),
            PipelineData::Value(Value::String { val: stdin, .. }, _metadata) => Some(stdin),
            _ => {
                debug!("PipelineData {:?}", input);
                None
            }
        };

        debug!("run path={:?} stdin={:?}", &path, stdin);

        bash_env(span.unwrap_or(Span::unknown()), stdin, path, cwd)
            .map(|value| value.into_pipeline_data())
    }
}

fn bash_env(
    input_span: Span,
    stdin: Option<String>,
    path: Option<String>,
    cwd: String,
) -> Result<Value, LabeledError> {
    let script_path = bash_env_script_path();
    let mut argv: Vec<_> = [script_path].into();
    if stdin.is_some() {
        argv.push("--stdin");
    }
    if let Some(ref path) = path {
        argv.push(path.as_str());
    }

    debug!("popen({:?})", &argv);

    let p = Popen::create(
        argv.as_slice(),
        PopenConfig {
            stdin: if stdin.is_some() {
                subprocess::Redirection::Pipe
            } else {
                subprocess::Redirection::None
            },
            stdout: subprocess::Redirection::Pipe,
            cwd: Some(cwd.into()),
            ..Default::default()
        },
    )
    .map_err(|e| create_error(format!("popen({}): {}", script_path, e), input_span))?;

    if let Some(stdin) = stdin {
        p.stdin
            .as_ref()
            .unwrap()
            .write_all(stdin.as_bytes())
            .map_err(to_labeled)?;
    }

    match serde_json::from_reader(p.stdout.as_ref().unwrap()).map_err(to_labeled)? {
        BashEnvResult::Record(value) => Ok(Value::record(value, input_span)),
        BashEnvResult::Error(msg) => Err(create_error(msg, Span::unknown())),
    }
}

#[derive(Serialize, Deserialize)]
enum BashEnvResult {
    Record(Record),
    Error(String),
}

fn create_error<S>(msg: S, creation_site_span: Span) -> LabeledError
where
    S: Into<String>,
{
    LabeledError::new(msg).with_label("bash-env", creation_site_span)
}

fn to_labeled<E>(e: E) -> LabeledError
where
    E: std::error::Error,
{
    create_error(e.to_string(), Span::unknown())
}

fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to setup tracing subscriber");

    serve_plugin(&BashEnvPlugin, JsonSerializer)
}

fn bash_env_script_path() -> &'static str {
    // prefer to take the path from the environment variable, falling back to writing a temporary file
    // with contents taken from the embedded script
    BASH_ENV_SCRIPT_PATH.get_or_init(|| {
        if let Ok(path) = env::var("NU_PLUGIN_BASH_ENV_SCRIPT") {
            path
        } else {
            let tempdir = TempDir::new().unwrap();
            let path = tempdir.path().join("bash_env.sh").to_path_buf();
            fs::write(&path, Scripts::get("bash_env.sh").unwrap().data.as_ref()).unwrap();

            // make executable
            let mut perms = fs::metadata(&path)
                .unwrap_or_else(|e| panic!("metadata({:?}): {}", &path, e))
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms)
                .unwrap_or_else(|e| panic!("set_permissions({:?}): {}", &path, e));

            BASH_ENV_SCRIPT_TEMPDIR.set(tempdir).unwrap();
            path.into_os_string().into_string().unwrap()
        }
    })
}

static BASH_ENV_SCRIPT_PATH: OnceCell<String> = OnceCell::new();
static BASH_ENV_SCRIPT_TEMPDIR: OnceCell<TempDir> = OnceCell::new();

// embed the bash script
#[derive(Embed)]
#[folder = "scripts"]
struct Scripts;
