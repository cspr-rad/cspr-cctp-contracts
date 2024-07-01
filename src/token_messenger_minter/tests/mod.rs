#[cfg(test)]
mod install_message_transmitter {
    use crate::token_messenger_minter::setup_tests::setup;

    #[test]
    fn test_install() {
        let (_env, _token_messenger_minter) = setup();
    }
}
