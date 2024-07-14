use super::update::update;
use crate::Configuration;
use anyhow::{Context, Result};
use clap::{
    builder::{OsStr, Resettable},
    Parser, Subcommand,
};
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf, time::Instant};
use tracing::{debug, info, Level};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The configuration file.
    #[arg(
        short,
        long,
        value_name = "PATH",
        env = "HANKO_CONFIG",
        default_value = default_config_path()
    )]
    pub config: PathBuf,

    // The runtime configuration.
    #[command(flatten)]
    runtime_config: RuntimeConfiguration,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Update the allowed signers file.
    Update,
    // /// Manage signers.
    // #[command(subcommand)]
    // Signer(ManageSigners),

    // /// Manage sources.
    // #[command(subcommand)]
    // Source(ManageSources),
}

/// Runtime configuration that overrides the configuration file.
#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct RuntimeConfiguration {
    /// The allowed signers file.
    #[arg(
        long,
        value_name = "PATH",
        env = "HANKO_ALLOWED_SIGNERS",
        default_value = git_allowed_signers()
    )]
    pub allowed_signers: Option<PathBuf>,

    /// Increase verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

/// The default configuration file path according to the XDG Base Directory Specification.
/// If neither `$XDG_CONFIG_HOME` nor `$HOME` are set, `Resettable::Reset` is returned, forcing the user to specify the path.
fn default_config_path() -> Resettable<OsStr> {
    let dirname = env!("CARGO_PKG_NAME");
    let filename = "config.toml";

    if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
        Resettable::Value(format!("{xdg_config_home}/{dirname}/{filename}").into())
    } else if let Ok(home) = env::var("HOME") {
        Resettable::Value(format!("{home}/.config/{dirname}/{filename}").into())
    } else {
        Resettable::Reset
    }
}

/// The path to the allowed signers file as configured within Git.
fn git_allowed_signers() -> Resettable<OsStr> {
    if let Ok(file) = gix_config::File::from_globals() {
        if let Some(path) = file.path("gpg", Some("ssh".into()), "allowedsignersfile") {
            if let Ok(interpolated) = path.interpolate(gix_config::path::interpolate::Context {
                home_dir: env::var("HOME")
                    .ok()
                    .map(std::convert::Into::<PathBuf>::into)
                    .as_deref(),
                ..Default::default()
            }) {
                return Resettable::Value(OsStr::from(interpolated.to_string_lossy().to_string()));
            }
        }
    }

    Resettable::Reset
}

/// The main CLI entrypoint.
pub fn entrypoint() -> Result<()> {
    let start = Instant::now();
    let args = Args::parse();

    setup_tracing(args.runtime_config.verbose);

    let config = Configuration::load(&args.config, Some(args.runtime_config))
        .context("Failed to load configuration")?;

    match &args.command {
        Commands::Update => {
            let path = config.allowed_signers();
            let sources = config.sources();
            debug!(?sources, "Initialized sources");
            if let Some(users) = config.users(&sources) {
                update(path, &users).context("Failed to update the allowed signers file")?;

                let duration = start.elapsed();
                info!(
                    "Updated allowed signers file {} in {:?}",
                    path.display(),
                    duration
                );
            }
        }
    }
    Ok(())
}

fn setup_tracing(vebosity_level: u8) {
    let level = match vebosity_level {
        0 => return, // The user did not specify a verbosity level, do not configure tracing.
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };
    let filter = {
        // For verbosity levels of 3 and above, given a debug build, traces from external crates are included.
        if vebosity_level > 3 && cfg!(debug_assertions) {
            tracing_subscriber::filter::EnvFilter::new(format!("{level}"))
        } else {
            // Otherwise, traces from external crates are filtered.
            tracing_subscriber::filter::EnvFilter::new(format!(
                "{}={level}",
                env!("CARGO_PKG_NAME")
            ))
        }
    };
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(filter)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Args::command().debug_assert();
    }
}
