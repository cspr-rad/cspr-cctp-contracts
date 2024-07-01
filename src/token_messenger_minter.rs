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

    }

    pub fn handleReceiveMessage(&self){
        // caller must be message_transmitter
        // message_transmitter has already checked the signatures
        // call stablecoin::mint if the message is a stablecoin burn message
        // ... for other messages
    }
}

/* What should the TokenMessengerMinter do?
    depositForBurn(
        amount,
        destinationDomain,
        mintRecipient,
        burnToken
    ) // will call message_transmitter to send a burn message
    
    handleReceiveMessage{
   
    } // will be called by message_transmitter to handle a signed mint message
      // must verify all the signatures

    add RemoteTokenMessenger
    remove RemoteTokenMessenger
    addLocalMinter
    removeLocalMinter

    LocalMessageTransmitter Address is set on deployment.
*/
