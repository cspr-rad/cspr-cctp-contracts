#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
extern crate alloc;

pub mod message_transmitter;
pub mod stablecoin;
pub mod token_messenger_minter;

// type alias for generic Pubkey
pub type Pubkey = [u8; 32];
