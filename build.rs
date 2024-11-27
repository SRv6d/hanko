use std::{env, fmt::Write};

fn main() {
    println!("cargo:rustc-env=LONG_VERSION={}", version());
    println!("cargo:rustc-env=LONG_VERSION_BUILD={}", build_env());
    println!(
        "cargo:rustc-env=LONG_VERSION_FEATURES={}",
        enabled_features()
    );
}

fn version() -> String {
    format!(
        "{} ({})",
        env!("CARGO_PKG_VERSION"),
        env::var("GIT_COMMIT").unwrap_or("unknown commit".to_string())
    )
}

fn build_env() -> String {
    let mut build_env = format!(
        "rustc {}, {} profile",
        env!("RUST_VERSION"),
        env::var("PROFILE").unwrap(),
    );
    if env::var("GITHUB_ACTION").is_ok() {
        // The following variables are expected to exist when building within a GitHub workflow.
        // https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/store-information-in-variables#default-environment-variables
        write!(
            build_env,
            ", GitHub CI {}",
            env::var("GITHUB_WORKFLOW_REF").unwrap()
        )
        .unwrap();
    }
    build_env
}

fn enabled_features() -> String {
    let prefix = "CARGO_FEATURE_";
    let features = env::vars()
        .filter(|(key, _)| key.starts_with(prefix))
        .map(|(key, _)| {
            key.strip_prefix(prefix)
                .unwrap()
                .to_lowercase()
                .replace('_', "-")
        });
    features
        .map(|name| format!("+{name}"))
        .collect::<Vec<_>>()
        .join(" ")
}
