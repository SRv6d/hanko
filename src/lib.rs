#![warn(clippy::pedantic)]
#![warn(clippy::panic)]
#![forbid(unsafe_code)]

pub use allowed_signers::{AllowedSigner, AllowedSignersFile};
pub use config::Config;
pub use core::*;
pub use source::Source;

mod allowed_signers;
pub mod cli;
mod config;
mod core;
mod source;
