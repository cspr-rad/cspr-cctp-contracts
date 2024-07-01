#[cfg(test)]
mod test_permissions {
    use crate::stablecoin::setup_tests::setup;
    use odra::casper_types::U256;

    #[test]
    fn test_minter_permissions() {
        let (env, master_minter, controller_1, minter_1, .., user, mut stablecoin) = setup();
        env.set_caller(master_minter);
        stablecoin.configure_controller(&controller_1, &minter_1);
        assert!(
            env.emitted(&stablecoin, "ControllerConfigured"),
            "ControllerConfigured event not emitted"
        );
        env.set_caller(controller_1);
        stablecoin.configure_minter_allowance(U256::from(10));
        assert!(
            env.emitted(&stablecoin, "MinterConfigured"),
            "MinterConfigured event not emitted"
        );
        env.set_caller(minter_1);
        // try to mint legally, but exceed the allowance
        let result: Result<(), odra::OdraError> = stablecoin.try_mint(&user, U256::from(11));
        match result {
            Ok(_) => {
                panic!("Security Incident: Mint that exceeds the Allowance went through!")
            }
            _ => {}
        }
        // try to mint illegally
        env.set_caller(user);
        let result: Result<(), odra::OdraError> = stablecoin.try_mint(&user, U256::from(10));
        match result {
            Ok(_) => {
                panic!("Security Incident: Illegal mint went through!")
            }
            _ => {}
        }
        // remove the minter
        env.set_caller(controller_1);
        stablecoin.remove_minter();
        // try to mint with disabled minter
        env.set_caller(minter_1);
        let result: Result<(), odra::OdraError> = stablecoin.try_mint(&user, U256::from(10));
        match result {
            Ok(_) => {
                panic!("Security Incident: Illegal mint went through!")
            }
            _ => {}
        }
    }

    #[test]
    fn test_revoke_minter_and_controller() {
        let (env, master_minter, controller_1, minter_1, .., mut stablecoin) = setup();
        env.set_caller(master_minter);
        stablecoin.configure_controller(&controller_1, &minter_1);
        env.set_caller(controller_1);
        stablecoin.remove_minter();
        assert!(
            env.emitted(&stablecoin, "MinterRemoved"),
            "MinterRemoved event not emitted"
        );
        env.set_caller(master_minter);
        stablecoin.remove_controller(&controller_1);
        assert!(
            env.emitted(&stablecoin, "ControllerRemoved"),
            "ControllerRemoved event not emitted"
        );
    }

    #[test]
    fn must_not_mint_when_paused() {
        let (env, master_minter, controller_1, minter_1, .., pauser, user, mut stablecoin) =
            setup();
        env.set_caller(master_minter);
        stablecoin.configure_controller(&controller_1, &minter_1);
        env.set_caller(controller_1);
        stablecoin.configure_minter_allowance(U256::from(10));
        env.set_caller(pauser);
        stablecoin.pause();
        env.set_caller(minter_1);
        let result: Result<(), odra::OdraError> = stablecoin.try_mint(&user, U256::from(10));
        match result {
            Ok(_) => {
                panic!("Security Incident: Illegal mint went through!")
            }
            _ => {}
        }
    }
}
