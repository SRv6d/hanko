#![warn(clippy::pedantic)]
#![warn(clippy::panic)]
#![forbid(unsafe_code)]

pub use allowed_signers::{AllowedSigner, AllowedSignersFile};
pub use config::Config;
pub use public_key::SshPublicKey;
pub use source::Source;
pub use user::User;

mod allowed_signers;
pub mod cli;
mod config;
mod public_key;
mod source;
mod user;

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
