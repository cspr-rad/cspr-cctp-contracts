use core::fmt::Display;
use core::fmt::Error;
use num_traits::FromBytes;
use odra::casper_types::U256;
use odra::prelude::*;
use odra::Address;
use odra::SubModule;
use odra::Var;
use storage::UsedNonces;

use crate::Hash;
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
        owner: Address,
    ) {
        self.local_domain.set(local_domain);
        self.version.set(version);
        self.max_message_body_size.set(max_message_body_size);
        self.next_available_nonce.set(next_available_nonce);
        self.owner.set(owner);
    }
    pub fn send_message(&self) {
        todo!("Format a message and emit an event");
    }
    pub fn send_message_with_caller(&self) {
        todo!("Format a message and emit and event");
    }
    pub fn receive_message(&self) {
        // todo: verify attestation signatures
        // todo: check if the signature threshold is met
        // todo: call token_messenger_minter handleReceiveMessage


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
}

pub struct Message<'a>{
    data: &'a[u8]
}

impl<'a> Message<'a>{
    const VERSION_INDEX: usize = 0;
    const SOURCE_DOMAIN_INDEX: usize = 4;
    const DESTINATION_DOMAIN_INDEX: usize = 8;
    const NONCE_INDEX: usize = 12;
    const SENDER_INDEX: usize = 20;
    const RECIPIENT_INDEX: usize = 52;
    const DESTINATION_CALLER_INDEX: usize = 84;
    const MESSAGE_BODY_INDEX: usize = 116;

    pub fn new(message_bytes: &'a [u8]) ->Self {
        Message{
            data: &message_bytes
        }
    }

    fn format_message(
        &self,
        version: u32,
        local_domain: u32,
        destination_domain: u32,
        nonce: u64,
        sender: &Pubkey,
        recipient: &Pubkey,
        // [0;32] if the destination caller can be any
        destination_caller: &Pubkey,
        message_body: &Vec<u8>,
    ) -> Vec<u8>{
        let mut output: Vec<u8> = Vec::new();
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
        self.read_u32::<u32>(Self::VERSION_INDEX).unwrap()
    }

    /// Returns sender field
    pub fn sender(&self) -> Pubkey {
        self.read_pubkey(Self::SENDER_INDEX).unwrap()
    }

    /// Returns recipient field
    pub fn recipient(&self) -> Pubkey {
        self.read_pubkey(Self::RECIPIENT_INDEX).unwrap()
    }

    /// Returns source_domain field
    pub fn source_domain(&self) -> u32 {
        self.read_u32::<u32>(Self::SOURCE_DOMAIN_INDEX).unwrap()
    }

    /// Returns destination_domain field
    pub fn destination_domain(&self) -> u32 {
        self.read_u32::<u32>(Self::DESTINATION_DOMAIN_INDEX).unwrap()
    }

    /// Returns destination_caller field
    pub fn destination_caller(&self) -> Pubkey {
        self.read_pubkey(Self::DESTINATION_CALLER_INDEX).unwrap()
    }

    /// Returns nonce field
    pub fn nonce(&self) -> u64 {
        self.read_u64::<u64>(Self::NONCE_INDEX).unwrap()
    }

    /// Returns message_body field
    pub fn message_body(&self) -> &[u8] {
        &self.data[Self::MESSAGE_BODY_INDEX..]
    }

    fn read_u32<T>(&self, index: usize) -> Result<u32, Error>
    where
        T: num_traits::PrimInt + FromBytes + Display,
        &'a <T as FromBytes>::Bytes: TryFrom<&'a [u8]> + 'a,
    {
        Ok(u32::from_be_bytes(
            // u32 size is 32 bytes
            self.data[index..(index + 32)]
                .try_into()
                .map_err(|_| Error)?
        ))
    }

    fn read_u64<T>(&self, index: usize) -> Result<u64, Error>
    where
        T: num_traits::PrimInt + FromBytes + Display,
        &'a <T as FromBytes>::Bytes: TryFrom<&'a [u8]> + 'a,
    {
        Ok(u64::from_be_bytes(
            // u32 size is 32 bytes
            self.data[index..(index + 64)]
                .try_into()
                .map_err(|_| Error)?
        ))
    }

    /// Reads pubkey field at the given offset
    fn read_pubkey(&self, index: usize) -> Result<Pubkey, Error> {
        Ok(Pubkey::try_from(
            // Pubkey size is 32 bytes
            &self.data[index..(index + 32)],
        )
        .map_err(|_| Error)?)
    }


    fn format_burn_message_body(
        version: u32,
        burn_token: &Pubkey,
        mint_recipient: &Pubkey,
        amount: u64,
        message_sender: &Pubkey,
    ) {
        // todo: format burn message body
    }
    // todo: replace message_hash with a keccak hash
    fn verify_attestation_signatures(message_hash: &Vec<u8>, attestation: &Vec<u8>){
        // todo: verify signatures using ECDSA, ED25519 or SECP256k1?
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
            next_available_nonce: 0,                     // start from nonce = 0
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
