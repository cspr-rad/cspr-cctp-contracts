#[cfg(test)]
mod transfer_tests {
    use odra::casper_types::U256;
    use odra::host::{Deployer, HostRef, NoArgs};

    use crate::stablecoin::errors::Error::{CannotTargetSelfUser, InsufficientBalance};
    use crate::stablecoin::cross_contract_test::StablecoinClientContractHostRef;
    use crate::stablecoin::setup_tests::{
        setup, ALLOWANCE_AMOUNT_1, TOKEN_TOTAL_SUPPLY, TRANSFER_AMOUNT_1,
    };

    #[test]
    fn should_transfer_full_owned_amount() {
        let (env, .., mut stablecoin) = setup();
        let owner = env.get_account(0);
        let alice = env.get_account(1);
        let amount = TOKEN_TOTAL_SUPPLY.into();
        stablecoin.transfer(&alice, &amount);
        assert!(
            stablecoin.env().emitted(&stablecoin, "Transfer"),
            "Transfer event not emitted"
        );
        assert_eq!(stablecoin.balance_of(&owner), 0.into());
        assert_eq!(stablecoin.balance_of(&alice), amount);
        assert_eq!(stablecoin.total_supply(), amount);
    }

    #[test]
    fn should_not_transfer_more_than_owned_balance() {
        let (env, .., mut stablecoin) = setup();
        let owner = env.get_account(0);
        let alice = env.get_account(1);
        let amount = TOKEN_TOTAL_SUPPLY.into();
        let result = stablecoin.try_transfer(&alice, &(amount + 1));
        assert_eq!(result.err().unwrap(), InsufficientBalance.into());
        assert_eq!(stablecoin.balance_of(&owner), amount);
        assert_eq!(stablecoin.balance_of(&alice), 0.into());
        assert_eq!(stablecoin.total_supply(), amount);
    }

    #[test]
    fn should_transfer_from_account_to_account() {
        let (env, .., mut stablecoin) = setup();
        let owner = env.get_account(0);
        let alice = env.get_account(1);
        let transfer_amount = TRANSFER_AMOUNT_1.into();
        let allowance_amount = ALLOWANCE_AMOUNT_1.into();
        stablecoin.approve(&alice, &allowance_amount);
        assert!(
            stablecoin.env().emitted(&stablecoin, "SetAllowance"),
            "SetAllowance event not emitted"
        );
        assert_eq!(stablecoin.allowance(&owner, &alice), allowance_amount);
        env.set_caller(alice);
        stablecoin.transfer_from(&owner, &alice, &transfer_amount);
        assert_eq!(
            stablecoin.balance_of(&owner),
            U256::from(TOKEN_TOTAL_SUPPLY) - transfer_amount
        );
        assert!(
            stablecoin.env().emitted(&stablecoin, "TransferFrom"),
            "TransferFrom event not emitted"
        );
        assert_eq!(stablecoin.balance_of(&alice), transfer_amount);
        assert_eq!(
            stablecoin.allowance(&owner, &alice),
            allowance_amount - transfer_amount
        );
    }

    #[test]
    fn should_transfer_from_account_by_contract() {
        let (env, .., mut stablecoin) = setup();
        let client_contract = StablecoinClientContractHostRef::deploy(stablecoin.env(), NoArgs);
        let spender = env.get_account(1);
        let owner = env.get_account(0);
        stablecoin.approve(client_contract.address(), &ALLOWANCE_AMOUNT_1.into());
        assert!(
            stablecoin.env().emitted(&stablecoin, "SetAllowance"),
            "SetAllowance event not emitted"
        );
        let spender_allowance_before = stablecoin.allowance(&owner, client_contract.address());
        let owner_balance_before = stablecoin.balance_of(&owner);
        client_contract.transfer_from_as_stored_contract(
            *stablecoin.address(),
            owner,
            spender,
            ALLOWANCE_AMOUNT_1.into(),
        );
        assert!(
            stablecoin.env().emitted(&stablecoin, "TransferFrom"),
            "TransferFrom event not emitted"
        );
        assert_eq!(
            spender_allowance_before - ALLOWANCE_AMOUNT_1,
            stablecoin.allowance(&owner, &spender)
        );
        assert!(
            stablecoin.env().emitted(&stablecoin, "SetAllowance"),
            "SetAllowance event not emitted"
        );
        assert_eq!(
            owner_balance_before - ALLOWANCE_AMOUNT_1,
            stablecoin.balance_of(&owner)
        );
        assert_eq!(
            U256::from(ALLOWANCE_AMOUNT_1),
            stablecoin.balance_of(&spender)
        );
    }

