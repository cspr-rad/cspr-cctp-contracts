use crate::{GenericAddress, Pubkey};
extern crate alloc;
use alloc::{vec, vec::Vec};
use sha3::{Digest, Keccak256};
pub struct Message<'a> {
    pub data: &'a [u8],
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
        assert!(message_bytes.len() >= Self::MESSAGE_BODY_INDEX);
        let message = Self {
            data: message_bytes,
        };
        assert_eq!(message.version(), expected_version);
        message
    }
    #[allow(clippy::too_many_arguments)]
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
        let mut hasher = Keccak256::new();
        hasher.update(self.data);
        hasher.finalize().as_slice().try_into().unwrap()
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
