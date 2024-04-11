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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        AllowedSigner {
            principal: "j.snow@wall.com".to_string(),
            valid_after: None,
            valid_before: None,
            key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS"
                .parse()
                .unwrap(),
        },
        "j.snow@wall.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGtQUDZWhs8k/cZcykMkaoX7ZE7DXld8TP79HyddMVTS"
    )]
    )]
    fn display_allowed_signer(#[case] signer: AllowedSigner, #[case] expected_display: &str) {
        assert_eq!(signer.to_string(), expected_display);
    }
}
