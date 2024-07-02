use odra::casper_types::bytesrepr::FromBytes;
use odra::casper_types::ContractPackageHash;
use odra::casper_types::U256;
use odra::prelude::*;
use odra::Address;
use odra::SubModule;
use odra::Var;
use storage::UsedNonces;

use crate::Pubkey;

pub mod errors;
pub mod events;
pub mod storage;
mod tests;


#[odra::module]
pub struct MessageTransmitter {
    local_domain: Var<u32>,
    version: Var<u32>,
    max_message_body_size: Var<U256>,
    next_available_nonce: Var<u64>,
    used_nonces: SubModule<UsedNonces>,
    attestation_threshold: Var<u32>,
    owner: Var<Address>,
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
        attestation_threshold: u32,
        owner: Address,
    ) {
        self.local_domain.set(local_domain);
        self.version.set(version);
        self.max_message_body_size.set(max_message_body_size);
        self.next_available_nonce.set(next_available_nonce);
        self.attestation_threshold.set(attestation_threshold);
        self.owner.set(owner);
    }
    pub fn send_message(&self) {
        todo!("Format a message and emit an event");
    }
    pub fn send_message_with_caller(&self) {
        todo!("Format a message and emit and event");
    }
    pub fn receive_message(&self, data: &Vec<u8>) {
        // todo: check if paused
        let message: Message = Message{
            data
        };


        //let recipient: Address = Address::Contract(ContractPackageHash::from_bytes().unwrap().0);
        // todo: verify attestation signatures
        // todo: check if the signature threshold is met
        // todo: call token_messenger_minter::handle_receive_message

        // check that the nonce has not been used yet
        // mark the nonce as used
        todo!("Implement");
    }
    pub fn replace_message(&self) {
        todo!("Implement");
    }
    pub fn set_max_message_body_size(&self) {
        todo!("Implement");
    }
    pub fn set_signature_threshold(&self) {
        todo!("Implement");
    }
    pub fn transfer_ownership(&self) {
        todo!("Implement");
    }
    pub fn accept_ownership(&self) {
        todo!("Implement");
    }
    pub fn pause(&self) {
        todo!("Pause the transmitter");
    }
    pub fn unpause(&self) {
        todo!("Unpause the transmitter");
    }
    pub fn is_nonce_used(&self) -> bool {
        todo!("Implement");
    }
    pub fn get_nonce_pda(&self) {
        todo!("Implement");
    }
    pub fn enable_attester(&self) {
        todo!("Implement");
    }
    pub fn disable_attester(&self) {
        todo!("Implement")
    }
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
        &self,
        version: u32,
        local_domain: u32,
        destination_domain: u32,
        nonce: u64,
        sender: &Pubkey,
        // know this is a contract
        recipient: &ContractPackageHash,
        // [0;32] if the destination caller can be any
        // assume this is an account
        destination_caller: &Pubkey,
        message_body: &Vec<u8>,
    ) -> Vec<u8> {
        let mut output: Vec<u8> = vec![0;Self::MESSAGE_BODY_INDEX+message_body.len()];
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
    pub fn hash(&self) {
        todo!("Add keccak hasher for bytes");
    }

    /// Returns version field
    pub fn version(&self) -> u32 {
        self.read_u32(Self::VERSION_INDEX)
    }

    /// Returns sender field
    pub fn sender(&self) -> ContractPackageHash {
        self.read_contract_package_hash(Self::SENDER_INDEX)
    }

    /// Returns recipient field
    pub fn recipient(&self) -> ContractPackageHash {
        self.read_contract_package_hash(Self::RECIPIENT_INDEX)
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
    pub fn destination_caller(&self) -> ContractPackageHash {
        self.read_contract_package_hash(Self::DESTINATION_CALLER_INDEX)
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
    fn read_contract_package_hash(&self, index: usize) -> ContractPackageHash {
        ContractPackageHash::from_bytes(&self.data[index..(index + 32)]).unwrap().0
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
            attestation_threshold: 1,                    // default: 1
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
