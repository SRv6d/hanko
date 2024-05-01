//! The update subcommand used to get the latest allowed signers and write them to the output file.
use crate::{AllowedSignersEntry, Configuration, SourceMap, SshPublicKey};
use std::collections::HashSet;

pub(super) fn update(config: Configuration) {
    let sources = config.get_sources();

    let mut allowed_signers: HashSet<AllowedSignersEntry> = HashSet::new();
    if let Some(users) = config.users {
        for user in users {
            let public_keys = get_public_keys((), &sources);
            for public_key in public_keys {
                todo!("Insert allowed signer into set.");
            }
        }
    }
}

fn get_public_keys(user: (), sources: &SourceMap) -> Vec<SshPublicKey> {
    todo!("Retrieve public keys for a user from all sources.");
}
