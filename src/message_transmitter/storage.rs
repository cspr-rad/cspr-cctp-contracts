use crate::Pubkey;
use odra::{prelude::*, Mapping};

#[odra::module()]
/// Storage module for the allowances of the token.
pub struct UsedNonces {
    used_nonces: Mapping<[u8; 32], bool>,
}

#[odra::module]
impl UsedNonces {
    pub fn use_nonce(&mut self, nonce_hashed: [u8; 32]) {
        self.used_nonces.set(&nonce_hashed, true);
    }
    pub fn is_used_nonce(&self, nonce_hashed: [u8; 32]) -> bool {
        self.used_nonces.get(&nonce_hashed).unwrap_or_default()
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
