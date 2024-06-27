use std::path::PathBuf;

use nu_plugin::{serve_plugin, EvaluatedCall, JsonSerializer};
use nu_plugin::{EngineInterface, Plugin, PluginCommand, SimplePluginCommand};
use nu_protocol::{Category, LabeledError, Record, Signature, Span, SyntaxShape, Type, Value};

struct BashEnvApiPlugin;

impl Plugin for BashEnvApiPlugin {
    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(BashEnvApi)]
    }

    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

struct BashEnvApi;

impl SimplePluginCommand for BashEnvApi {
    type Plugin = BashEnvApiPlugin;

    fn name(&self) -> &str {
        "bash-env-api"
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
                "list of shell variables to export",
                None,
            )
            .input_output_types(vec![(Type::Nothing, Type::Any), (Type::String, Type::Any)])
            .filter()
            .allow_variants_without_examples(true)
    }

    fn run(
        &self,
        _plugin: &BashEnvApiPlugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let _cwd = engine.get_current_dir();

        let span = input.span();
        match call.positional.first() {
            Some(value @ Value::String { val: path, .. }) => {
                if PathBuf::from(path).exists() {
                    Ok(create_dummy_environment(span, call.head))
                } else {
                    Err(create_error(format!("no such file {}", path), value.span()))
                }
            }
            None => Ok(create_dummy_environment(span, call.head)),
            Some(value) => Err(create_error(
                format!("positional requires string; got {}", value.get_type()),
                call.head,
            )),
        }
    }
}

fn create_dummy_environment(input_span: Span, creation_site_span: Span) -> Value {
    let cols_vals = [("A", "a"), ("B", "b"), ("C", "c")];
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
    LabeledError::new(msg).with_label("bash-env-api", creation_site_span)
}

fn main() {
    serve_plugin(&BashEnvApiPlugin, JsonSerializer)
}
