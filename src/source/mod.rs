pub use github::Github;
pub use gitlab::Gitlab;
pub use main::{Error, Source};

mod github;
mod gitlab;
mod main;
