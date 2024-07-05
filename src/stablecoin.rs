//! CEP-18 Casper Fungible Token standard implementation.
use odra::prelude::*;
use odra::{casper_types::U256, Address, Mapping, SubModule, UnwrapOrRevert, Var};

pub mod cross_contract_test;
pub mod errors;
pub mod events;
pub mod storage;
#[cfg(test)]
mod tests;
pub mod utils;

use crate::stablecoin::errors::Error;

use crate::generic_address;
use crate::stablecoin::events::{
    Blacklist, BlacklisterChanged, Burn, ControllerConfigured, ControllerRemoved,
    DecreaseAllowance, IncreaseAllowance, Mint, MinterConfigured, MinterRemoved, Paused,
    SetAllowance, Transfer, TransferFrom, Unblacklist, Unpaused,
};
use crate::stablecoin::storage::Roles::{self, Role};
use crate::stablecoin::storage::{
    StablecoinAllowancesStorage, StablecoinBalancesStorage, StablecoinDecimalsStorage,
    StablecoinMinterAllowancesStorage, StablecoinNameStorage, StablecoinRoles,
    StablecoinSymbolStorage, StablecoinTotalSupplyStorage,
};
use crate::stablecoin::utils::StablecoinModality;

/// CEP-18 token module
#[odra::module(events = [Mint, Burn, SetAllowance, IncreaseAllowance, DecreaseAllowance, Transfer, TransferFrom])]
pub struct Stablecoin {
    decimals: SubModule<StablecoinDecimalsStorage>,
    symbol: SubModule<StablecoinSymbolStorage>,
    name: SubModule<StablecoinNameStorage>,
    total_supply: SubModule<StablecoinTotalSupplyStorage>,
    balances: SubModule<StablecoinBalancesStorage>,
    allowances: SubModule<StablecoinAllowancesStorage>,
    minter_allowances: SubModule<StablecoinMinterAllowancesStorage>,
    roles: SubModule<StablecoinRoles>,
    controllers: Mapping<Address, Address>,
    // The Blacklister for this Contract
    blacklister: Var<Address>,
    paused: Var<bool>,
    /// This stores all Stablecoin Roles (MasterMinters, Owners, Pauser, ...)
    modality: Var<StablecoinModality>,
}

