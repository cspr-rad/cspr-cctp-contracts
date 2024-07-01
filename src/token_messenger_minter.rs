use odra::prelude::*;
use odra::Var;

mod tests;
pub mod errors;
pub mod events;
pub mod storage;

#[odra::module]
pub struct TokenMessengerMinter {

}

#[odra::module]
impl TokenMessengerMinter {

    pub fn depositForBurn(&self){
        // this entry point may be called by a user
        // tokens will be burned and a formatted burn message will be sent to the MessageTransmitter
        // the MessageTransmitter will then emit a MessageSent Event
        // the Message body will be the BurnMessage
    }

    pub fn handleReceiveMessage(&self){
        // caller must be message_transmitter
        // message_transmitter has already checked the signatures
        // call stablecoin::mint if the message is a stablecoin burn message
        // ... for other messages
    }
}