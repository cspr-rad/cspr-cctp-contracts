use alloc::string::String;
use odra::casper_types::U256;
use odra::ExecutionError::AdditionOverflow;

use odra::casper_types::bytesrepr::ToBytes;
use odra::{prelude::*, Mapping};
use odra::{Address, UnwrapOrRevert};

use crate::stablecoin::errors::Error::{InvalidState, Overflow};

use base64::prelude::*;

use super::events::{RoleConfigured, RoleRevoked};
const ALLOWANCES_KEY: &str = "allowances";
const MINTER_ALLOWANCES_KEY: &str = "minter_allowances";
const BALANCES_KEY: &str = "balances";
const NAME_KEY: &str = "name";
const DECIMALS_KEY: &str = "decimals";
const SYMBOL_KEY: &str = "symbol";
const TOTAL_SUPPLY_KEY: &str = "total_supply";

#[odra::module]
/// Storage module for the name of the token.
pub struct StablecoinNameStorage;

#[odra::module]
impl StablecoinNameStorage {
    /// Sets the name of the token.
    pub fn set(&self, name: String) {
        self.env().set_named_value(NAME_KEY, name);
    }

    /// Gets the name of the token.
    pub fn get(&self) -> String {
        self.env()
            .get_named_value(NAME_KEY)
            .unwrap_or_revert_with(&self.env(), InvalidState)
    }
}

#[odra::module]
/// Storage module for the number of decimals of the token.
pub struct StablecoinDecimalsStorage;

#[odra::module]
impl StablecoinDecimalsStorage {
    /// Sets the number of decimals of the token.
    pub fn set(&self, decimals: u8) {
        self.env().set_named_value(DECIMALS_KEY, decimals);
    }

    /// Gets the number of decimals of the token.
    pub fn get(&self) -> u8 {
        self.env()
            .get_named_value(DECIMALS_KEY)
            .unwrap_or_revert_with(&self.env(), InvalidState)
    }
}

#[odra::module]
/// Storage module for the symbol of the token.
pub struct StablecoinSymbolStorage;

#[odra::module]
impl StablecoinSymbolStorage {
    /// Sets the symbol of the token.
    pub fn set(&self, symbol: String) {
        self.env().set_named_value(SYMBOL_KEY, symbol);
    }

    /// Gets the symbol of the token.
    pub fn get(&self) -> String {
        self.env()
            .get_named_value(SYMBOL_KEY)
            .unwrap_or_revert_with(&self.env(), InvalidState)
    }
}

#[odra::module]
/// Storage module for the total supply of the token.
pub struct StablecoinTotalSupplyStorage;

#[odra::module]
impl StablecoinTotalSupplyStorage {
    /// Sets the total supply of the token.
    pub fn set(&self, total_supply: U256) {
        self.env().set_named_value(TOTAL_SUPPLY_KEY, total_supply);
    }

    /// Gets the total supply of the token.
    pub fn get(&self) -> U256 {
        self.env()
            .get_named_value(TOTAL_SUPPLY_KEY)
            .unwrap_or_revert_with(&self.env(), InvalidState)
    }

    /// Adds the given amount to the total supply of the token.
    pub fn add(&self, amount: U256) {
        let total_supply = self.get();
        let new_total_supply = total_supply
            .checked_add(amount)
            .unwrap_or_revert_with(&self.env(), AdditionOverflow);
        self.set(new_total_supply);
    }

    /// Subtracts the given amount from the total supply of the token.
    pub fn subtract(&self, amount: U256) {
        let total_supply = self.get();
        let new_total_supply = total_supply
            .checked_sub(amount)
            .unwrap_or_revert_with(&self.env(), Overflow);
        self.set(new_total_supply);
    }
}

#[odra::module]
/// Storage module for the balances of the token.
pub struct StablecoinBalancesStorage;

#[odra::module]
impl StablecoinBalancesStorage {
    /// Sets the balance of the given account.
    pub fn set(&self, account: &Address, balance: U256) {
        self.env()
            .set_dictionary_value(BALANCES_KEY, self.key(account).as_bytes(), balance);
    }

