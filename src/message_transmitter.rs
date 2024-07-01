use odra::casper_types::U256;
use odra::prelude::*;
use odra::Address;
use odra::Var;

mod tests;
pub mod errors;
pub mod events;

// type alias for generic Pubkey
pub type Pubkey = [u8;32];

#[odra::module]
pub struct MessageTransmitter {
    local_domain: Var<u32>,
    version: Var<u32>,
    max_message_body_size: Var<U256>,
    next_available_nonce: Var<u64>,
    owner: Var<Address>
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
        owner: Address
    ) {
        self.local_domain.set(local_domain);
        self.version.set(version);
        self.max_message_body_size.set(max_message_body_size);
        self.next_available_nonce.set(next_available_nonce);
        self.owner.set(owner);
    }
    pub fn send_message(&self){
        // todo: format the message and emit an event
    }
    pub fn receive_message(&self){
        // todo: verify attestation signatures
        // todo: check if the signature threshold is met
        // todo: call token_messenger_minter handleReceiveMessage
    }
}

/* What should the MessageTransmitter do?
    send Message
    receive Message
*/

#[cfg(test)]
pub (crate) mod setup_tests {
    use odra::host::{Deployer, HostEnv};
    use crate::message_transmitter::{MessageTransmitterHostRef, MessageTransmitterInitArgs};

    pub fn setup() -> (
        HostEnv,
        MessageTransmitterHostRef,
    ) {
        let env = odra_test::env();
        let args = MessageTransmitterInitArgs {
            local_domain: 31u32, // 31: CA
            version: 1u32,
            max_message_body_size: 1_000_000_000.into(), // unreasonably high for development
            next_available_nonce: 0, // start from nonce = 0
            owner: env.get_account(0) // default account as owner
        };
        let message_transmitter = setup_with_args(&env, args);
        (
            env,
            message_transmitter,
        )
    }

    pub fn setup_with_args(env: &HostEnv, args: MessageTransmitterInitArgs) -> MessageTransmitterHostRef {
        MessageTransmitterHostRef::deploy(env, args)
    }
}