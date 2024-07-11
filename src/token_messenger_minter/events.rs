use crate::GenericAddress;
use odra::casper_types::U256;
use odra::prelude::*;

#[odra::event]
pub struct DepositForBurn {
    pub nonce: u64,
    pub burn_token: GenericAddress,
    pub amount: U256,
    pub depositor: GenericAddress,
    pub mint_recipient: GenericAddress,
    pub destination_domain: u32,
    pub destination_token_messenger: GenericAddress,
    pub destination_caller: GenericAddress,
}

#[odra::event]
pub struct MintAndWithdraw {
    pub mint_recipient: GenericAddress,
    pub amount: U256,
    pub mint_token: GenericAddress,
}

#[odra::event]
pub struct RemoteTokenMessengerAdded {
    pub domain: u32,
    pub token_messenger: GenericAddress,
}

#[odra::event]
pub struct RemoteTokenMessengerRemoved {
    pub domain: u32,
    pub token_messenger: GenericAddress,
}

#[odra::event]
pub struct TokenPairLinked {
    pub local_token: GenericAddress,
    pub remote_token: GenericAddress,
    pub domain: u32,
}

#[odra::event]
pub struct TokenPairUnlinked {
    pub local_token: GenericAddress,
    pub remote_token: GenericAddress,
    pub domain: u32,
}
