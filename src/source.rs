use crate::SshPublicKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Source {
    source_type: SourceType,
    url: String,
}

impl Source {
    fn get_keys_by_username(username: &str) -> Vec<SshPublicKey> {
        todo!();
    }
}

/// The source type.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Github,
    Gitlab,
}
