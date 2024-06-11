#![warn(clippy::pedantic)]
#![warn(clippy::panic)]
#![forbid(unsafe_code)]

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub use config::Configuration;
pub use key::SshPublicKey;
pub use signer::{AllowedSignersEntry, AllowedSignersFile};
pub use source::{Github, Gitlab, Source, SourceError, SourceMap};

pub mod cli;
mod config;
mod key;
mod signer;
mod source;
mod user;
