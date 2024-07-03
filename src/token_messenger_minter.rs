use burn_message::BurnMessage;
use odra::prelude::*;
use odra::Address;
use odra::SubModule;
use odra::Var;

use crate::GenericAddress;
use crate::Pubkey;

mod burn_message;
pub mod errors;
pub mod events;
pub mod storage;
mod tests;

use storage::RemoteTokenMessengers;

#[odra::module]
pub struct TokenMessengerMinter {
    version: Var<u32>,
    local_message_transmitter: Var<Address>,
    remote_token_messengers: SubModule<RemoteTokenMessengers>,
    owner: Var<Address>,
}

#[odra::module]
impl TokenMessengerMinter {
    #[allow(clippy::too_many_arguments)]
    pub fn init(&mut self, version: u32, local_message_transmitter: Address, owner: Address) {
        self.version.set(version);
        self.local_message_transmitter
            .set(local_message_transmitter);
        self.owner.set(owner);
    }

    pub fn deposit_for_burn(&self, amount: u64, destination_domain: u32, mint_recipient: Pubkey) {

    }

    pub fn deposit_for_burn_with_caller(&self) {}

    pub fn replace_deposit_for_burn(&self) {}

    pub fn handle_receive_message(&self, remote_domain: u32, sender: Pubkey, message_body: &Vec<u8>) {
        // todo: validate burn message format
        let burn_message: BurnMessage = BurnMessage{
            data: &message_body
        };
        assert_eq!(self.version.get().unwrap(), burn_message.version());
        let mint_recipient: GenericAddress = burn_message.mint_recipient();
        let burn_token: GenericAddress = burn_message.burn_token();
        let amount: u64 = burn_message.amount();

        // todo: find local minter for the token
        // and mint amount to mint_recipient
    }

    pub fn transfer_ownership(&self) {
        todo!("Implement");
    }
    pub fn accept_ownership(&self) {
        todo!("Implement");
    }
    pub fn add_remote_token_messenger(&mut self, domain: u32, remote_token_messenger: Pubkey) {
        self.require_owner();
        self.remote_token_messengers
            .add_remote_token_messenger(domain, remote_token_messenger);
    }
    pub fn remove_remote_token_messenger(&mut self, domain: u32, remote_token_messenger: Pubkey) {
        self.require_owner();
        self.remote_token_messengers
            .remove_remote_token_messenger(domain, remote_token_messenger);
    }

    fn send_deposit_for_burn_message(destination_domain: u32, destination_token_messenger: Pubkey, destination_caller: Pubkey, burn_message: &Vec<u8>){
        todo!(r#"
            Send message either to local_transmitter::send_message
            or local_transmitter::send_message_with_caller
        "#)
    }

    fn mint_and_withdraw(){

    }

    pub fn link_token_pair(&self) {}
    pub fn unlink_token_pair(&self) {}
    pub fn pause(&self) {}
    pub fn unpause(&self) {}
    pub fn set_max_burn_amount_per_message(&self) {}
    fn require_owner(&self) {
        if self.env().caller() != self.owner.get().unwrap() {
            todo!("Throw a meaningful error")
        }
    }
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
            version: 2u32,
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
