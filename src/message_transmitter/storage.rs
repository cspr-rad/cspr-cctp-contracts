use crate::Hash;
use odra::{prelude::*, Mapping};

#[odra::module()]
/// Storage module for the allowances of the token.
pub struct UsedNonces {
    roles: Mapping<Hash, bool>,
}

#[odra::module]
impl UsedNonces {
    pub fn use_nonce(&mut self, source_and_nonce: Hash) {
        self.roles.set(&source_and_nonce, true);
    }
    pub fn is_used_nonce(&mut self, source_and_nonce: Hash) -> bool {
        self.roles.get(&source_and_nonce).unwrap_or_default()
    }
}
