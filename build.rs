use anyhow::anyhow;
use std::{
    env,
    fs::{copy, create_dir},
    path::{Path, PathBuf},
};

// If fetching bash-env-json directly from GitHub, this defines what we fetch.
// Note that when building the package from the Nix Flake, this is not used.
// In that case, the package version is as defined in the Flake input.
const BASH_ENV_JSON_GITHUB_TAG: &str = "0.6.1";

// install bash-env-json locally if available
fn install_nix_bash_env_json<P>(out_dir: P) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    match env::var("NIX_BASH_ENV_JSON") {
        Err(_) => {
            println!("cargo:warning=install_nix_bash_env_json failed to find NIX_BASH_ENV_JSON, falling back to fetch from GitHub");
            None
        }
        Ok(bash_env_json) => {
            let src_path: PathBuf = bash_env_json.into();

            let bash_env_json_repo_dir = PathBuf::from("bash-env-json");
            let bash_env_json_repo_path = out_dir.as_ref().join(bash_env_json_repo_dir.as_path());

            match create_dir(&bash_env_json_repo_path) {
                Err(e) => {
                    println!(
                        "cargo:warning=install_nix_bash_env_json failed to create directory {}: {}",
                        bash_env_json_repo_path.to_string_lossy(),
                        e
                    );
                    None
                }
                Ok(_) => {
                    let dst_path = bash_env_json_repo_path.join("bash-env-json");
                    match copy(&src_path, &dst_path) {
                        Err(e) => {
                            println!(
                                "cargo:warning=install_nix_bash_env_json failed to copy {} to {}: {}",
                                src_path.to_string_lossy(),
                                &dst_path.to_string_lossy(),
                                e
                            );
                            None
                        }
                        Ok(_) => {
                            println!(
                                "cargo:warning=install_nix_bash_env_json installed {} as {}",
                                src_path.to_string_lossy(),
                                &dst_path.to_string_lossy(),
                            );
                            Some(dst_path)
                        }
                    }
                }
            }
        }
    }
}

// fetch bash-env-json from GitHub
fn fetch_bash_env_json<P>(out_dir: P) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    let bash_env_json_repo_dir = PathBuf::from("bash-env-json");
    let bash_env_json_repo_path = out_dir.as_ref().join(bash_env_json_repo_dir.as_path());

    if Path::exists(&bash_env_json_repo_path) {
        Some(bash_env_json_repo_path)
    } else {
        let bash_env_json_repo_path_str = bash_env_json_repo_path.to_string_lossy();
        let git_args = [
            "clone",
            "--filter=blob:none",
            "--branch",
            BASH_ENV_JSON_GITHUB_TAG,
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

fn main() -> anyhow::Result<()> {
    let out_dir: PathBuf = env::var("OUT_DIR").unwrap().into();

    match install_nix_bash_env_json(&out_dir) {
        Some(_path) => Ok(()),
        None => match fetch_bash_env_json(&out_dir) {
            Some(_path) => Ok(()),
            None => Err(anyhow!("failed to fetch bash-env-json from GitHub")),
        },
    }
}
