pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Returns the parent directory of the given path with appropriate context added in the error case.
pub(crate) fn parent_dir(path: &std::path::Path) -> anyhow::Result<&std::path::Path> {
    use anyhow::Context;

    path.parent()
        .with_context(|| format!("{} has no parent directory", path.display()))
}

pub use source::{Error, Github, Gitlab, Protocol, Source};

pub mod allowed_signers;
pub mod cli;
pub mod config;
mod source;
