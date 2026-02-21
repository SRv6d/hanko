use std::{env, process::Command};

fn main() {
    if let Ok(sha) = env::var("GIT_SHA") {
        println!("cargo:rustc-env=GIT_SHA={sha}");
    }
    println!("cargo:rustc-env=RUSTC_SEMVER={}", rustc_semver());
    println!("cargo:rustc-env=PROFILE={}", env::var("PROFILE").unwrap());
    if let Some(build_env) = build_env() {
        println!("cargo:rustc-env=BUILD_ENV={build_env}");
    }
    println!("cargo:rustc-env=ENABLED_FEATURES={}", enabled_features());
}

fn rustc_semver() -> String {
    env::var("RUSTC")
        .ok()
        .and_then(|rustc| {
            Command::new(rustc)
                .arg("--version")
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
        })
        .and_then(|version| version.split_whitespace().nth(1).map(String::from))
        .unwrap_or_default()
}

fn build_env() -> Option<String> {
    if let Ok(workflow_ref) = env::var("GITHUB_WORKFLOW_REF") {
        return Some(format!("GitHub CI {workflow_ref}"));
    }
    if env::var("NIX_BUILD_TOP").is_ok() {
        return Some("Nix".to_string());
    }
    None
}

fn enabled_features() -> String {
    let prefix = "CARGO_FEATURE_";
    env::vars()
        .filter(|(key, _)| key.starts_with(prefix))
        .map(|(key, _)| {
            format!(
                "+{}",
                key.strip_prefix(prefix)
                    .unwrap()
                    .to_lowercase()
                    .replace('_', "-")
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}
