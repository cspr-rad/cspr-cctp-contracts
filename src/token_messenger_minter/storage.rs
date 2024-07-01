use alloc::string::String;
use odra::casper_types::U256;
use odra::ExecutionError::AdditionOverflow;

use odra::casper_types::bytesrepr::ToBytes;
use odra::{prelude::*, Mapping};
use odra::{Address, UnwrapOrRevert};

use base64::prelude::*;

use crate::Pubkey;

#[odra::module()]
/// Storage module for the allowances of the token.
pub struct RemoteTokenMessengers {
    roles: Mapping<Pubkey, bool>,
}

#[odra::module]
impl RemoteTokenMessengers {
    pub fn add_remote_token_messenger(&mut self, remote_token_messenger: Pubkey) {
        self.roles.set(&remote_token_messenger, true);
    }
    pub fn remove_remote_token_messenger(&mut self, remote_token_messenger: Pubkey) {
        self.roles.set(&remote_token_messenger, false);
    }
    pub fn is_remote_token_messenger_active(&mut self, remote_token_messenger: Pubkey) -> bool {
        self.roles.get(&remote_token_messenger).unwrap_or_default()
    }
}
