#![warn(clippy::pedantic)]
#![warn(clippy::panic)]
#![forbid(unsafe_code)]

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub use config::Configuration;
pub use source::{Error, Github, Gitlab, Source};

pub mod allowed_signers;
pub mod cli;
mod config;
mod source;
