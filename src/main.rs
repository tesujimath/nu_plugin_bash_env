use std::path::PathBuf;

use nu_plugin::{serve_plugin, EvaluatedCall, JsonSerializer};
use nu_plugin::{EngineInterface, Plugin, PluginCommand};
use nu_protocol::{
    Category, IntoPipelineData, LabeledError, PipelineData, Record, Signature, Span, SyntaxShape,
    Type, Value,
};
use shellexpand::tilde;
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
        let _cwd = engine.get_current_dir();

        let span = input.span();
        let path = match call.positional.first() {
            Some(value @ Value::String { val: path, .. }) => {
                let path = PathBuf::from(tilde(path).into_owned());
                if path.exists() {
                    Some(path)
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
            PipelineData::ByteStream(s, _metadata) => Some(s),
            _ => None,
        };

        debug!("run path={:?} stdin={}", &path, stdin.is_some());

        Ok(create_environment(span.unwrap_or(Span::unknown()), call.head).into_pipeline_data())
    }
}

fn create_environment(input_span: Span, creation_site_span: Span) -> Value {
    let cols_vals = [("A", "a"), ("B", "b"), ("C", "c1")];
    let cols = cols_vals
        .iter()
        .map(|s| s.0.to_string())
        .collect::<Vec<_>>();
    let vals = cols_vals
        .iter()
        .map(|s| Value::string(s.1.to_string(), Span::unknown()))
        .collect::<Vec<_>>();
    Value::record(
        Record::from_raw_cols_vals(cols, vals, input_span, creation_site_span).unwrap(),
        input_span,
    )
}

fn create_error<S>(msg: S, creation_site_span: Span) -> LabeledError
where
    S: Into<String>,
{
    LabeledError::new(msg).with_label("bash-env", creation_site_span)
}

fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to setup tracing subscriber");

    serve_plugin(&BashEnvPlugin, JsonSerializer)
}
