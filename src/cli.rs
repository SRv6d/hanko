use crate::{
    allowed_signers,
    config::{default_user_source, Configuration},
};
use anyhow::{Context, Result};
use clap::{
    builder::{OsStr, Resettable},
    Parser, Subcommand,
};
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
    time::Instant,
};
use tracing::{info, Level};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[command(flatten)]
    global_args: GlobalArgs,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Update the allowed signers file.
    Update,
    /// Manage allowed signers.
    #[command(subcommand)]
    Signer(ManageSigners),
}

#[derive(Debug, Serialize, Deserialize, clap::Args)]
struct GlobalArgs {
    /// The configuration file.
    #[arg(
        short,
        long,
        value_name = "PATH",
        env = "HANKO_CONFIG",
        global = true,
        default_value = default_config_path()
    )]
    pub config: PathBuf,

    /// The allowed signers file.
    #[arg(
        long,
        value_name = "PATH",
        env = "HANKO_ALLOWED_SIGNERS",
        global = true,
        default_value = git_allowed_signers()
    )]
    pub file: PathBuf,

    /// Use verbose output.
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Debug, Subcommand)]
enum ManageSigners {
    /// Add an allowed signer.
    Add {
        /// The name of the signer to add.
        name: String,
        /// The principals of the signer to add.
        principals: Vec<String>,
        /// The source(s) of the signer to add.
        #[arg(short, long, default_values_t = default_user_source())]
        source: Vec<String>,
        /// Don't update the allowed signers file with the added signer(s).
        #[arg(long)]
        no_update: bool,
    },
}

/// The default configuration file path according to the XDG Base Directory Specification.
/// If neither `$XDG_CONFIG_HOME` nor `$HOME` are set, [`Resettable::Reset`] is returned, forcing the user to specify the path.
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
/// If the `detect-allowed-signers` feature is not enabled or no allowed signers file
/// is configured within Git, the user will be forced to specify a path manually.
fn git_allowed_signers() -> Resettable<OsStr> {
    #[cfg(feature = "detect-allowed-signers")]
    if let Ok(file) = gix_config::File::from_globals() {
        if let Some(path) = file.path("gpg.ssh.allowedsignersfile") {
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
    let cli = Cli::parse();
    let args = cli.global_args;
    let signers_file = &args.file;

    setup_tracing(args.verbose);

    let mut config = Configuration::load(&args.config).context(format!(
        "Failed to load configuration from {}",
        &args.config.display()
    ))?;

    match cli.command {
        Commands::Update => {}
        Commands::Signer(action) => match action {
            ManageSigners::Add {
                name,
                principals,
                source,
                no_update,
            } => {
                config
                    .add_signer(name, principals, source)
                    .context("Failed to add allowed signer")?;
                config.save().context(format!(
                    "Failed to save configuration to {}",
                    &args.config.display()
                ))?;
                if no_update {
                    return Ok(());
                }
            }
        },
    }

    update_allowed_singers(signers_file, &config)
}

#[tokio::main]
async fn update_allowed_singers(file: &Path, config: &Configuration) -> Result<()> {
    let start = Instant::now();

    let sources = config.sources();
    let signers = config.signers(&sources);

    allowed_signers::update(file, signers)
        .await
        .context("Failed to update the allowed signers file")?;

    let duration = start.elapsed();
    info!(
        "Updated allowed signers file {} in {:?}",
        file.display(),
        duration
    );
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
        Cli::command().debug_assert();
    }
}
