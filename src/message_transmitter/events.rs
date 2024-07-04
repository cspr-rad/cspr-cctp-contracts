use crate::{GenericAddress, Pubkey};
use odra::prelude::*;

#[odra::event]
pub struct MessageSent {
    pub message: Vec<u8>,
}

#[odra::event]
pub struct MessageReceived {
    pub caller: GenericAddress,
    pub source_domain: u32,
    pub nonce: u64,
    pub sender: Pubkey,
    pub message_body: Vec<u8>,
}