#[odra::module]
impl Stablecoin {
    /// Initializes the contract with the given metadata, initial supply, security and modality.
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: U256,
        // the master_minter can't mint
        master_minter_list: Vec<Address>,
        pauser_list: Vec<Address>,
        blacklister: Address,
        modality: Option<StablecoinModality>,
    ) {
        let caller: Address = self.env().caller();
        // set the metadata
        self.symbol.set(symbol);
        self.name.set(name);
        self.decimals.set(decimals);
        self.total_supply.set(initial_supply);

        // mint the initial supply for the caller
        self.balances.set(&generic_address(caller), initial_supply);
        self.env().emit_event(Mint {
            recipient: generic_address(caller),
            amount: initial_supply,
        });

        for master_minter in master_minter_list {
            self.roles
                .configure_role(&Roles::MasterMinter, &generic_address(master_minter));
        }

        for pauser in pauser_list {
            self.roles
                .configure_role(&Roles::Pauser, &generic_address(pauser));
        }

        self.roles
            .configure_role(&Roles::Blacklister, &generic_address(blacklister));

        // set the modality
        if let Some(modality) = modality {
            self.modality.set(modality);
        }
    }

    /// Returns the name of the token.
    pub fn name(&self) -> String {
        self.name.get()
    }

    /// Returns the symbol of the token.
    pub fn symbol(&self) -> String {
        self.symbol.get()
    }

    /// Returns the number of decimals the token uses.
    pub fn decimals(&self) -> u8 {
        self.decimals.get()
    }

    /// Returns the total supply of the token.
    pub fn total_supply(&self) -> U256 {
        self.total_supply.get()
    }

    /// Returns the balance of the given address.
    pub fn balance_of(&self, address: &Address) -> U256 {
        self.balances.get_or_default(&generic_address(*address))
    }

    /// Returns the amount of tokens the owner has allowed the spender to spend.
    pub fn allowance(&self, owner: &Address, spender: &Address) -> U256 {
        self.allowances
            .get_or_default(&generic_address(*owner), &generic_address(*spender))
    }

    /// Approves the spender to spend the given amount of tokens on behalf of the caller.
    pub fn approve(&mut self, spender: &Address, amount: &U256) {
        self.require_unpaused();
        self.require_not_role(&self.caller(), &Roles::Blacklisted);
        self.require_not_role(spender, &Roles::Blacklisted);
        let owner = self.env().caller();
        if owner == *spender {
            self.env().revert(Error::CannotTargetSelfUser);
        }

        self.allowances
            .set(&generic_address(owner), &generic_address(*spender), *amount);
        self.env().emit_event(SetAllowance {
            owner: generic_address(owner),
            spender: generic_address(*spender),
            allowance: *amount,
        });
    }

    /// Decreases the allowance of the spender by the given amount.
    pub fn decrease_allowance(&mut self, spender: &Address, decr_by: &U256) {
        let owner = self.env().caller();
        let allowance = self.allowance(&owner, spender);
        self.allowances.set(
            &generic_address(owner),
            &generic_address(*spender),
            allowance.saturating_sub(*decr_by),
        );
        self.env().emit_event(DecreaseAllowance {
            owner: generic_address(owner),
            spender: generic_address(*spender),
            allowance,
            decr_by: *decr_by,
        });
    }

    /// Increases the allowance of the spender by the given amount.
    pub fn increase_allowance(&mut self, spender: &Address, inc_by: &U256) {
        let owner = self.env().caller();
        if owner == *spender {
            self.env().revert(Error::CannotTargetSelfUser);
        }
        let allowance = self
            .allowances
            .get_or_default(&generic_address(owner), &generic_address(*spender));

        self.allowances.set(
            &generic_address(owner),
            &generic_address(*spender),
            allowance.saturating_add(*inc_by),
        );
        self.env().emit_event(IncreaseAllowance {
            owner: generic_address(owner),
            spender: generic_address(*spender),
            allowance,
            inc_by: *inc_by,
        });
    }

    /// Transfers tokens from the caller to the recipient.
    pub fn transfer(&mut self, recipient: &Address, amount: &U256) {
        let caller = self.env().caller();
        if caller == *recipient {
            self.env().revert(Error::CannotTargetSelfUser);
        }
        self.raw_transfer(&caller, recipient, amount);
    }

    /// Transfers tokens from the owner to the recipient using the spender's allowance.
    pub fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256) {
        self.require_not_role(owner, &Roles::Blacklisted);
        self.require_not_role(recipient, &Roles::Blacklisted);
        let spender = self.env().caller();
        if owner == recipient {
            self.env().revert(Error::CannotTargetSelfUser);
        }
        if amount.is_zero() {
            return;
        }
        let allowance = self.allowance(owner, &spender);
        self.allowances.set(
            &generic_address(*owner),
            &generic_address(*recipient),
            allowance
                .checked_sub(*amount)
                .unwrap_or_revert_with(&self.env(), Error::InsufficientAllowance),
        );
        self.env().emit_event(TransferFrom {
            spender: generic_address(spender),
            owner: generic_address(*owner),
            recipient: generic_address(*recipient),
            amount: *amount,
        });
        self.raw_transfer(owner, recipient, amount);
    }

    /// Burns the given amount of tokens from the given address.
    pub fn burn(&mut self, owner: &Address, amount: &U256) {
        self.assert_burn_and_mint_enabled();

        if self.env().caller() != *owner {
            self.env().revert(Error::InvalidBurnTarget);
        }

        if self.balance_of(owner) < *amount {
            self.env().revert(Error::InsufficientBalance);
        }

        self.raw_burn(owner, amount);
    }

    /// Burns the given amount of tokens from the given address.
    pub fn burn_cctp(&mut self, amount: U256, account: Address) {
        self.assert_burn_and_mint_enabled();
        self.require_not_role(&self.caller(), &Roles::Blacklisted);
        self.require_role(&self.caller(), &Roles::Minter);
        if amount == U256::zero() {
            self.env().revert(Error::InvalidAmount)
        }
        let minter_allowance: U256 = self
            .minter_allowances
            .get_or_default(&generic_address(*&self.caller()));
        if &minter_allowance < &amount {
            self.env().revert(Error::InsufficientMinterAllowance);
        }
        self.minter_allowances
            .subtract(&generic_address(*&self.caller()), amount);
        // must check spender allowance
        let allowance = self.allowance(&account, &self.caller());
        if allowance < amount {
            self.env().revert(Error::InsufficientAllowance);
        }
        self.allowances.set(
            &generic_address(account),
            &generic_address(self.caller()),
            allowance
                .checked_sub(amount)
                .unwrap_or_revert_with(&self.env(), Error::InsufficientAllowance),
        );
        // problem: Odra does not support callstack traversal
        self.raw_burn(&account, &amount);
    }

    /// Mints new tokens and assigns them to the given address.
    pub fn mint(&mut self, owner: &Address, amount: U256) {
        self.require_role(&self.caller(), &Roles::Minter);
        self.require_not_role(owner, &Roles::Blacklisted);
        self.assert_burn_and_mint_enabled();
        let minter_allowance: U256 = self
            .minter_allowances
            .get_or_default(&generic_address(*&self.caller()));
        if &minter_allowance < &amount {
            self.env().revert(Error::InsufficientMinterAllowance);
        }
        self.minter_allowances
            .subtract(&generic_address(*&self.caller()), amount);
        self.raw_mint(owner, &amount);
    }

    /// Pause this contract
    pub fn pause(&mut self) {
        self.require_role(&self.caller(), &Roles::Pauser);
        self.require_not_role(&self.caller(), &Roles::Blacklisted);
        self.paused.set(true);
        self.env().emit_event(Paused {});
    }

    /// Unpause this contract
    pub fn unpause(&mut self) {
        self.require_role(&self.caller(), &Roles::Pauser);
        self.require_not_role(&self.caller(), &Roles::Blacklisted);
        self.paused.set(false);
        self.env().emit_event(Unpaused {});
    }

    /// Blacklist an account
    pub fn blacklist(&mut self, account: &Address) {
        self.require_role(&self.caller(), &Roles::Blacklister);
        self.roles
            .configure_role(&Roles::Blacklisted, &generic_address(*account));
        self.env().emit_event(Blacklist {
            account: generic_address(*account),
        });
    }

    /// Remove an account from the Blacklist
    pub fn unblacklist(&mut self, account: &Address) {
        self.require_role(&self.caller(), &Roles::Blacklister);
        self.roles
            .revoke_role(&Roles::Blacklisted, &generic_address(*account));
        self.env().emit_event(Unblacklist {
            account: generic_address(*account),
        });
    }

    /// Update the Blacklister, can only be called by Owner
    pub fn update_blacklister(&mut self, new_blacklister: &Address) {
        self.require_role(&self.caller(), &Roles::Owner);
        self.roles.revoke_role(
            &Roles::Blacklister,
            &generic_address(
                *&self
                    .blacklister
                    .get()
                    // borrow checker is unhappy if we unwrap_or_revert() here.
                    .unwrap(),
            ),
        );
        self.roles
            .configure_role(&Roles::Blacklister, &generic_address(*new_blacklister));
        self.env().emit_event(BlacklisterChanged {
            new_blacklister: generic_address(*new_blacklister),
        });
        self.env().emit_event(BlacklisterChanged {
            new_blacklister: generic_address(*new_blacklister),
        });
    }

    /// Configure minter allowance
    pub fn configure_minter_allowance(&mut self, minter_allowance: U256) {
        self.require_role(&self.caller(), &Roles::Controller);
        let minter = self.get_associated_minter(&self.caller());
        self.require_not_role(&minter, &Roles::Blacklisted);
        self.minter_allowances
            .set(&generic_address(minter), minter_allowance);
        self.env().emit_event(MinterConfigured {
            minter: generic_address(minter),
            minter_allowance,
        });
    }

    /// Increase allowance for a minter
    pub fn increase_minter_allowance(&mut self, increment: U256) {
        self.require_role(&self.caller(), &Roles::Controller);
        let minter = self.get_associated_minter(&self.caller());
        self.require_not_role(&minter, &Roles::Blacklisted);
        self.minter_allowances
            .add(&generic_address(minter), increment);
        self.env().emit_event(MinterConfigured {
            minter: generic_address(minter),
            minter_allowance: self.minter_allowance(&minter),
        });
    }

    /// Decrease allowance for a minter
    pub fn decrease_minter_allowance(&mut self, decrement: U256) {
        self.require_role(&self.caller(), &Roles::Controller);
        let minter = self.get_associated_minter(&self.caller());
        self.require_not_role(&minter, &Roles::Blacklisted);
        self.minter_allowances
            .subtract(&generic_address(minter), decrement);
        self.env().emit_event(MinterConfigured {
            minter: generic_address(minter),
            minter_allowance: self.minter_allowance(&minter),
        });
    }

    /// Add a controller, minter pair
    pub fn configure_controller(&mut self, controller: &Address, minter: &Address) {
        self.require_role(&self.caller(), &Roles::MasterMinter);
        self.require_not_role(controller, &Roles::Blacklisted);
        self.require_not_role(minter, &Roles::Blacklisted);
        self.roles
            .configure_role(&Roles::Controller, &generic_address(*controller));
        self.roles
            .configure_role(&Roles::Minter, &generic_address(*minter));
        self.controllers.set(&controller, *minter);
        self.env().emit_event(ControllerConfigured {
            controller: generic_address(*controller),
            minter: generic_address(*minter),
        });
    }

    /// Remove a controller
    pub fn remove_controller(&mut self, controller: &Address) {
        self.require_role(&self.caller(), &Roles::MasterMinter);
        self.roles
            .revoke_role(&Roles::Controller, &generic_address(*controller));
        self.env().emit_event(ControllerRemoved {
            controller: generic_address(*controller),
        });
    }

    /// Remove the minter role from an account
    pub fn remove_minter(&mut self) {
        self.require_role(&self.caller(), &Roles::Controller);
        self.require_not_role(&self.caller(), &Roles::Blacklisted);
        let minter: Address = self.get_associated_minter(&self.env().caller());
        self.roles
            .revoke_role(&Roles::Minter, &generic_address(minter));
        self.env().emit_event(MinterRemoved {
            minter: generic_address(minter),
        })
    }

    // Queries start here

    /// Query if an account is a minter
    pub fn is_minter(&self, account: &Address) -> bool {
        self.roles.is_minter(&generic_address(*account))
    }

    /// Query if an account is blacklisted
    pub fn is_blacklisted(&self, account: &Address) -> bool {
        self.roles.is_blacklisted(&generic_address(*account))
    }

    /// Query the owners of this account
    pub fn is_owner(&self, account: &Address) -> bool {
        self.roles.is_owner(&generic_address(*account))
    }

    /// Query the owners of this account
    pub fn is_pausers(&self, account: &Address) -> bool {
        self.roles.is_pauser(&generic_address(*account))
    }

    /// Query a minter for a controller
    pub fn get_minter(&mut self) -> Address {
        self.require_role(&self.caller(), &Roles::Controller);
        self.require_not_role(&self.caller(), &Roles::Blacklisted);
        self.get_associated_minter(&self.caller())
    }

    /// Query the allowance of a minter
    pub fn minter_allowance(&self, minter: &Address) -> U256 {
        self.minter_allowances
            .get_or_default(&generic_address(*minter))
    }

    fn require_unpaused(&self) {
        if self.paused.get().unwrap_or(false) {
            self.env().revert(Error::ContractIsPaused);
        }
    }

    fn require_role(&mut self, account: &Address, role: &Role) {
        if !self.roles.has_role(&role, &generic_address(*account)) {
            self.env().revert(Error::InsufficientRights);
        }
    }

    fn require_not_role(&mut self, account: &Address, role: &Role) {
        if self.roles.has_role(&role, &generic_address(*account)) {
            self.env().revert(Error::InsufficientRights)
        }
    }

    // Get the minter that is associated with the controller
    fn get_associated_minter(&mut self, controller: &Address) -> Address {
        self.controllers
            .get(&controller)
            .unwrap_or_revert_with(&self.env(), Error::MissingController)
    }

    fn caller(&self) -> Address {
        self.env().caller()
    }
}

