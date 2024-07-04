use odra::casper_types::bytesrepr::ToBytes;
use odra::casper_types::U256;
use odra::prelude::*;
use odra::Address;
use odra::SubModule;
use odra::Var;
use storage::Attesters;
use storage::UsedNonces;
use tiny_keccak::Hasher;

use crate::generic_address;
use crate::generic_address_to_contract_address;
use crate::GenericAddress;
use crate::Pubkey;

pub mod errors;
pub mod events;
pub mod storage;
mod tests;

use crate::token_messenger_minter::TokenMessengerMinterContractRef;
use tiny_keccak::Keccak;

#[odra::module]
pub struct MessageTransmitter {
    local_domain: Var<u32>,
    version: Var<u32>,
    paused: Var<bool>,
    max_message_body_size: Var<U256>,
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
        self.next_available_nonce.set(next_available_nonce);
        self.signature_threshold.set(signature_threshold);
        self.owner.set(owner);
        self.pending_owner.set(None);
    }
    pub fn send_message(&self, destination_domain: u32, recipient: Pubkey, message_body: &Vec<u8>) {
        self.require_not_paused();
        let empty_destination_caller: [u8; 32] = [0u8; 32];
        let nonce: u64 = self.next_available_nonce.get().unwrap();
        let message_sender: GenericAddress = generic_address(self.env().caller());
        self._send_message(
            self.version.get().unwrap(),
            self.local_domain.get().unwrap(),
            destination_domain,
            recipient,
            empty_destination_caller,
            message_sender,
            nonce,
            message_body,
        );
    }
    pub fn send_message_with_caller(
        &self,
        destination_domain: u32,
        recipient: Pubkey,
        message_body: &Vec<u8>,
        destination_caller: Pubkey,
    ) {
        self.require_not_paused();
        let nonce: u64 = self.next_available_nonce.get().unwrap();
        let message_sender: GenericAddress = generic_address(self.env().caller());
        self._send_message(
            self.version.get().unwrap(),
            self.local_domain.get().unwrap(),
            destination_domain,
            recipient,
            destination_caller,
            message_sender,
            nonce,
            message_body,
        );
    }
    pub fn receive_message(&mut self, data: &Vec<u8>, attestations: &Vec<u8>) {
        self.require_not_paused();
        let message: Message = Message { data };
        assert_eq!(message.version(), self.version.get().unwrap());
        let token_messenger_minter_contract: TokenMessengerMinterContractRef =
            TokenMessengerMinterContractRef::new(
                self.env(),
                generic_address_to_contract_address(message.recipient()),
            );
        let nonce: u64 = message.nonce();
        let sender: [u8; 32] = message.sender();
        let hashed_nonce: [u8; 32] = hash_nonce(nonce, sender);
        assert_eq!(self.used_nonces.is_used_nonce(nonce, hashed_nonce), false);
        self.used_nonces.use_nonce(nonce, hash_nonce(nonce, sender));

        token_messenger_minter_contract.handle_receive_message(
            message.source_domain(),
            message.sender(),
            &message.message_body().to_vec(),
        );
        // emit a message received event
    }
    pub fn replace_message(&self) {
        self.require_not_paused();
        todo!("Implement");
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
            todo!("Throw a meaningful error")
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
    // the purpose of this function is currently unclear
    pub fn get_nonce_pda(&self) {
        todo!("Implement");
    }
    pub fn is_used_nonce(&self, nonce: u64, account: GenericAddress) -> bool {
        let nonce_hashed = hash_nonce(nonce, account);
        self.used_nonces.is_used_nonce(nonce, nonce_hashed)
    }
    pub fn enable_attester(&mut self, new_attester: Pubkey) {
        self.require_owner();
        self.attesters.enable_attester(new_attester);
    }
    pub fn disable_attester(&mut self, attester: Pubkey) {
        self.require_owner();
        self.attesters.disable_attester(attester);
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
    fn _send_message(
        &self,
        version: u32,
        local_domain: u32,
        destination_domain: u32,
        recipient: Pubkey,
        destination_caller: Pubkey,
        sender: GenericAddress,
        nonce: u64,
        message_body: &Vec<u8>,
    ) {
        assert_ne!(recipient, [0u8; 32]);
        // Validate message body length
        assert!(U256::from(message_body.len()) <= self.max_message_body_size.get().unwrap());
        let message: Message = Message {
            data: &Message::format_message(
                version,
                local_domain,
                destination_domain,
                nonce,
                &sender,
                &recipient,
                &destination_caller,
                message_body,
            ),
        };
        // Todo: Emit the constructed Message as an Event
    }
}
fn hash_nonce(nonce: u64, account: GenericAddress) -> [u8; 32] {
    let mut hasher = Keccak::v384();
    let mut output = [0u8; 32];
    hasher.update(&nonce.to_bytes().unwrap());
    hasher.update(&account);
    hasher.finalize(&mut output);
    output
}

pub struct Message<'a> {
    data: &'a [u8],
}

impl<'a> Message<'a> {
    const VERSION_INDEX: usize = 0;
    const SOURCE_DOMAIN_INDEX: usize = 4;
    const DESTINATION_DOMAIN_INDEX: usize = 8;
    const NONCE_INDEX: usize = 12;
    const SENDER_INDEX: usize = 20;
    const RECIPIENT_INDEX: usize = 52;
    const DESTINATION_CALLER_INDEX: usize = 84;
    const MESSAGE_BODY_INDEX: usize = 116;

