use odra::prelude::*;
use odra::Address;
use odra::Var;

pub mod errors;
pub mod events;
pub mod storage;
mod tests;

#[odra::module]
pub struct TokenMessengerMinter {
    message_transmitter: Var<Address>,
}

#[odra::module]
impl TokenMessengerMinter {
    pub fn deposit_for_burn(&self) {
        // this entry point may be called by a user
        // tokens will be burned and a formatted burn message will be sent to the MessageTransmitter
        // the MessageTransmitter will then emit a MessageSent Event
        // the Message body will be the BurnMessage
    }

    pub fn deposit_for_burn_with_caller(&self) {}

    pub fn replace_deposit_for_burn(&self) {}

    pub fn handle_receive_message(&self) {
        // caller must be message_transmitter
        // message_transmitter has already checked the signatures
        // call stablecoin::mint if the message is a stablecoin burn message
        // ... for other messages
    }

    pub fn transfer_ownership(&self) {
        todo!("Implement");
    }
    pub fn accept_ownership(&self) {
        todo!("Implement");
    }
    pub fn add_remote_token_messenger(&self) {}
    pub fn remove_remote_token_messenger(&self) {}
}
