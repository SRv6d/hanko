use crate::SshPublicKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Source<P: Provider> {
    url: String,
    provider: P,
}

impl<P: Provider> Source<P> {
    pub fn new(url: String, source_type: SourceType) -> Self {
        todo!()
    }
}

/// The source type.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Github,
    Gitlab,
}

trait Provider {
    async fn get_keys_by_username(&self, username: &str) -> Vec<SshPublicKey>;
}
