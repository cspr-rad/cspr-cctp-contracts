use burn_message::BurnMessage;
use events::DepositForBurn;
use events::MintAndWithdraw;
use events::RemoteTokenMessengerAdded;
use events::RemoteTokenMessengerRemoved;
use events::TokenPairLinked;
use events::TokenPairUnlinked;
use odra::casper_types::bytesrepr::Bytes;
use odra::casper_types::U256;
use odra::prelude::*;
use odra::Address;
use odra::Mapping;
use odra::SubModule;
use odra::UnwrapOrRevert;
use odra::Var;

use crate::generic_address;
use crate::generic_address_to_account_address;
use crate::generic_address_to_contract_address;
use crate::message_transmitter::message::Message;
use crate::GenericAddress;

pub mod burn_message;
pub mod errors;
pub mod events;
pub mod storage;

use crate::message_transmitter::MessageTransmitterContractRef;
use crate::stablecoin::StablecoinContractRef;
use errors::Error;
use storage::RemoteTokenMessengers;

#[odra::module]
pub struct TokenMessengerMinter {
    version: Var<u32>,
    paused: Var<bool>,
    local_message_transmitter: Var<Address>,
    remote_token_messengers: SubModule<RemoteTokenMessengers>,
    max_burn_amount_per_message: Var<U256>,
    owner: Var<Address>,
    pending_owner: Var<Option<Address>>,
    linked_token_pairs: Mapping<(u32, GenericAddress), Option<Address>>,
}

