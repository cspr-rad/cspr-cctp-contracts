#[cfg(test)]
mod install_message_transmitter {
    use crate::message_transmitter::setup_tests::setup;

    #[test]
    fn test_stablecoin_mint() {
        let (env, message_transmitter) = setup();
    }
}
