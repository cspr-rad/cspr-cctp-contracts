use crate::{GenericAddress, Pubkey};
use odra::{casper_types::ApiError, prelude::*, Mapping, Var};
use tiny_keccak::Keccak;

#[odra::module()]
/// Storage module for the allowances of the token.
pub struct UsedNonces {
    first_nonce: Var<u64>,
    used_nonces: Mapping<[u8; 32], bool>,
}

#[odra::module]
impl UsedNonces {
    pub fn use_nonce(&mut self, nonce: u64, nonce_hashed: [u8; 32]) {
        if nonce < self.first_nonce.get().unwrap() {
            todo!("Revert with a meaningful error");
        }
        self.used_nonces.set(&nonce_hashed, true);
    }
    pub fn is_used_nonce(&self, nonce: u64, nonce_hashed: [u8; 32]) -> bool {
        if nonce < self.first_nonce.get().unwrap() {
            return true;
        }
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
