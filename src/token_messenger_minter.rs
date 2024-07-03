use odra::prelude::*;
use odra::Address;
use odra::SubModule;
use odra::Var;

use crate::Pubkey;

mod burn_message;
pub mod errors;
pub mod events;
pub mod storage;
mod tests;

use storage::RemoteTokenMessengers;

#[odra::module]
pub struct TokenMessengerMinter {
    local_message_transmitter: Var<Address>,
    remote_token_messengers: SubModule<RemoteTokenMessengers>,
    owner: Var<Address>,
}

#[odra::module]
impl TokenMessengerMinter {
    #[allow(clippy::too_many_arguments)]
    pub fn init(&mut self, local_message_transmitter: Address, owner: Address) {
        self.local_message_transmitter
            .set(local_message_transmitter);
        self.owner.set(owner);
    }

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
    pub fn add_remote_token_messenger(&mut self, remote_token_messenger: Pubkey) {
        // todo: access control
        self.remote_token_messengers
            .add_remote_token_messenger(remote_token_messenger);
    }
    pub fn remove_remote_token_messenger(&mut self, remote_token_messenger: Pubkey) {
        // todo: access control
        self.remote_token_messengers
            .remove_remote_token_messenger(remote_token_messenger);
    }

    pub fn link_token_pair(&self) {}
    pub fn unlink_token_pair(&self) {}
    pub fn pause(&self) {}
    pub fn unpause(&self) {}
    pub fn set_max_burn_amount_per_message(&self) {}
}

#[cfg(test)]
pub(crate) mod setup_tests {
    use odra::host::Deployer;
    use odra::host::HostEnv;

    use crate::token_messenger_minter::{
        TokenMessengerMinterHostRef, TokenMessengerMinterInitArgs,
    };

    pub fn setup() -> (HostEnv, TokenMessengerMinterHostRef) {
        let env = odra_test::env();
        let args = TokenMessengerMinterInitArgs {
            local_message_transmitter: env.get_account(0), // default account,
            owner: env.get_account(0),                     //default account
        };
        let token_messenger_minter = setup_with_args(&env, args);
        (env, token_messenger_minter)
    }

    pub fn setup_with_args(
        env: &HostEnv,
        args: TokenMessengerMinterInitArgs,
    ) -> TokenMessengerMinterHostRef {
        TokenMessengerMinterHostRef::deploy(env, args)
    }
}
