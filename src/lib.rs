#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
extern crate alloc;
use alloc::vec::Vec;
use odra::{casper_types::bytesrepr::ToBytes, Address};
pub mod message_transmitter;
pub mod stablecoin;
pub mod token_messenger_minter;

// type alias for generic Pubkey
pub type Pubkey = [u8; 32];
// type alias for a generic Casper Address
pub type GenericAddress = [u8;32];
// a keccak
pub type Hash = Vec<u8>;

pub fn generic_address(address: Address) -> GenericAddress{
    let mut address_bytes = address.to_bytes().unwrap();
    address_bytes.remove(0);
    address_bytes.try_into().unwrap()
}