#[odra::module]
impl TokenMessengerMinter {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        version: u32,
        local_message_transmitter: Address,
        max_burn_amount_per_message: U256,
        owner: Address,
    ) {
        self.version.set(version);
        self.paused.set(false);
        self.local_message_transmitter
            .set(local_message_transmitter);
        self.max_burn_amount_per_message
            .set(max_burn_amount_per_message);
        self.owner.set(owner);
        self.pending_owner.set(None);
    }

    pub fn deposit_for_burn(
        &self,
        amount: u64,
        destination_domain: u32,
        mint_recipient: GenericAddress,
        burn_token: Address,
    ) {
        let destination_caller: GenericAddress = [0u8; 32];
        self._deposit_for_burn(
            amount,
            destination_domain,
            mint_recipient,
            generic_address(burn_token),
            destination_caller,
        );
    }

    pub fn deposit_for_burn_with_caller(
        &self,
        amount: u64,
        destination_domain: u32,
        mint_recipient: GenericAddress,
        burn_token: GenericAddress,
        destination_caller: GenericAddress,
    ) {
        self._deposit_for_burn(
            amount,
            destination_domain,
            mint_recipient,
            burn_token,
            destination_caller,
        );
    }

    pub fn replace_deposit_for_burn(
        &self,
        original_message: Bytes,
        original_attestation: Bytes,
        new_destination_caller: GenericAddress,
        new_mint_recipient: GenericAddress,
    ) {
        let original_msg: Message = Message::new(self.version.get().unwrap(), &original_message);
        let original_burn_msg: BurnMessage =
            BurnMessage::new(self.version.get().unwrap(), original_msg.message_body());
        let burn_token: [u8; 32] = original_burn_msg.burn_token();
        let amount: u64 = original_burn_msg.amount();
        let sender: [u8; 32] = original_burn_msg.message_sender();
        assert_eq!(generic_address(self.env().caller()), sender);
        assert_ne!(new_mint_recipient, [0u8; 32]);
        let version: u32 = original_burn_msg.version();
        let new_burn_message_body: Vec<u8> =
            BurnMessage::format_message(version, &burn_token, &new_mint_recipient, amount, &sender);
        let local_message_transmitter: MessageTransmitterContractRef =
            MessageTransmitterContractRef::new(
                self.env(),
                self.local_message_transmitter.get().unwrap(),
            );
        local_message_transmitter.replace_message(
            original_message.clone(),
            original_attestation.clone(),
            Bytes::from(new_burn_message_body),
            new_destination_caller,
        );
        self.env().emit_event(DepositForBurn {
            nonce: original_msg.nonce(),
            burn_token,
            amount: U256::from(amount),
            depositor: generic_address(self.env().caller()),
            mint_recipient: new_mint_recipient,
            destination_domain: original_msg.destination_domain(),
            destination_caller: new_destination_caller,
            destination_token_messenger: original_msg.recipient(),
        })
    }

    pub fn handle_receive_message(
        &self,
        remote_domain: u32,
        sender: GenericAddress,
        message_body: Bytes,
    ) {
        self.require_local_message_transmitter();
        // remote sender must be remote token messenger
        assert_eq!(
            self.remote_token_messengers
                .get_remote_token_messenger(remote_domain)
                .unwrap(),
            sender
        );
        // todo: validate burn message format
        let burn_message: BurnMessage =
            BurnMessage::new(self.version.get().unwrap(), &message_body);
        assert_eq!(self.version.get().unwrap(), burn_message.version());
        let mint_recipient: GenericAddress = burn_message.mint_recipient();
        let burn_token: GenericAddress = burn_message.burn_token();
        let amount: u64 = burn_message.amount();
        assert!(U256::from(amount) <= self.max_burn_amount_per_message.get().unwrap());
        let mint_token = self.mint(remote_domain, burn_token, mint_recipient, amount);
        self.env().emit_event(MintAndWithdraw {
            mint_recipient,
            amount: U256::from(amount),
            mint_token: generic_address(mint_token),
        });
    }
    pub fn transfer_ownership(&mut self, new_pending_owner: Address) {
        self.require_owner();
        self.pending_owner.set(Some(new_pending_owner));
    }
    pub fn accept_ownership(&mut self) {
        let pending_owner = self.pending_owner.get().unwrap().unwrap();
        if self.env().caller() != pending_owner {
            self.env().revert(Error::InsufficientRights)
        }
        self.owner.set(pending_owner);
        self.pending_owner.set(None);
    }
    pub fn add_remote_token_messenger(
        &mut self,
        domain: u32,
        remote_token_messenger: GenericAddress,
    ) {
        self.require_owner();
        self.remote_token_messengers
            .add_remote_token_messenger(domain, remote_token_messenger);
        self.env().emit_event(RemoteTokenMessengerAdded {
            domain,
            token_messenger: remote_token_messenger,
        });
    }
    pub fn remove_remote_token_messenger(&mut self, domain: u32) {
        self.require_owner();
        let token_messenger: GenericAddress = self
            .remote_token_messengers
            .get_remote_token_messenger(domain)
            .unwrap();
        self.remote_token_messengers
            .remove_remote_token_messenger(domain);
        self.env().emit_event(RemoteTokenMessengerRemoved {
            domain,
            token_messenger,
        });
    }

    pub fn link_token_pair(
        &mut self,
        local_token: Address,
        remote_token: GenericAddress,
        domain: u32,
    ) {
        self.require_owner();
        self.linked_token_pairs
            .set(&(domain, remote_token), Some(local_token));
        self.env().emit_event(TokenPairLinked {
            local_token: generic_address(local_token),
            remote_token,
            domain,
        });
    }
    pub fn unlink_token_pair(&mut self, remote_token: GenericAddress, domain: u32) {
        self.require_owner();
        let local_token: Address = self
            .linked_token_pairs
            .get(&(domain, remote_token))
            .unwrap()
            .unwrap();
        self.linked_token_pairs.set(&(domain, remote_token), None);
        self.env().emit_event(TokenPairUnlinked {
            local_token: generic_address(local_token),
            remote_token,
            domain,
        });
    }
    pub fn pause(&mut self) {
        self.require_owner();
        self.paused.set(true);
    }
    pub fn unpause(&mut self) {
        self.require_owner();
        self.paused.set(false);
    }
    pub fn set_max_burn_amount_per_message(&mut self, amount: U256) {
        self.require_owner();
        self.max_burn_amount_per_message.set(amount);
    }
    // Mint get_local_token(burn_token) on the Casper domain
    fn mint(
        &self,
        source_domain: u32,
        burn_token: GenericAddress,
        to: GenericAddress,
        amount: u64,
    ) -> Address {
        self.require_not_paused();
        let local_token: Address = self
            .linked_token_pairs
            .get(&(source_domain, burn_token))
            .unwrap_or_revert(&self.env())
            .unwrap();
        let mut stable_coin_contract: StablecoinContractRef =
            StablecoinContractRef::new(self.env(), local_token);
        // This will work for both Address::Account and Address::ContractHash, since the first byte is dropped by the accounting
        // logic of the stablecoin.
        stable_coin_contract.mint(&generic_address_to_account_address(to), U256::from(amount));
        local_token
    }
    fn burn(&self, burn_token: Address, burn_amount: U256) {
        self.require_not_paused();
        let mut stable_coin_contract: StablecoinContractRef =
            StablecoinContractRef::new(self.env(), burn_token);
        stable_coin_contract.burn(burn_amount, self.env().caller());
    }
    fn _deposit_for_burn(
        &self,
        burn_amount: u64,
        destination_domain: u32,
        mint_recipient: GenericAddress,
        burn_token: GenericAddress,
        destination_caller: GenericAddress,
    ) {
        assert_ne!(burn_amount, 0u64);
        assert_ne!(mint_recipient, [0u8; 32]);
        let token_contract_address: Address = generic_address_to_contract_address(burn_token);
        self.burn(token_contract_address, U256::from(burn_amount));
        let burn_message: Vec<u8> = BurnMessage::format_message(
            self.version.get().unwrap(),
            &burn_token,
            &mint_recipient,
            burn_amount,
            &generic_address(self.env().caller()),
        );
        let destination_token_messenger: GenericAddress = self
            .remote_token_messengers
            .get_remote_token_messenger(destination_domain)
            .unwrap();
        let nonce = self._send_deposit_for_burn_message(
            destination_domain,
            destination_token_messenger,
            destination_caller,
            Bytes::from(burn_message.clone()),
        );
        self.env().emit_event(DepositForBurn {
            nonce,
            burn_token,
            amount: U256::from(burn_amount),
            depositor: generic_address(self.env().caller()),
            mint_recipient,
            destination_domain,
            destination_token_messenger,
            destination_caller,
        })
    }

    fn _send_deposit_for_burn_message(
        &self,
        destination_domain: u32,
        destination_token_messenger: GenericAddress,
        destination_caller: GenericAddress,
        burn_message: Bytes,
    ) -> u64 {
        let mut local_message_transmitter: MessageTransmitterContractRef =
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
                burn_message.clone(),
            )
        } else {
            local_message_transmitter.send_message_with_caller(
                destination_domain,
                destination_token_messenger,
                burn_message.clone(),
                destination_caller,
            )
        }
    }

    fn require_not_paused(&self) {
        if self.paused.get().unwrap() {
            self.env().revert(Error::ContractIsPaused)
        }
    }
    fn require_owner(&self) {
        if self.env().caller() != self.owner.get().unwrap() {
            self.env().revert(Error::InsufficientRights)
        }
    }
    fn require_local_message_transmitter(&self) {
        if self.env().caller() != self.local_message_transmitter.get().unwrap() {
            self.env().revert(Error::InsufficientRights)
        }
    }
}