    pub fn new(expected_version: u32, message_bytes: &'a [u8]) -> Self {
        //todo: check message bytes size is >= MESSAGE_BODY_INDEX
        //todo: check message version against local
        Message {
            data: &message_bytes,
        }
    }

    pub fn format_message(
        version: u32,
        local_domain: u32,
        destination_domain: u32,
        nonce: u64,
        sender: &Pubkey,
        // know this is a contract
        recipient: &GenericAddress,
        // [0;32] if the destination caller can be any
        // assume this is an account
        destination_caller: &Pubkey,
        message_body: &Vec<u8>,
    ) -> Vec<u8> {
        let mut output: Vec<u8> = vec![0; Self::MESSAGE_BODY_INDEX + message_body.len()];
        output[Self::VERSION_INDEX..Self::SOURCE_DOMAIN_INDEX]
            .copy_from_slice(&version.to_be_bytes());
        output[Self::SOURCE_DOMAIN_INDEX..Self::DESTINATION_DOMAIN_INDEX]
            .copy_from_slice(&local_domain.to_be_bytes());
        output[Self::DESTINATION_DOMAIN_INDEX..Self::NONCE_INDEX]
            .copy_from_slice(&destination_domain.to_be_bytes());
        output[Self::NONCE_INDEX..Self::SENDER_INDEX].copy_from_slice(&nonce.to_be_bytes());
        output[Self::SENDER_INDEX..Self::RECIPIENT_INDEX].copy_from_slice(sender.as_ref());
        output[Self::RECIPIENT_INDEX..Self::DESTINATION_CALLER_INDEX]
            .copy_from_slice(recipient.as_ref());
        output[Self::DESTINATION_CALLER_INDEX..Self::MESSAGE_BODY_INDEX]
            .copy_from_slice(destination_caller.as_ref());
        if !message_body.is_empty() {
            output[Self::MESSAGE_BODY_INDEX..].copy_from_slice(message_body.as_slice());
        }
        output
    }

    /// Returns Keccak hash of the message
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Keccak::v384();
        let mut output = [0u8; 32];
        hasher.update(&self.data);
        hasher.finalize(&mut output);
        output
    }

    /// Returns version field
    pub fn version(&self) -> u32 {
        self.read_u32(Self::VERSION_INDEX)
    }

    /// Returns sender field
    pub fn sender(&self) -> GenericAddress {
        self.read_generic_address(Self::SENDER_INDEX)
    }

    /// Returns recipient field
    pub fn recipient(&self) -> GenericAddress {
        self.read_generic_address(Self::RECIPIENT_INDEX)
    }

    /// Returns source_domain field
    pub fn source_domain(&self) -> u32 {
        self.read_u32(Self::SOURCE_DOMAIN_INDEX)
    }

    /// Returns destination_domain field
    pub fn destination_domain(&self) -> u32 {
        self.read_u32(Self::DESTINATION_DOMAIN_INDEX)
    }

    /// Returns destination_caller field
    pub fn destination_caller(&self) -> GenericAddress {
        self.read_generic_address(Self::DESTINATION_CALLER_INDEX)
    }

    /// Returns nonce field
    pub fn nonce(&self) -> u64 {
        self.read_u64(Self::NONCE_INDEX)
    }

    /// Returns message_body field
    pub fn message_body(&self) -> &[u8] {
        &self.data[Self::MESSAGE_BODY_INDEX..]
    }

    fn read_u32(&self, index: usize) -> u32 {
        u32::from_be_bytes(
            // u32 size is 32 bits = 4 bytes
            self.data[index..(index + 4)].try_into().unwrap(),
        )
    }

    fn read_u64(&self, index: usize) -> u64 {
        u64::from_be_bytes(
            // 64 size is 64 bits = 8 bytes
            self.data[index..(index + 8)].try_into().unwrap(),
        )
    }

    /// Reads pubkey field at the given offset
    fn read_generic_address(&self, index: usize) -> GenericAddress {
        self.data[index..(index + 32)].try_into().unwrap()
    }
}

#[cfg(test)]
pub(crate) mod setup_tests {
    use crate::message_transmitter::{MessageTransmitterHostRef, MessageTransmitterInitArgs};
    use odra::host::{Deployer, HostEnv};

    pub fn setup() -> (HostEnv, MessageTransmitterHostRef) {
        let env = odra_test::env();
        let args = MessageTransmitterInitArgs {
            local_domain: 31u32, // 31: CA
            version: 1u32,
            max_message_body_size: 1_000_000_000.into(), // unreasonably high for development
            next_available_nonce: 1,                     // start from nonce = 1
            signature_threshold: 1,                      // default: 1
            owner: env.get_account(0),                   // default account as owner
        };
        let message_transmitter = setup_with_args(&env, args);
        (env, message_transmitter)
    }

    pub fn setup_with_args(
        env: &HostEnv,
        args: MessageTransmitterInitArgs,
    ) -> MessageTransmitterHostRef {
        MessageTransmitterHostRef::deploy(env, args)
    }
}