impl Stablecoin {
    /// Transfers tokens from the sender to the recipient without checking the permissions.
    fn raw_transfer(&mut self, sender: &Address, recipient: &Address, amount: &U256) {
        self.require_unpaused();
        if *amount > self.balances.get_or_default(&generic_address(*sender)) {
            self.env().revert(Error::InsufficientBalance)
        }

        if amount > &U256::zero() {
            self.balances.subtract(&generic_address(*sender), *amount);
            self.balances.add(&generic_address(*recipient), *amount);
        }

        self.env().emit_event(Transfer {
            sender: generic_address(*sender),
            recipient: generic_address(*recipient),
            amount: *amount,
        });
    }

    /// Mints new tokens and assigns them to the given address without checking the permissions.
    fn raw_mint(&mut self, owner: &Address, amount: &U256) {
        self.require_unpaused();
        self.total_supply.add(*amount);
        self.balances.add(&generic_address(*owner), *amount);

        self.env().emit_event(Mint {
            recipient: generic_address(*owner),
            amount: *amount,
        });
    }

    /// Burns the given amount of tokens from the given address without checking the permissions.
    fn raw_burn(&mut self, owner: &Address, amount: &U256) {
        self.require_unpaused();
        self.total_supply.subtract(*amount);
        self.balances.subtract(&generic_address(*owner), *amount);

        self.env().emit_event(Burn {
            owner: generic_address(*owner),
            amount: *amount,
        });
    }

