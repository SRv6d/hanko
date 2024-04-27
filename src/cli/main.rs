use super::{manage_signers::ManageSigners, manage_sources::ManageSources};
use clap::{
    builder::{OsStr, Resettable},
    Args, Parser, Subcommand,
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

    /// The allowed signers file used by Git.
    #[arg(
        long,
        value_name = "PATH",
        env = "HANKO_ALLOWED_SIGNERS",
        default_value = git_allowed_signers_path()
    )]
    pub allowed_signers: PathBuf,

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
        Resettable::Value(format!("{}/{}/{}", xdg_config_home, dirname, filename).into())
    } else if let Ok(home) = env::var("HOME") {
        Resettable::Value(format!("{}/.config/{}/{}", home, dirname, filename).into())
    } else {
        Resettable::Reset
    }
}

/// The path to the allowed signers file as configured within Git.
fn git_allowed_signers_path() -> Resettable<OsStr> {
    // TODO: Get value from Git config.
    Resettable::Value("~/.config/git/allowed_signers".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