    #[test]
    fn should_not_be_able_to_own_transfer() {
        let (env, .., mut stablecoin) = setup();
        let owner = env.get_account(0);
        let amount = TOKEN_TOTAL_SUPPLY.into();
        let result = stablecoin.try_transfer(&owner, &amount);
        assert_eq!(result.err().unwrap(), CannotTargetSelfUser.into());
        assert_eq!(stablecoin.balance_of(&owner), amount);
        assert_eq!(stablecoin.total_supply(), amount);
    }

    #[test]
    fn should_not_be_able_to_own_transfer_from() {
        let (env, .., mut stablecoin) = setup();
        let owner = env.get_account(0);
        let amount = TOKEN_TOTAL_SUPPLY.into();
        let result = stablecoin.try_approve(&owner, &amount);
        assert_eq!(result.err().unwrap(), CannotTargetSelfUser.into());
        let result = stablecoin.try_transfer_from(&owner, &owner, &amount);
        assert_eq!(result.err().unwrap(), CannotTargetSelfUser.into());
        assert_eq!(stablecoin.balance_of(&owner), amount);
        assert_eq!(stablecoin.total_supply(), amount);
    }

    #[test]
    fn should_verify_zero_amount_transfer_is_noop() {
        let (env, .., mut stablecoin) = setup();
        let owner = env.get_account(0);
        let alice = env.get_account(1);
        let amount = TOKEN_TOTAL_SUPPLY.into();
        stablecoin.transfer(&alice, &U256::zero());
        assert_eq!(stablecoin.balance_of(&owner), amount);
        assert_eq!(stablecoin.balance_of(&alice), 0.into());
        assert_eq!(stablecoin.total_supply(), amount);
    }

    #[test]
    fn should_verify_zero_amount_transfer_from_is_noop() {
        let (env, .., mut stablecoin) = setup();
        let owner = env.get_account(0);
        let alice = env.get_account(1);
        let amount = TOKEN_TOTAL_SUPPLY.into();
        stablecoin.approve(&alice, &ALLOWANCE_AMOUNT_1.into());
        stablecoin.transfer_from(&owner, &alice, &U256::zero());
        assert_eq!(stablecoin.balance_of(&owner), amount);
        assert_eq!(stablecoin.balance_of(&alice), 0.into());
        assert_eq!(stablecoin.total_supply(), amount);
    }

    #[test]
    fn should_transfer() {
        let (env, .., mut stablecoin) = setup();
        let owner = env.get_account(0);
        let client_contract = StablecoinClientContractHostRef::deploy(stablecoin.env(), NoArgs);
        stablecoin.transfer(client_contract.address(), &TRANSFER_AMOUNT_1.into());
        assert!(
            stablecoin.env().emitted(&stablecoin, "Transfer"),
            "Transfer event not emitted"
        );
        assert_eq!(
            stablecoin.balance_of(&owner),
            (TOKEN_TOTAL_SUPPLY - TRANSFER_AMOUNT_1).into()
        );
        assert_eq!(
            stablecoin.balance_of(client_contract.address()),
            TRANSFER_AMOUNT_1.into()
        );
        client_contract.transfer_as_stored_contract(
            *stablecoin.address(),
            *stablecoin.address(),
            TRANSFER_AMOUNT_1.into(),
        );
        assert_eq!(stablecoin.balance_of(client_contract.address()), 0.into());
        assert_eq!(
            stablecoin.balance_of(stablecoin.address()),
            TRANSFER_AMOUNT_1.into()
        );
    }
}
