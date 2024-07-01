use odra::casper_types::U256;
use odra::prelude::*;
use odra::Address;
use odra::Var;

use crate::Pubkey;

pub mod errors;
pub mod events;
mod tests;

#[odra::module]
pub struct MessageTransmitter {
    local_domain: Var<u32>,
    version: Var<u32>,
    max_message_body_size: Var<U256>,
    next_available_nonce: Var<u64>,
    owner: Var<Address>,
}

#[odra::module]
impl MessageTransmitter {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        local_domain: u32,
        version: u32,
        max_message_body_size: U256,
        next_available_nonce: u64,
        owner: Address,
    ) {
        self.local_domain.set(local_domain);
        self.version.set(version);
        self.max_message_body_size.set(max_message_body_size);
        self.next_available_nonce.set(next_available_nonce);
        self.owner.set(owner);
    }
    pub fn send_message(&self) {
        todo!("Format a message and emit an event");
    }
    pub fn send_message_with_caller(&self) {
        todo!("Format a message and emit and event");
    }
    pub fn receive_message(&self) {
        // todo: verify attestation signatures
        // todo: check if the signature threshold is met
        // todo: call token_messenger_minter handleReceiveMessage
        todo!("Implement");
    }
    pub fn replace_message(&self) {
        todo!("Implement");
    }
    pub fn set_max_message_body_size(&self) {
        todo!("Implement");
    }
    pub fn set_signature_threshold(&self) {
        todo!("Implement");
    }
    pub fn transfer_ownership(&self) {
        todo!("Implement");
    }
    pub fn accept_ownership(&self) {
        todo!("Implement");
    }
    pub fn pause(&self) {
        todo!("Pause the transmitter");
    }
    pub fn unpause(&self) {
        todo!("Unpause the transmitter");
    }
    pub fn is_nonce_used(&self) -> bool {
        todo!("Implement");
    }
    pub fn get_nonce_pda(&self) {
        todo!("Implement");
    }
}

impl MessageTransmitter {
    fn format_message(
        &self,
        version: u32,
        local_domain: u32,
        destination_domain: u32,
        nonce: u64,
        sender: &Pubkey,
        recipient: &Pubkey,
        // [0;32] if the destination caller can be any
        destination_caller: &Pubkey,
        message_body: &Vec<u8>,
    ) {
        // todo: format message
    }
    pub fn format_burn_message_body(
        version: u32,
        burn_token: &Pubkey,
        mint_recipient: &Pubkey,
        amount: u64,
        message_sender: &Pubkey,
    ) {
        // todo: format burn message body
    }
}

#[cfg(test)]
pub(crate) mod setup_tests {
    use crate::message_transmitter::{MessageTransmitterHostRef, MessageTransmitterInitArgs};
    use odra::host::{Deployer, HostEnv};

    pub fn setup() -> (HostEnv, MessageTransmitterHostRef) {
        let env = odra_test::env();
        let args = MessageTransmitterInitArgs {
            local_domain: 31u32, // 31: CA
            version: 1u32,
            max_message_body_size: 1_000_000_000.into(), // unreasonably high for development
            next_available_nonce: 0,                     // start from nonce = 0
            owner: env.get_account(0),                   // default account as owner
        };
        let message_transmitter = setup_with_args(&env, args);
        (env, message_transmitter)
    }

    pub fn setup_with_args(
        env: &HostEnv,
        args: MessageTransmitterInitArgs,
    ) -> MessageTransmitterHostRef {
        MessageTransmitterHostRef::deploy(env, args)
    }
}
