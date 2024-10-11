use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// An SSH public key.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PublicKey {
    key: String,
    // TODO: Add expiration field for GitLab keys.
}

impl FromStr for PublicKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PublicKey { key: s.to_string() })
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}
