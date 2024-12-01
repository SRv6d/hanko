use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let cargo = vergen_gix::CargoBuilder::all_cargo()?;
    let rustc = vergen_gix::RustcBuilder::all_rustc()?;
    let git = vergen_gix::GixBuilder::all_git()?;
    vergen_gix::Emitter::default()
        .add_instructions(&cargo)?
        .add_instructions(&rustc)?
        .add_instructions(&git)?
        .emit()?;

    println!(
        "cargo:rustc-env=LONG_VERSION_PROFILE={}",
        env::var("PROFILE").unwrap()
    );
    println!("cargo:rustc-env=LONG_VERSION_ENV={}", env());
    println!(
        "cargo:rustc-env=LONG_VERSION_FEATURES={}",
        enabled_features()
    );

    Ok(())
}

/// Build environment information.
fn env() -> String {
    if env::var("GITHUB_ACTION").is_ok() {
        // The following variables are expected to exist when building within a GitHub workflow.
        // https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/store-information-in-variables#default-environment-variables
        return format!("GitHub CI {}", env::var("GITHUB_WORKFLOW_REF").unwrap());
    }

    "unknown environment".to_string()
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
