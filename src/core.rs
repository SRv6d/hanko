use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
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

#[async_trait]
pub trait GetPublicKeys {
    type Err;

    /// Get the public keys of a user by their username.
    async fn by_username(&self, username: &str) -> Result<Vec<SshPublicKey>, Self::Err>;
}
