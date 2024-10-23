pub use github::Github;
pub use gitlab::Gitlab;
pub use main::{Source, SourceError};

mod github;
mod gitlab;
mod main;
