#![warn(clippy::pedantic)]
#![warn(clippy::panic)]
#![forbid(unsafe_code)]

pub use config::Config;
pub use core::*;
pub use signer::{AllowedSignersEntry, AllowedSignersFile};
pub use source::{Github, Gitlab, Source};

pub mod cli;
mod config;
mod core;
mod signer;
mod source;
