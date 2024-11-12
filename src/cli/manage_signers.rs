use crate::config::default_user_source;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum ManageSigners {
    /// Add allowed signers.
    Add {
        /// The name(s) of the signers(s) to add.
        name: Vec<String>,
        /// The source(s) of the given signer(s).
        #[arg(short, long, default_values_t = default_user_source())]
        source: Vec<String>,
    },
}
