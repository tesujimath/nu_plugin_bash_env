use std::env;

use nu_plugin::{LabeledError, Plugin, PluginResponse};
use nu_protocol::*;

struct BashEnv;

impl BashEnv {
    fn new() -> Self {
        Self
    }
}

impl Plugin for BashEnv {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![PluginSignature::build("bash-env")
            .usage("get environment variables from a Bash environment file")
            .category(Category::Env)
            .required("path", SyntaxShape::String, "path to environment file")
            .input_output_types(vec![(Type::String, Type::Any)])
            .allow_variants_without_examples(true)]
    }

    fn run(
        &mut self,
        _name: &str,
        _call: &nu_plugin::EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, nu_plugin::LabeledError> {
        todo!()
    }
}

fn print_signature() {
    let plugin = BashEnv::new();
    let signature = PluginResponse::Signature(plugin.signature());
    let json_signature = serde_json::to_string_pretty(&signature).unwrap();
    println!("{}", json_signature);
}

fn print_response() {
    let cols_vals = [("A", "a"), ("B", "b"), ("C", "c")];
    let cols = cols_vals
        .iter()
        .map(|s| s.0.to_string())
        .collect::<Vec<_>>();
    let vals = cols_vals
        .iter()
        .map(|s| Value::string(s.1.to_string(), Span::unknown()))
        .collect::<Vec<_>>();
    let record = Value::record(Record::from_raw_cols_vals(cols, vals), Span::unknown());
    let response = PluginResponse::Value(Box::new(record));
    let json_response = serde_json::to_string_pretty(&response).unwrap();
    println!("{}", json_response);
}

fn print_error() {
    let response = PluginResponse::Error(LabeledError {
        label: "bash-env".to_string(),
        msg: "oops".to_string(),
        span: Some(Span::unknown()),
    });
    let json_response = serde_json::to_string_pretty(&response).unwrap();
    println!("{}", json_response);
}

pub fn main() {
    let mut args = env::args();
    let usage = format!("usage: {} sig|response|error", args.next().unwrap());
    match args.next() {
        Some(cmd) => {
            if cmd == "sig" {
                print_signature()
            } else if cmd == "response" {
                print_response()
            } else if cmd == "error" {
                print_error()
            } else {
                println!("{}", usage)
            }
        }
        None => println!("{}", usage),
    }
}
