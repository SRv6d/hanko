//! The update subcommand used to get the latest allowed signers and write them to the output file.
use crate::{
    config::UserConfiguration, AllowedSignersEntry, Configuration, SourceMap, SshPublicKey,
};
use std::collections::HashSet;
use tokio::runtime::Runtime;

pub(super) fn update(config: Configuration) {
    let rt = Runtime::new().unwrap();
    let sources = config.get_sources();

    let mut allowed_signers: HashSet<AllowedSignersEntry> = HashSet::new();
    if let Some(users) = config.users {
        for user in users {
            let public_keys = rt.block_on(get_public_keys(&user, &sources));
            for public_key in public_keys {
                todo!("Insert allowed signer into set.");
            }
        }
    }
}

async fn get_public_keys(user: &UserConfiguration, sources: &SourceMap) -> Vec<SshPublicKey> {
    let client = reqwest::Client::new();
    let mut keys: Vec<SshPublicKey> = Vec::new();
    for source_name in &user.sources {
        let source = sources.get(source_name).unwrap();
        keys.extend(
            source
                .get_keys_by_username(&user.name, &client)
                .await
                .unwrap(),
        );
    }
    keys
}
