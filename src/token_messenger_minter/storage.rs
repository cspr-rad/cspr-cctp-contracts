use crate::Pubkey;
use odra::{prelude::*, Mapping};

#[odra::module()]
/// Storage module for the allowances of the token.
pub struct RemoteTokenMessengers {
    remote_token_messengers: Mapping<u32, Option<Pubkey>>,
}

#[odra::module]
impl RemoteTokenMessengers {
    pub fn add_remote_token_messenger(&mut self, domain: u32, remote_token_messenger: Pubkey) {
        self.remote_token_messengers
            .set(&domain, Some(remote_token_messenger));
    }
    pub fn remove_remote_token_messenger(&mut self, domain: u32) {
        self.remote_token_messengers.set(&domain, None);
    }
    pub fn get_remote_token_messenger(&self, domain: u32) -> Option<Pubkey> {
        self.remote_token_messengers.get(&domain).unwrap()
    }
}
