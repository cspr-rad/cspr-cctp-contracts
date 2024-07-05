use burn_message::BurnMessage;
use events::DepositForBurn;
use events::MintAndWithdraw;
use events::RemoteTokenMessengerAdded;
use events::RemoteTokenMessengerRemoved;
use events::TokenPairLinked;
use events::TokenPairUnlinked;
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
use crate::message_transmitter::Message;
use crate::GenericAddress;
use crate::Pubkey;

mod burn_message;
pub mod errors;
pub mod events;
pub mod storage;

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
    linked_token_pairs: Mapping<(u32, Pubkey), Option<Address>>,
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
        burn_token: Address,
    ) {
        let destination_caller: Pubkey = [0u8; 32];
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

    pub fn replace_deposit_for_burn(
        &self,
        original_message: &Vec<u8>,
        original_attestation: &Vec<u8>,
        new_destination_caller: Pubkey,
        new_mint_recipient: Pubkey,
    ) {
        // todo: validate message format
        // todo: validate message body format
        let original_msg: Message = Message {
            data: &original_message,
        };
        let original_burn_msg: BurnMessage = BurnMessage {
            data: original_msg.message_body(),
        };
        let burn_token: [u8; 32] = original_burn_msg.burn_token();
        let amount: u64 = original_burn_msg.amount();
        let sender: [u8; 32] = original_burn_msg.message_sender();
        let version: u32 = original_burn_msg.version();
        let new_burn_message_body: Vec<u8> =
            BurnMessage::format_message(version, &burn_token, &new_mint_recipient, amount, &sender);
        let local_message_transmitter: MessageTransmitterContractRef =
            MessageTransmitterContractRef::new(
                self.env(),
                self.local_message_transmitter.get().unwrap(),
            );
        local_message_transmitter.replace_message(
            original_message,
            original_attestation,
            &new_burn_message_body,
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
        sender: Pubkey,
        message_body: &Vec<u8>,
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
        let burn_message: BurnMessage = BurnMessage {
            data: &message_body,
        };
        assert_eq!(self.version.get().unwrap(), burn_message.version());
        let mint_recipient: GenericAddress = burn_message.mint_recipient();
        let burn_token: Pubkey = burn_message.burn_token();
        let amount: u64 = burn_message.amount();
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
            todo!("Throw a meaningful error")
        }
        self.owner.set(pending_owner);
        self.pending_owner.set(None);
    }
    pub fn add_remote_token_messenger(&mut self, domain: u32, remote_token_messenger: Pubkey) {
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
        let token_messenger: Pubkey = self
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

    pub fn link_token_pair(&mut self, local_token: Address, remote_token: Pubkey, domain: u32) {
        self.require_owner();
        self.linked_token_pairs
            .set(&(domain, remote_token), Some(local_token));
        self.env().emit_event(TokenPairLinked {
            local_token: generic_address(local_token),
            remote_token,
            domain,
        });
    }
    pub fn unlink_token_pair(&mut self, remote_token: Pubkey, domain: u32) {
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
    pub fn set_max_burn_amount_per_message(&self) {}
    // Mint get_local_token(burn_token) on the Casper domain
    fn mint(
        &self,
        source_domain: u32,
        burn_token: Pubkey,
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
        // todo: emit event
        local_token
    }
    fn burn(&self, burn_token: Address, burn_amount: U256) {
        self.require_not_paused();
        let mut stable_coin_contract: StablecoinContractRef =
            StablecoinContractRef::new(self.env(), burn_token);
        stable_coin_contract.burn_cctp(burn_amount, self.env().caller());
    }
    fn _deposit_for_burn(
        &self,
        burn_amount: u64,
        destination_domain: u32,
        mint_recipient: Pubkey,
        burn_token: GenericAddress,
        destination_caller: Pubkey,
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
        let destination_token_messenger: Pubkey = self
            .remote_token_messengers
            .get_remote_token_messenger(destination_domain)
            .unwrap();
        self._send_deposit_for_burn_message(
            destination_domain,
            destination_token_messenger,
            destination_caller,
            &burn_message,
        );
        self.env().emit_event(DepositForBurn {
            // todo: adjust nonce logic to get next available nonce from transmitter
            nonce: 0u64,
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
    fn require_local_message_transmitter(&self) {
        if self.env().caller() != self.local_message_transmitter.get().unwrap() {
            todo!("Throw a meaningful error")
        }
    }
}