    /// Gets the balance of the given account.
    pub fn get_or_default(&self, account: &Address) -> U256 {
        self.env()
            .get_dictionary_value(BALANCES_KEY, self.key(account).as_bytes())
            .unwrap_or_default()
    }

    /// Adds the given amount to the balance of the given account.
    pub fn add(&self, account: &Address, amount: U256) {
        let balance = self.get_or_default(account);
        let new_balance = balance.checked_add(amount).unwrap_or_revert(&self.env());
        self.set(account, new_balance);
    }

    /// Subtracts the given amount from the balance of the given account.
    pub fn subtract(&self, account: &Address, amount: U256) {
        let balance = self.get_or_default(account);
        let new_balance = balance
            .checked_sub(amount)
            .unwrap_or_revert_with(&self.env(), Overflow);
        self.set(account, new_balance);
    }

    fn key(&self, owner: &Address) -> String {
        // PRENOTE: This note is copied from the original implementation of CEP-18.
        // NOTE: As for now dictionary item keys are limited to 64 characters only. Instead of using
        // hashing (which will effectively hash a hash) we'll use base64. Preimage is 33 bytes for
        // both used Key variants, and approximated base64-encoded length will be 4 * (33 / 3) ~ 44
        // characters.
        // Even if the preimage increased in size we still have extra space but even in case of much
        // larger preimage we can switch to base85 which has ratio of 4:5.
        let preimage = owner.to_bytes().unwrap_or_revert(&self.env());
        BASE64_STANDARD.encode(preimage)
    }
}

#[odra::module]
/// Storage module for the allowances of the token.
pub struct StablecoinAllowancesStorage;

#[odra::module]
impl StablecoinAllowancesStorage {
    /// Sets the allowance of the given owner and spender.
    pub fn set(&self, owner: &Address, spender: &Address, amount: U256) {
        self.env()
            .set_dictionary_value(ALLOWANCES_KEY, &self.key(owner, spender), amount);
    }

    /// Gets the allowance of the given owner and spender.
    pub fn get_or_default(&self, owner: &Address, spender: &Address) -> U256 {
        self.env()
            .get_dictionary_value(ALLOWANCES_KEY, &self.key(owner, spender))
            .unwrap_or_default()
    }

    /// Adds the given amount to the allowance of the given owner and spender.
    pub fn add(&self, owner: &Address, spender: &Address, amount: U256) {
        let allowance = self.get_or_default(owner, spender);
        let new_allowance = allowance
            .checked_add(amount)
            .unwrap_or_revert_with(&self.env(), AdditionOverflow);
        self.set(owner, spender, new_allowance);
    }

    /// Subtracts the given amount from the allowance of the given owner and spender.
    pub fn subtract(&self, owner: &Address, spender: &Address, amount: U256) {
        let allowance = self.get_or_default(owner, spender);
        let new_allowance = allowance
            .checked_sub(amount)
            .unwrap_or_revert_with(&self.env(), AdditionOverflow);
        self.set(owner, spender, new_allowance);
    }

    fn key(&self, owner: &Address, spender: &Address) -> [u8; 64] {
        let mut result = [0u8; 64];
        let mut preimage = Vec::new();
        preimage.append(&mut owner.to_bytes().unwrap_or_revert(&self.env()));
        preimage.append(&mut spender.to_bytes().unwrap_or_revert(&self.env()));

        let key_bytes = self.env().hash(&preimage);
        odra::utils::hex_to_slice(&key_bytes, &mut result);
        result
    }
}

#[odra::module]
/// Storage module for the allowances of the token.
pub struct StablecoinMinterAllowancesStorage;

#[odra::module]
impl StablecoinMinterAllowancesStorage {
    /// Sets the allowance of the given owner and spender.
    pub fn set(&self, minter: &Address, amount: U256) {
        self.env()
            .set_dictionary_value(MINTER_ALLOWANCES_KEY, &self.key(minter), amount);
    }

