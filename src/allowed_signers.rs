//! Interact with the OpenSSH `allowed_signers` file.
//!
//! # File format
//! https://man.openbsd.org/ssh-keygen.1#ALLOWED_SIGNERS
use crate::SshPublicKey;
use chrono::{DateTime, Local};

/// A single entry in the allowed signers file.
#[derive(Debug)]
struct AllowedSigner {
    principal: String,
    valid_after: Option<DateTime<Local>>,
    valid_before: Option<DateTime<Local>>,
    key: SshPublicKey,
}
