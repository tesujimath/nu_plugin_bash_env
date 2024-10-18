#![doc = include_str!("../README.md")]
use anyhow::{anyhow, Context};
use nu_plugin::{
    serve_plugin, EngineInterface, EvaluatedCall, JsonSerializer, Plugin, PluginCommand,
};
use nu_protocol::{
    Category, IntoPipelineData, LabeledError, PipelineData, Record, Signature, Span, SyntaxShape,
    Type, Value,
};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use shellexpand::tilde;
use std::{
    collections::{HashMap, HashSet},
    concat, env, fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use subprocess::{Popen, PopenConfig};
use tempfile::TempDir;
use tracing::{debug, info, trace};
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

    fn description(&self) -> &str {
        "get environment variables from Bash format file and/or stdin"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .description("get environment variables from Bash format file and/or stdin")
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
    ) -> std::result::Result<nu_protocol::PipelineData, LabeledError> {
        let cwd = engine.get_current_dir()?;

        let span = input.span();
        let path = match call.positional.first() {
            Some(value @ Value::String { val: path, .. }) => {
                let path = PathBuf::from(tilde(path).into_owned());
                let abs_path = Path::new(&cwd).join(&path);
                if abs_path.exists() {
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
                value.span(),
            ))?,
        };

        let export = call
            .named
            .iter()
            .filter(|&(name, _value)| (name.item == "export"))
            .map(|(_name, value)| {
                if let Some(Value::List { vals, .. }) = value {
                    vals.iter()
                        .filter_map(|value| {
                            if let Value::String { val, .. } = value {
                                Some(val.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<String>>()
                } else {
                    Vec::default()
                }
            })
            .next()
            .unwrap_or_default();

        trace!("PipelineData {:?}", &input);
        let stdin = match input {
            // TODO: pipe the stream into the subprocess rather than via a string
            PipelineData::ByteStream(bytes, _metadata) => Some(bytes.into_string()?),
            PipelineData::Value(Value::String { val: stdin, .. }, _metadata) => Some(stdin),
            _ => None,
        };

        trace!(
            "path={:?} stdin={:?} export={:?} cwd={:?}",
            &path,
            &stdin,
            &export,
            &cwd
        );

        bash_env(
            span.unwrap_or(Span::unknown()),
            call.head,
            stdin,
            path,
            export,
            cwd,
        )
        .map(|value| value.into_pipeline_data())
        .map_err(|e| {
            LabeledError::new(e.to_string()).with_label("bash-env", span.unwrap_or(Span::unknown()))
        })
    }
}

fn bash_env(
    input_span: Span,
    creation_site_span: Span,
    stdin: Option<String>,
    path: Option<String>,
    export: Vec<String>,
    cwd: String,
) -> anyhow::Result<Value> {
    let script_path = bash_env_json_path();
    let mut argv: Vec<_> = [script_path].into();

    // we no longer support both stdin and path at the same time
    if stdin.is_some() && path.is_some() {
        return Err(anyhow!(
            "both stdin and path at the same time is no longer supported"
        ));
    }

    if let Some(ref path) = path {
        argv.push(path.as_str());
    }

    trace!("Popen::create({:?})", &argv);

    let mut p = Popen::create(
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
    .with_context(|| format!("Popen::create({})", script_path))?;

    let (out, err) = p
        .communicate(stdin.as_deref())
        .with_context(|| "Popen::communicate()")?;
    if let Some(err) = err {
        std::io::stderr()
            .write_all(err.as_bytes())
            .with_context(|| "stderr.write_all()")?;
    }

    let BashEnvResult {
        env,
        shellvars,
        error,
    } = serde_json::from_str(out.as_ref().unwrap()).with_context(|| "serde_json::from_reader()")?;

    if let Some(msg) = error {
        Err(anyhow!(msg.clone()))
    } else if let (Some(env), Some(shellvars)) = (env, shellvars) {
        Ok(create_record(
            env,
            shellvars,
            export,
            input_span,
            creation_site_span,
        ))
    } else {
        Err(anyhow!("unexpected result from bash-env-json"))
    }
}

fn create_record(
    env: HashMap<String, String>,
    shellvars: HashMap<String, String>,
    export: Vec<String>,
    input_span: Span,
    creation_site_span: Span,
) -> Value {
    let export: HashSet<_> = export.iter().collect();
    let exported_shellvars = shellvars
        .into_iter()
        .filter(|(k, _v)| export.contains(k))
        // .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<HashMap<_, _>>();
    let cols = env
        .iter()
        .chain(exported_shellvars.iter())
        .map(|(k, _v)| k.clone())
        .collect::<Vec<_>>();
    let vals = env
        .iter()
        .chain(exported_shellvars.iter())
        .map(|(_k, v)| Value::string(v.clone(), Span::unknown()))
        .collect::<Vec<_>>();
    Value::record(
        Record::from_raw_cols_vals(cols, vals, input_span, creation_site_span).unwrap(),
        input_span,
    )
}

#[derive(Serialize, Deserialize)]
struct BashEnvResult {
    env: Option<HashMap<String, String>>,
    shellvars: Option<HashMap<String, String>>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct KV {
    k: String,
    v: String,
}

fn create_error<S>(msg: S, creation_site_span: Span) -> LabeledError
where
    S: Into<String>,
{
    LabeledError::new(msg).with_label("bash-env", creation_site_span)
}

fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("NU_PLUGIN_BASH_ENV_LOG"))
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to setup tracing subscriber");

    debug!("starting");

    // prefer to take the path from the environment variable, falling back to writing a temporary file
    // with contents taken from the embedded script
    let script_path_env_var = "NU_PLUGIN_BASH_ENV_JSON";
    let script_path_from_env = env::var(script_path_env_var).ok();
    #[allow(unused_assignments)]
    let mut tempdir: Option<TempDir> = None;

    let script_path = match script_path_from_env {
        Some(path) => {
            debug!("using {} from {}", &path, script_path_env_var);
            path
        }
        None => {
            tempdir = Some(TempDir::new().expect("failed to create tempdir for bash script"));
            extract_embedded_script(tempdir.as_ref().unwrap())
        }
    };

    BASH_ENV_JSON_PATH.get_or_init(|| script_path);

    serve_plugin(&BashEnvPlugin, JsonSerializer);

    if let Some(tempdir) = tempdir {
        info!("removing {:?}", tempdir.path());
    }

    debug!("exiting");
}

fn extract_embedded_script(tempdir: &TempDir) -> String {
    const SCRIPT: &str = "bash-env-json";
    let out_path = tempdir.path().join(SCRIPT).to_path_buf();
    let script_body = include_str!(concat!(env!("OUT_DIR"), "/bash-env-json/bash-env-json"));
    fs::write(&out_path, script_body).unwrap();

    // make executable
    let mut perms = fs::metadata(&out_path)
        .unwrap_or_else(|e| panic!("metadata({:?}): {}", &out_path, e))
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&out_path, perms)
        .unwrap_or_else(|e| panic!("set_permissions({:?}): {}", &out_path, e));

    let path = out_path.into_os_string().into_string().unwrap();
    info!("extracted {} into {}", SCRIPT, &path);
    path
}

fn bash_env_json_path() -> &'static str {
    BASH_ENV_JSON_PATH.get().unwrap()
}

static BASH_ENV_JSON_PATH: OnceCell<String> = OnceCell::new();
