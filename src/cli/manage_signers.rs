use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum ManageSigners {
    /// Add allowed signers.
    Add {
        #[command(flatten)]
        signers: Signers,
        /// The source(s) of the signer(s) to add.
        #[arg(short, long)]
        source: Vec<String>,
    },
    /// Remove allowed signers.
    Remove {
        #[command(flatten)]
        signers: Signers,
    },
}

#[derive(Debug, Args)]
#[group(multiple = true)]
pub struct Signers {
    /// By username.
    #[arg(short, long)]
    user: Vec<String>,
    /// By organization.
    #[arg(short, long, value_name = "ORGANIZATION")]
    org: Vec<String>,
}
