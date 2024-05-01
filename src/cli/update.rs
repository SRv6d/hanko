//! The update subcommand used to get the latest allowed signers and write them to the output file.
use std::collections::{HashMap, HashSet};

use crate::{AllowedSigner, Config, SshPublicKey};

pub(super) fn update(config: Config) {
    let sources = config.get_sources();

    let mut allowed_signers: HashSet<AllowedSigner> = HashSet::new();
    if let Some(users) = config.users {
        for user in users {
            let public_keys = get_public_keys((), sources.clone());
            for public_key in public_keys {
                todo!("Insert allowed signer into set.");
            }
        }
    }
}

fn get_public_keys(user: (), sources: HashMap<String, ()>) -> Vec<SshPublicKey> {
    todo!("Retrieve public keys for a user from all sources.");
}
