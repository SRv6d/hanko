pub use github::Github;
pub use gitlab::Gitlab;
pub(super) use main::Result;
pub use main::{Source, SourceError, SourceMap};

mod github;
mod gitlab;
mod main;
