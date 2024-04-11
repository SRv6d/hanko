//! Interact with the OpenSSH `allowed_signers` file.
//!
//! # File format
//! https://man.openbsd.org/ssh-keygen.1#ALLOWED_SIGNERS
use crate::SshPublicKey;
use chrono::{DateTime, Local};
use std::fmt;

/// The format string for time fields.
const TIME_FMT: &str = "%Y%m%d%H%M%S";

/// A single entry in the allowed signers file.
#[derive(Debug)]
struct AllowedSigner {
    principal: String,
    valid_after: Option<DateTime<Local>>,
    valid_before: Option<DateTime<Local>>,
    key: SshPublicKey,
}

impl fmt::Display for AllowedSigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.principal)?;

        if let Some(valid_after) = self.valid_after {
            write!(f, " valid-after={}", valid_after.format(TIME_FMT))?;
        };
        if let Some(valid_before) = self.valid_before {
            write!(f, " valid-before={}", valid_before.format(TIME_FMT))?;
        };

        write!(f, " {}", self.key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone as _;
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
    #[case(
        AllowedSigner {
            principal: "ian.malcom@acme.corp".to_string(),
            valid_after: Some(Local.with_ymd_and_hms(2024, 4, 11, 22, 00, 00).unwrap()),
            valid_before: None,
            key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILWtK6WxXw7NVhbn6fTQ0dECF8y98fahSIsqKMh+sSo9"
                .parse()
                .unwrap(),
        },
        "ian.malcom@acme.corp valid-after=20240411220000 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILWtK6WxXw7NVhbn6fTQ0dECF8y98fahSIsqKMh+sSo9"
    )]
    #[case(
        AllowedSigner {
            principal: "cwoods@universal.exports".to_string(),
            valid_after: None,
            valid_before: Some(Local.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()),
            key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw"
                .parse()
                .unwrap(),
        },
        "cwoods@universal.exports valid-before=20300101000000 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJHDGMF+tZQL3dcr1arPst+YP8v33Is0kAJVvyTKrxMw"
    )]
    fn display_allowed_signer(#[case] signer: AllowedSigner, #[case] expected_display: &str) {
        assert_eq!(signer.to_string(), expected_display);
    }
}
