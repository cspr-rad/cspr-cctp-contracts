use events::{MessageReceived, MessageSent};
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use odra::{
    casper_types::{
        bytesrepr::{Bytes, ToBytes},
        U256,
    },
    prelude::*,
    Address, SubModule, Var,
};
use sha3::{Digest, Keccak256};
use storage::{Attesters, UsedNonces};

use crate::generic_address_to_contract_address;
use crate::GenericAddress;
use crate::Pubkey;
use crate::{generic_address, EthAddress};

pub mod errors;
pub mod events;
pub mod message;
pub mod storage;
use message::Message;

use crate::token_messenger_minter::TokenMessengerMinterContractRef;
use errors::Error;

#[odra::module]
pub struct MessageTransmitter {
    local_domain: Var<u32>,
    version: Var<u32>,
    paused: Var<bool>,
    max_message_body_size: Var<U256>,
    // mapping of source domain : nonce
    next_available_nonce: Var<u64>,
    used_nonces: SubModule<UsedNonces>,
    attesters: SubModule<Attesters>,
    signature_threshold: Var<u32>,
    owner: Var<Address>,
    pending_owner: Var<Option<Address>>,
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
        signature_threshold: u32,
        owner: Address,
    ) {
        self.local_domain.set(local_domain);
        self.version.set(version);
        self.paused.set(false);
        self.max_message_body_size.set(max_message_body_size);
        self.signature_threshold.set(signature_threshold);
        self.next_available_nonce.set(next_available_nonce);
        self.owner.set(owner);
        self.pending_owner.set(None);
    }
    pub fn send_message(
        &mut self,
        destination_domain: u32,
        recipient: Pubkey,
        message_body: Bytes,
    ) -> u64 {
        self.require_not_paused();
        let empty_destination_caller: [u8; 32] = [0u8; 32];
        let nonce: u64 = self.next_available_nonce.get().unwrap();
        self.next_available_nonce
            .set(self.next_available_nonce.get().unwrap() + 1);
        let message_sender: GenericAddress = generic_address(self.env().caller());
        self._send_message(
            destination_domain,
            recipient,
            empty_destination_caller,
            message_sender,
            nonce,
            message_body,
        );
        nonce
    }
    pub fn send_message_with_caller(
        &mut self,
        destination_domain: u32,
        recipient: Pubkey,
        message_body: Bytes,
        destination_caller: Pubkey,
    ) -> u64 {
        self.require_not_paused();
        let nonce: u64 = self.next_available_nonce.get().unwrap();
        self.next_available_nonce
            .set(self.next_available_nonce.get().unwrap() + 1);
        let message_sender: GenericAddress = generic_address(self.env().caller());
        self._send_message(
            destination_domain,
            recipient,
            destination_caller,
            message_sender,
            nonce,
            message_body,
        );
        nonce
    }
    pub fn replace_message(
        &self,
        original_message: Bytes,
        original_attestation: Bytes,
        new_message_body: Bytes,
        new_destination_caller: Pubkey,
    ) {
        let original_msg: Message = Message::new(self.version.get().unwrap(), &original_message);
        // verify original attestation
        for chunk in original_attestation.to_vec().chunks(64) {
            let pubkey_recovered: EthAddress =
                recover_attester(original_msg.hash(), chunk.try_into().unwrap());
            assert!(self.attesters.is_attester(pubkey_recovered));
        }
        let sender = original_msg.sender();
        // Message must be replaced by the MessengerMinter that submitted the original message.
        assert_eq!(generic_address(self.env().caller()), sender);
        let destination_domain: u32 = original_msg.destination_domain();
        let recipient = original_msg.recipient();
        let nonce = original_msg.nonce();

        self._send_message(
            destination_domain,
            recipient,
            new_destination_caller,
            sender,
            nonce,
            new_message_body,
        );
    }
    pub fn receive_message(&mut self, data: Bytes, attestation: Bytes) {
        self.require_not_paused();
        let message: Message = Message::new(self.version.get().unwrap(), &data);
        let mut valid_attestations = 0;
        // todo: check ascending order
        for chunk in attestation.to_vec().chunks(64) {
            let pubkey_recovered: EthAddress =
                recover_attester(message.hash(), chunk.try_into().unwrap());
            assert!(self.attesters.is_attester(pubkey_recovered));
            valid_attestations += 1;
        }
        assert!(valid_attestations >= self.signature_threshold.get().unwrap());
        assert_eq!(message.version(), self.version.get().unwrap());
        let destination_caller: [u8; 32] = message.destination_caller();
        if destination_caller != [0u8; 32]
            && destination_caller != generic_address(self.env().self_address())
        {
            self.env().revert(Error::InvalidMessageRecipient)
        }
        let token_messenger_minter_contract: TokenMessengerMinterContractRef =
            TokenMessengerMinterContractRef::new(
                self.env(),
                generic_address_to_contract_address(message.recipient()),
            );
        let nonce: u64 = message.nonce();
        let sender: [u8; 32] = message.sender();
        let hashed_nonce: [u8; 32] = hash_nonce(nonce, sender);
        let source_domain: u32 = message.source_domain();
        let sender: [u8; 32] = message.sender();
        let message_body: &[u8] = message.message_body();

        assert!(!self.used_nonces.is_used_nonce(hashed_nonce));
        self.used_nonces.use_nonce(hashed_nonce);

        token_messenger_minter_contract.handle_receive_message(
            source_domain,
            sender,
            Bytes::from(message_body.to_vec()),
        );

        self.env().emit_event(MessageReceived {
            caller: generic_address(self.env().caller()),
            source_domain,
            nonce,
            sender,
            message_body: message_body.to_vec(),
        })
    }
    pub fn set_max_message_body_size(&mut self, new_max_message_body_size: U256) {
        self.require_owner();
        self.max_message_body_size.set(new_max_message_body_size);
    }
    pub fn set_signature_threshold(&mut self, new_signature_threshold: u32) {
        self.require_owner();
        self.signature_threshold.set(new_signature_threshold);
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
    pub fn pause(&mut self) {
        self.require_owner();
        self.paused.set(true);
    }
    pub fn unpause(&mut self) {
        self.require_owner();
        self.paused.set(false);
    }
    pub fn is_used_nonce(&self, nonce: u64, account: GenericAddress) -> bool {
        let nonce_hashed = hash_nonce(nonce, account);
        self.used_nonces.is_used_nonce(nonce_hashed)
    }
    pub fn enable_attester(&mut self, new_attester: EthAddress) {
        self.require_owner();
        self.attesters.enable_attester(new_attester);
    }
    pub fn disable_attester(&mut self, attester: EthAddress) {
        self.require_owner();
        self.attesters.disable_attester(attester);
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
    fn _send_message(
        &self,
        destination_domain: u32,
        recipient: Pubkey,
        destination_caller: Pubkey,
        sender: GenericAddress,
        nonce: u64,
        message_body: Bytes,
    ) {
        assert_ne!(recipient, [0u8; 32]);
        // Validate message body length
        assert!(U256::from(message_body.len()) <= self.max_message_body_size.get().unwrap());
        let message_body: &Vec<u8> = &Message::format_message(
            self.version.get().unwrap(),
            self.local_domain.get().unwrap(),
            destination_domain,
            nonce,
            &sender,
            &recipient,
            &destination_caller,
            &message_body.to_vec(),
        );
        let message: Message = Message::new(self.version.get().unwrap(), message_body);
        self.env().emit_event(MessageSent {
            message: message.data.to_vec(),
        });
    }
}
fn hash_nonce(nonce: u64, account: GenericAddress) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(&nonce.to_bytes().unwrap());
    hasher.update(&account);
    hasher.finalize().as_slice().try_into().unwrap()
}

