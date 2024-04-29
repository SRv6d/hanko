#![warn(clippy::pedantic)]
#![warn(clippy::panic)]
#![forbid(unsafe_code)]

pub use allowed_signers::{AllowedSigner, AllowedSignersFile};
pub use config::Config;
pub use provider::GitProvider;
pub use public_key::SshPublicKey;

mod allowed_signers;
pub mod cli;
mod config;
mod provider;
mod public_key;
mod user;

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
