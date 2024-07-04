
use crate::{GenericAddress, Pubkey};
use odra::casper_types::U256;
use odra::prelude::*;

#[odra::event]
pub struct DepositForBurn {
    pub nonce: u64,
    pub burn_token: GenericAddress,
    pub amount: U256,
    pub depositor: GenericAddress,
    pub mint_recipient: Pubkey,
    pub destination_domain: u32,
    pub destination_token_messenger: Pubkey,
    pub destination_caller: Pubkey
}