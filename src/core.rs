use serde::Deserialize;
use std::{fmt, str::FromStr};

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct SshPublicKey {
    key: String,
}

impl FromStr for SshPublicKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SshPublicKey { key: s.to_string() })
    }
}

impl fmt::Display for SshPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}
