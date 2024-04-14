use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// The path to the configuration file.
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(flatten)]
    logging: Logging,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Args)]
#[group(multiple = false)]
struct Logging {
    /// Enable verbose logging.
    #[arg(short, long, group = "logging")]
    verbose: bool,

    /// Disable all output.
    #[arg(long, group = "logging")]
    silent: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Update the allowed signers file.
    Update,

    /// Add allowed signers.
    Add,

    /// Remove allowed signers.
    Remove,
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
