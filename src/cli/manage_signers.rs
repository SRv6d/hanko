use crate::config::default_user_source;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum ManageSigners {
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
