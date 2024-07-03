use crate::Pubkey;
use odra::{prelude::*, Mapping};

#[odra::module()]
/// Storage module for the allowances of the token.
pub struct RemoteTokenMessengers {
    roles: Mapping<(u32, Pubkey), bool>,
}

#[odra::module]
impl RemoteTokenMessengers {
    pub fn add_remote_token_messenger(&mut self, domain: u32, remote_token_messenger: Pubkey) {
        self.roles.set(&(domain, remote_token_messenger), true);
    }
    pub fn remove_remote_token_messenger(&mut self, domain: u32, remote_token_messenger: Pubkey) {
        self.roles.set(&(domain, remote_token_messenger), false);
    }
    pub fn is_remote_token_messenger(&mut self, domain: u32, remote_token_messenger: Pubkey) -> bool {
        self.roles.get(&(domain, remote_token_messenger)).unwrap_or_default()
    }
}
