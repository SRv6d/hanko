use crate::GitProvider;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum ManageSources {
    /// Add sources.
    Add {
        /// The name of the source.
        name: String,
        /// The Git provider used by the source.
        #[arg(short, long)]
        provider: GitProvider,
        /// The URL of the source.
        #[arg(short, long)]
        url: Option<reqwest::Url>,
    },
    /// Remove sources.
    Remove { name: Vec<String> },
}
