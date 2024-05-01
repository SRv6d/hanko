use super::{manage_signers::ManageSigners, manage_sources::ManageSources, update::update};
use crate::Configuration;
use clap::{
    builder::{OsStr, Resettable},
    Args, Parser, Subcommand,
};
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use std::{env, path::PathBuf};

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

    #[command(flatten)]
    logging: Logging,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Args)]
#[group(multiple = false)]
struct Logging {
    /// Enable verbose logging.
    #[arg(short, long)]
    verbose: bool,

    /// Disable all output.
    #[arg(long)]
    silent: bool,
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
pub fn entrypoint() {
    let cli = Cli::parse();
    let config: Configuration = Figment::from(Serialized::defaults(Configuration::default()))
        .admerge(Toml::file(cli.config))
        .extract()
        .unwrap();

    match &cli.command {
        Commands::Update => {
            update(config);
        }
        Commands::Signer(_) => {
            panic!("Not yet implemented");
        }
        Commands::Source(_) => {
            panic!("Not yet implemented");
        }
    }
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
