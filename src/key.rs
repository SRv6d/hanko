use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

use crate::{Source, SourceError};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

/// Get all [`SshPublicKey`]s associated with a user from the given sources.
pub async fn get_public_keys(
    username: &str,
    sources: impl IntoIterator<Item = &dyn Source>,
    client: &Client,
) -> Result<Vec<SshPublicKey>, SourceError> {
    let mut keys: Vec<SshPublicKey> = Vec::new();
    for source in sources {
        keys.extend(source.get_keys_by_username(username, client).await?);
    }
    Ok(keys)
}
