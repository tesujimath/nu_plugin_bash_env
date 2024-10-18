use std::{
    env,
    path::{Path, PathBuf},
};

const BASH_ENV_JSON_VERSION: &str = "0.6.0";

fn fetch_bash_env_json() -> Option<PathBuf> {
    let out_dir: PathBuf = env::var("OUT_DIR").unwrap().into();
    let bash_env_json_repo_dir = PathBuf::from("bash-env-json");
    let bash_env_json_repo_path = out_dir.join(bash_env_json_repo_dir.as_path());

    if Path::exists(&bash_env_json_repo_path) {
        Some(bash_env_json_repo_path)
    } else {
        let bash_env_json_repo_path_str = bash_env_json_repo_path.to_string_lossy();
        let git_args = [
            "clone",
            "--filter=blob:none",
            "--branch",
            BASH_ENV_JSON_VERSION,
            "https://github.com/tesujimath/bash-env-json.git",
            bash_env_json_repo_path_str.as_ref(),
        ];
        println!("cargo:warning=git {}", &git_args.join(" "));
        match std::process::Command::new("git").args(git_args).output() {
            Ok(output) => {
                if output.status.success() {
                    Some(bash_env_json_repo_path)
                } else {
                    println!(
                        "cargo:warning=git {:?} failed: {}",
                        &git_args, output.status
                    );

                    None
                }
            }
            Err(e) => {
                println!("cargo:warning=git clone failed: {}", e);

                None
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    if let Some(_bash_env_json_repo_path) = fetch_bash_env_json() {
        // TODO what now?
    }

    Ok(())
}
