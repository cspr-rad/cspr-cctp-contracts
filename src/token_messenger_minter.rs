use burn_message::BurnMessage;
use odra::casper_types::U256;
use odra::prelude::*;
use odra::Address;
use odra::SubModule;
use odra::UnwrapOrRevert;
use odra::Var;

use crate::generic_address;
use crate::generic_address_to_contract_address;
use crate::GenericAddress;
use crate::Pubkey;

mod burn_message;
pub mod errors;
pub mod events;
pub mod storage;
mod tests;

use crate::message_transmitter::MessageTransmitterContractRef;
use crate::stablecoin::StablecoinContractRef;
use storage::RemoteTokenMessengers;

#[odra::module]
pub struct TokenMessengerMinter {
    version: Var<u32>,
    paused: Var<bool>,
    local_message_transmitter: Var<Address>,
    remote_token_messengers: SubModule<RemoteTokenMessengers>,
    owner: Var<Address>,
    pending_owner: Var<Option<Address>>,
}

#[odra::module]
impl TokenMessengerMinter {
    #[allow(clippy::too_many_arguments)]
    pub fn init(&mut self, version: u32, local_message_transmitter: Address, owner: Address) {
        self.version.set(version);
        self.paused.set(false);
        self.local_message_transmitter
            .set(local_message_transmitter);
        self.owner.set(owner);
        self.pending_owner.set(None);
    }

    pub fn deposit_for_burn(
        &self,
        amount: u64,
        destination_domain: u32,
        mint_recipient: Pubkey,
        burn_token: GenericAddress,
    ) {
        let destination_caller: Pubkey = [0u8; 32];
        self._deposit_for_burn(
            amount,
            destination_domain,
            mint_recipient,
            burn_token,
            destination_caller,
        );
    }

    pub fn deposit_for_burn_with_caller(
        &self,
        amount: u64,
        destination_domain: u32,
        mint_recipient: Pubkey,
        burn_token: GenericAddress,
        destination_caller: Pubkey,
    ) {
        self._deposit_for_burn(
            amount,
            destination_domain,
            mint_recipient,
            burn_token,
            destination_caller,
        );
    }

    pub fn replace_deposit_for_burn(&self) {}

    pub fn handle_receive_message(
        &self,
        remote_domain: u32,
        sender: Pubkey,
        message_body: &Vec<u8>,
    ) {
        self.require_local_message_transmitter();
        // todo: validate burn message format
        let burn_message: BurnMessage = BurnMessage {
            data: &message_body,
        };
        assert_eq!(self.version.get().unwrap(), burn_message.version());
        let mint_recipient: GenericAddress = burn_message.mint_recipient();
        let burn_token: Pubkey = burn_message.burn_token();
        let amount: u64 = burn_message.amount();

        todo!("Finish mint function");
        self.mint(remote_domain, burn_token, mint_recipient);
    }
    pub fn transfer_ownership(&mut self, new_pending_owner: Address) {
        self.require_owner();
        self.pending_owner.set(Some(new_pending_owner));
    }
    pub fn accept_ownership(&mut self) {
        let pending_owner = self.pending_owner.get().unwrap().unwrap();
        if self.env().caller() != pending_owner {
            todo!("Throw a meaningful error")
        }
        self.owner.set(pending_owner);
        self.pending_owner.set(None);
    }
    pub fn add_remote_token_messenger(&mut self, domain: u32, remote_token_messenger: Pubkey) {
        self.require_owner();
        self.remote_token_messengers
            .add_remote_token_messenger(domain, remote_token_messenger);
    }
    pub fn remove_remote_token_messenger(&mut self, domain: u32) {
        self.require_owner();
        self.remote_token_messengers
            .remove_remote_token_messenger(domain);
    }

    pub fn link_token_pair(&self) {}
    pub fn unlink_token_pair(&self) {}
    pub fn pause(&mut self) {
        self.require_owner();
        self.paused.set(true);
    }
    pub fn unpause(&mut self) {
        self.require_owner();
        self.paused.set(false);
    }
    pub fn set_max_burn_amount_per_message(&self) {}
    // Mint get_local_token(burn_token) on the Casper domain
    fn mint(&self, source_domain: u32, burn_token: Pubkey, to: GenericAddress) {
        self.require_not_paused();
        // todo: get local_token from burn_token
        // todo: cross-contract call to mint local_token
    }
    fn burn(&self, burn_token: Address, burn_amount: U256) {
        self.require_not_paused();
        let mut stable_coin_contract = StablecoinContractRef::new(self.env(), burn_token);
        // burn Stablecoin from MessengerMinter allowance
        stable_coin_contract.burn(burn_amount, self.env().caller());
    }
    fn _deposit_for_burn(
        &self,
        burn_amount: u64,
        destination_domain: u32,
        mint_recipient: Pubkey,
        burn_token: GenericAddress,
        destination_caller: Pubkey,
    ) {
        assert_eq!(burn_amount, 0u64);
        assert_eq!(mint_recipient, [0u8; 32]);
        self.burn(
            generic_address_to_contract_address(burn_token),
            U256::from(burn_amount),
        );
        let burn_message: Vec<u8> = BurnMessage::format_message(
            self.version.get().unwrap(),
            &burn_token,
            &mint_recipient,
            burn_amount,
            &generic_address(self.env().caller()),
        );
        let destination_token_messenger: Pubkey = self
            .remote_token_messengers
            .get_remote_token_messenger(destination_domain)
            .unwrap();
        self._send_deposit_for_burn_message(
            destination_domain,
            destination_token_messenger,
            destination_caller,
            &burn_message,
        )
    }

    fn _send_deposit_for_burn_message(
        &self,
        destination_domain: u32,
        destination_token_messenger: Pubkey,
        destination_caller: Pubkey,
        burn_message: &Vec<u8>,
    ) {
        let local_message_transmitter: MessageTransmitterContractRef =
            MessageTransmitterContractRef::new(
                self.env(),
                self.local_message_transmitter
                    .get()
                    .unwrap_or_revert(&self.env()),
            );
        if destination_caller == [0u8; 32] {
            local_message_transmitter.send_message(
                destination_domain,
                destination_token_messenger,
                burn_message,
            );
        } else {
            local_message_transmitter.send_message_with_caller(
                destination_domain,
                destination_token_messenger,
                burn_message,
                destination_caller,
            );
        }
        // todo: emit event
    }

    fn require_not_paused(&self) {
        if self.paused.get().unwrap() == true {
            todo!("Throw a meaningful error")
        }
    }
    fn require_owner(&self) {
        if self.env().caller() != self.owner.get().unwrap() {
            todo!("Throw a meaningful error")
        }
    }
    fn require_local_message_transmitter(&self){
        if self.env().caller() != self.local_message_transmitter.get().unwrap() {
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