fn recover_attester(message_hash: [u8; 32], signature: [u8; 64]) -> EthAddress {
    let recid: RecoveryId = RecoveryId::try_from(1u8).unwrap();
    let mut hasher = Keccak256::new();
    hasher.update(message_hash);
    let recovered_key = VerifyingKey::recover_from_digest(
        hasher,
        &Signature::from_bytes(&signature.into()).unwrap(),
        recid,
    )
    .unwrap();
    recover_ethereum_address(
        recovered_key.to_encoded_point(false).as_ref()[1..]
            .try_into()
            .expect("Failed to fit pubkey into slice"),
    )
}
fn recover_ethereum_address(pubkey: [u8; 64]) -> EthAddress {
    let mut hasher = Keccak256::new();
    hasher.update(pubkey);
    let hash = hasher.finalize();
    hash.as_slice()[12..]
        .try_into()
        .expect("Failed to fit pubkey into slice")
}

#[test]
fn test_pubkey_recovery() {
    use k256::ecdsa::signature::SignerMut;
    let message_hash = [0u8; 32];
    let mut sk =
        k256::ecdsa::SigningKey::from_slice(&[1; 32]).expect("Failed to construct SigningKey");
    let signature: Signature = sk.sign(&message_hash);
    recover_attester(message_hash, signature.to_bytes().into());
}
