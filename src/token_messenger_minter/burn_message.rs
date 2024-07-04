extern crate alloc;
use crate::{GenericAddress, Pubkey};
use alloc::{vec, vec::Vec};
use odra::casper_types::ContractPackageHash;

pub struct BurnMessage<'a> {
    pub data: &'a [u8],
}

impl<'a> BurnMessage<'a> {
    // Indices of each field in the message
    const VERSION_INDEX: usize = 0;
    const BURN_TOKEN_INDEX: usize = 4;
    const MINT_RECIPIENT_INDEX: usize = 36;
    const AMOUNT_INDEX: usize = 68;
    const MSG_SENDER_INDEX: usize = 100;
    // 4 byte version + 32 bytes burnToken + 32 bytes mintpubkey + 32 bytes amount + 32 bytes messageSender
    const BURN_MESSAGE_LEN: usize = 132;
    // EVM amount is 32 bytes while we use only 8 bytes on Solana
    const AMOUNT_OFFSET: usize = 24;

    /// Validates source array size and returns a new message
    pub fn new(message_bytes: &'a [u8]) -> Self {
        Self {
            data: &message_bytes,
        }
    }

    #[allow(clippy::too_many_arguments)]
    /// Serializes given fields into a burn message
    pub fn format_message(
        version: u32,
        // always contract
        burn_token: &GenericAddress,
        mint_recipient: &Pubkey,
        amount: u64,
        // throw away the byte
        message_sender: &GenericAddress,
    ) -> Vec<u8> {
        let mut output: Vec<u8> = vec![0; Self::BURN_MESSAGE_LEN];
        output[Self::VERSION_INDEX..Self::BURN_TOKEN_INDEX].copy_from_slice(&version.to_be_bytes());
        output[Self::BURN_TOKEN_INDEX..Self::MINT_RECIPIENT_INDEX]
            .copy_from_slice(burn_token.as_ref());
        output[Self::MINT_RECIPIENT_INDEX..Self::AMOUNT_INDEX]
            .copy_from_slice(mint_recipient.as_ref());
        output[(Self::AMOUNT_INDEX + Self::AMOUNT_OFFSET)..Self::MSG_SENDER_INDEX]
            .copy_from_slice(&amount.to_be_bytes());
        output[Self::MSG_SENDER_INDEX..Self::BURN_MESSAGE_LEN]
            .copy_from_slice(message_sender.as_ref());

        output
    }

    /// Returns version field
    pub fn version(&self) -> u32 {
        self.read_u32(Self::VERSION_INDEX)
    }

    /// Returns burn_token field
    pub fn burn_token(&self) -> Pubkey {
        self.read_pubkey(Self::BURN_TOKEN_INDEX)
    }

    /// Returns mint_pubkey field
    pub fn mint_recipient(&self) -> Pubkey {
        self.read_pubkey(Self::MINT_RECIPIENT_INDEX)
    }

    /// Returns amount field
    pub fn amount(&self) -> u64 {
        self.read_u64(Self::AMOUNT_INDEX + Self::AMOUNT_OFFSET)
    }

    /// Returns message_sender field
    pub fn message_sender(&self) -> Pubkey {
        self.read_pubkey(Self::MSG_SENDER_INDEX)
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
    fn read_pubkey(&self, index: usize) -> Pubkey {
        Pubkey::try_from(
            // Pubkey size is 32 bytes
            &self.data[index..(index + 32)],
        )
        .unwrap()
    }
}