    fn assert_burn_and_mint_enabled(&mut self) {
        // check if mint_burn is enabled
        if !self.modality.get_or_default().mint_and_burn_enabled() {
            self.env().revert(Error::MintBurnDisabled);
        }
    }
}

#[cfg(test)]
pub(crate) mod setup_tests {
    use alloc::string::ToString;
    use alloc::vec;
    use odra::host::{Deployer, HostEnv, HostRef};
    use odra::Address;

    use crate::stablecoin::{StablecoinHostRef, StablecoinInitArgs};

    pub const TOKEN_NAME: &str = "USDCoin";
    pub const TOKEN_SYMBOL: &str = "USDC";
    pub const TOKEN_DECIMALS: u8 = 100;
    pub const TOKEN_TOTAL_SUPPLY: u64 = 1_000_000_000;
    pub const TRANSFER_AMOUNT_1: u64 = 200_001;
    pub const ALLOWANCE_AMOUNT_1: u64 = 456_789;
    pub const ALLOWANCE_AMOUNT_2: u64 = 87_654;

    pub fn setup() -> (
        HostEnv,
        Address,
        Address,
        Address,
        Address,
        Address,
        Address,
        StablecoinHostRef,
    ) {
        let env = odra_test::env();
        let master_minter = env.get_account(1);
        let controller_1 = env.get_account(2);
        let minter_1 = env.get_account(3);
        let blacklister = env.get_account(4);
        let pauser = env.get_account(5);
        let user = env.get_account(6);
        let args = StablecoinInitArgs {
            symbol: TOKEN_SYMBOL.to_string(),
            name: TOKEN_NAME.to_string(),
            decimals: TOKEN_DECIMALS,
            initial_supply: TOKEN_TOTAL_SUPPLY.into(),
            master_minter_list: vec![master_minter],
            pauser_list: vec![pauser],
            blacklister: blacklister,
            modality: Some(crate::stablecoin::utils::StablecoinModality::MintAndBurn),
        };
        let stablecoin = setup_with_args(&env, args);
        (
            env,
            master_minter,
            controller_1,
            minter_1,
            blacklister,
            pauser,
            user,
            stablecoin,
        )
    }

    pub fn setup_with_args(env: &HostEnv, args: StablecoinInitArgs) -> StablecoinHostRef {
        StablecoinHostRef::deploy(env, args)
    }

    #[test]
    fn should_have_queryable_properties() {
        let (env, .., stablecoin) = setup();

        assert_eq!(stablecoin.name(), TOKEN_NAME);
        assert_eq!(stablecoin.symbol(), TOKEN_SYMBOL);
        assert_eq!(stablecoin.decimals(), TOKEN_DECIMALS);
        assert_eq!(stablecoin.total_supply(), TOKEN_TOTAL_SUPPLY.into());

        let admin_key = env.caller();
        let admin_balance = stablecoin.balance_of(&admin_key);
        assert_eq!(admin_balance, TOKEN_TOTAL_SUPPLY.into());

        let contract_balance = stablecoin.balance_of(stablecoin.address());
        assert_eq!(contract_balance, 0.into());
    }
}
