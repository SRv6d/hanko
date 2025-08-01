pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub use source::{Error, Github, Gitlab, Source};

pub mod allowed_signers;
pub mod cli;
pub mod config;
mod source;
