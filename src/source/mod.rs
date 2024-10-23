pub use github::Github;
pub use gitlab::Gitlab;
pub use main::{Source, Error};

mod github;
mod gitlab;
mod main;
