use clap::Subcommand;

#[derive(Subcommand)]
pub enum ManageSigners {
    /// Add an allowed signer.
    Add,
    /// Remove an allowed signer.
    Remove,
}
