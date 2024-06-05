use super::{manage_signers::ManageSigners, manage_sources::ManageSources, update::update};
use crate::Configuration;
use anyhow::{Context, Result};
use clap::{
    builder::{OsStr, Resettable},
    Parser, Subcommand,
};
use std::{env, path::PathBuf};
use tracing::Level;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// The configuration file.
    #[arg(
        short,
        long,
        value_name = "PATH",
        env = "HANKO_CONFIG",
        default_value = default_config_path()
    )]
    pub config: PathBuf,

    /// Override where allowed signers are written to.
    #[arg(long, value_name = "PATH", env = "HANKO_OUTPUT")]
    pub output: Option<PathBuf>,

    /// Increase verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Update the allowed signers file.
    Update,

    /// Manage signers.
    #[command(subcommand)]
    Signer(ManageSigners),

    /// Manage sources.
    #[command(subcommand)]
    Source(ManageSources),
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

/// The main CLI entrypoint.
pub fn entrypoint() -> Result<()> {
    let cli = Cli::parse();

    setup_tracing(cli.verbose);

    let config = Configuration::load(&cli.config, true).context("Failed to load configuration")?;

    match &cli.command {
        Commands::Update => update(config).context("Failed to update the allowed signers file")?,
        _ => panic!("Not yet implemented"),
    }
    Ok(())
}

fn setup_tracing(level: u8) {
    if level == 0 {
        return;
    }

    let builder = tracing_subscriber::fmt()
        .compact()
        .with_max_level(match level {
            0 => unreachable!(),
            1 => Level::INFO,
            2 => Level::DEBUG,
            _ => Level::TRACE,
        });

    builder.init();
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
