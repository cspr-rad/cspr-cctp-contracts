use crate::stablecoin::storage::Roles::Role;
use odra::casper_types::U256;
use odra::prelude::*;

type GenericAddress = [u8; 32];

/// An event emitted when a mint operation is performed.
#[odra::event]
pub struct Mint {
    /// The recipient of the minted tokens.
    pub recipient: GenericAddress,
    /// The amount of tokens minted.
    pub amount: U256,
}

/// An event emitted when a burn operation is performed.
#[odra::event]
pub struct Burn {
    /// The owner of the tokens that are burned.
    pub owner: GenericAddress,
    /// The amount of tokens burned.
    pub amount: U256,
}

/// An event emitted when an allowance is set.
#[odra::event]
pub struct SetAllowance {
    /// The owner of the tokens.
    pub owner: GenericAddress,
    /// The spender that is allowed to spend the tokens.
    pub spender: GenericAddress,
    /// The allowance amount.
    pub allowance: U256,
}

/// An event emitted when an allowance is increased.
#[odra::event]
pub struct IncreaseAllowance {
    /// The owner of the tokens.
    pub owner: GenericAddress,
    /// The spender that is allowed to spend the tokens.
    pub spender: GenericAddress,
    /// The final allowance amount.
    pub allowance: U256,
    /// The amount by which the allowance was increased.
    pub inc_by: U256,
}

/// An event emitted when an allowance is decreased.
#[odra::event]
pub struct DecreaseAllowance {
    /// The owner of the tokens.
    pub owner: GenericAddress,
    /// The spender that is allowed to spend the tokens.
    pub spender: GenericAddress,
    /// The final allowance amount.
    pub allowance: U256,
    /// The amount by which the allowance was decreased.
    pub decr_by: U256,
}

/// An event emitted when a transfer is performed.
#[odra::event]
pub struct Transfer {
    /// The sender of the tokens.
    pub sender: GenericAddress,
    /// The recipient of the tokens.
    pub recipient: GenericAddress,
    /// The amount of tokens transferred.
    pub amount: U256,
}

/// An event emitted when a transfer_from is performed.
#[odra::event]
pub struct TransferFrom {
    /// The spender that is allowed to spend the tokens.
    pub spender: GenericAddress,
    /// The sender of the tokens.
    pub owner: GenericAddress,
    /// The recipient of the tokens.
    pub recipient: GenericAddress,
    /// The amount of tokens transferred.
    pub amount: U256,
}

// Stablecoin Events

#[odra::event]
/// Emitted when account ID is blacklisted.
pub struct Blacklist {
    pub account: GenericAddress,
}

#[odra::event]
/// Emitted when blacklister account ID is changed
pub struct BlacklisterChanged {
    pub new_blacklister: GenericAddress,
}

#[odra::event]
/// Emitted when a controller is configured with a minter.
pub struct ControllerConfigured {
    pub controller: GenericAddress,
    pub minter: GenericAddress,
}

#[odra::event]
/// Emitted when a controller is disabled.
pub struct ControllerRemoved {
    pub controller: GenericAddress,
}

#[odra::event]
/// Emitted when minter account ID is configured.
pub struct MinterConfigured {
    pub minter: GenericAddress,
    pub minter_allowance: U256,
}

#[odra::event]
/// Emitted when minter account ID is removed.
pub struct MinterRemoved {
    pub minter: GenericAddress,
}

#[odra::event]
/// Emitted when contract is paused.
pub struct Paused {}

#[odra::event]
/// Emitted when contract is unpaused.
pub struct Unpaused {}

#[odra::event]
/// Emitted when an account is configured as one of the contract's main multi-sig roles, e.g.
/// Admin, MasterMinter, etc.
pub struct RoleConfigured {
    pub role: Role,
    pub account: GenericAddress,
}

#[odra::event]
/// Emitted when one of the contract's main multi-sig roles, e.g. Admin, MasterMinter, etc.
/// is revoked from their role.
pub struct RoleRevoked {
    pub role: Role,
    pub account: GenericAddress,
}

#[odra::event]
/// Emitted when account ID is unblacklisted.
pub struct Unblacklist {
    pub account: GenericAddress,
}
