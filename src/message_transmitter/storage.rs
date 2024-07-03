use odra::{prelude::*, Mapping, Var};

use crate::Pubkey;

#[odra::module()]
/// Storage module for the allowances of the token.
pub struct UsedNonces {
    first_nonce: Var<u64>,
    used_nonces: Mapping<u64, bool>,
}

#[odra::module]
impl UsedNonces {
    pub fn use_nonce(&mut self, nonce: u64) {
        self.used_nonces.set(&nonce, true);
    }
    pub fn is_used_nonce(&self, nonce: u64) -> bool {
        if nonce < self.first_nonce.get().unwrap() {
            return true;
        }
        self.used_nonces.get(&nonce).unwrap_or_default()
    }
}

#[odra::module()]
pub struct Attesters {
    attesters: Mapping<Pubkey, bool>,
}

#[odra::module]
impl Attesters {
    pub fn enable_attester(&mut self, attester: Pubkey) {
        self.attesters.set(&attester, true);
    }
    pub fn disable_attester(&mut self, attester: Pubkey) {
        self.attesters.set(&attester, false);
    }
    pub fn is_attester(&self, attester: Pubkey) -> bool {
        self.attesters.get(&attester).unwrap()
    }
}