    /// Gets the allowance of the given owner and spender.
    pub fn get_or_default(&self, minter: &Address) -> U256 {
        self.env()
            .get_dictionary_value(MINTER_ALLOWANCES_KEY, &self.key(minter))
            .unwrap_or_default()
    }

    /// Adds the given amount to the allowance of the given owner and spender.
    pub fn add(&self, minter: &Address, amount: U256) {
        let allowance = self.get_or_default(minter);
        let new_allowance = allowance
            .checked_add(amount)
            .unwrap_or_revert_with(&self.env(), AdditionOverflow);
        self.set(minter, new_allowance);
    }

    /// Subtracts the given amount from the allowance of the given owner and spender.
    pub fn subtract(&self, minter: &Address, amount: U256) {
        let allowance = self.get_or_default(minter);
        let new_allowance = allowance
            .checked_sub(amount)
            .unwrap_or_revert_with(&self.env(), AdditionOverflow);
        self.set(minter, new_allowance);
    }

    fn key(&self, account: &Address) -> [u8; 64] {
        let mut result = [0u8; 64];
        let mut preimage = Vec::new();
        preimage.append(&mut account.to_bytes().unwrap_or_revert(&self.env()));
        let key_bytes = self.env().hash(&preimage);
        odra::utils::hex_to_slice(&key_bytes, &mut result);
        result
    }
}

#[allow(non_snake_case)]
pub mod Roles {
    pub type Role = [u8; 32];
    #[allow(non_upper_case_globals)]
    pub const Minter: Role = [0u8; 32];
    #[allow(non_upper_case_globals)]
    pub const MasterMinter: Role = [1u8; 32];
    #[allow(non_upper_case_globals)]
    pub const Blacklister: Role = [2u8; 32];
    #[allow(non_upper_case_globals)]
    pub const Blacklisted: Role = [3u8; 32];
    #[allow(non_upper_case_globals)]
    pub const Pauser: Role = [4u8; 32];
    #[allow(non_upper_case_globals)]
    pub const Controller: Role = [5u8; 32];
    #[allow(non_upper_case_globals)]
    pub const Owner: Role = [6u8; 32];
}

#[odra::module(events=[RoleConfigured, RoleRevoked])]
/// Storage module for the allowances of the token.
pub struct StablecoinRoles {
    roles: Mapping<(Roles::Role, Address), bool>,
}

#[odra::module]
impl StablecoinRoles {
    pub fn configure_role(&mut self, role: &Roles::Role, account: &Address) {
        self.roles.set(&(*role, *account), true);
        self.env().emit_event(RoleConfigured {
            role: *role,
            account: *account,
        });
    }

    pub fn revoke_role(&mut self, role: &Roles::Role, account: &Address) {
        if self.has_role(role, account) {
            self.roles.set(&(*role, *account), false);
            self.env().emit_event(RoleRevoked {
                role: *role,
                account: *account,
            });
        }
    }

    pub fn is_minter(&self, account: &Address) -> bool {
        self.has_role(&Roles::Minter, account)
    }
    pub fn is_master_minter(&self, account: &Address) -> bool {
        self.has_role(&Roles::MasterMinter, account)
    }
    pub fn is_blacklister(&self, account: &Address) -> bool {
        self.has_role(&Roles::Blacklister, account)
    }
    pub fn is_blacklisted(&self, account: &Address) -> bool {
        self.has_role(&Roles::Blacklisted, account)
    }
    pub fn is_pauser(&self, account: &Address) -> bool {
        self.has_role(&Roles::Pauser, account)
    }
    pub fn is_controller(&self, account: &Address) -> bool {
        self.has_role(&Roles::Controller, account)
    }
    pub fn is_owner(&self, account: &Address) -> bool {
        self.has_role(&Roles::Owner, account)
    }
    pub fn has_role(&self, role: &Roles::Role, account: &Address) -> bool {
        self.roles.get_or_default(&(*role, *account))
    }
}
