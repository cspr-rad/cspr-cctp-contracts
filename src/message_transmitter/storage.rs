use odra::{prelude::*, Mapping, Var};

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
        if nonce < self.first_nonce.get().unwrap(){
            return true;
        }
        self.used_nonces.get(&nonce).unwrap_or_default()
    }
}
