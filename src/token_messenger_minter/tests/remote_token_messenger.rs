#[cfg(test)]
mod install_message_transmitter {
    use crate::token_messenger_minter::setup_tests::setup;

    #[test]
    fn test_set_remote_token_messenger() {
        let (_env, mut token_messenger_minter) = setup();
        token_messenger_minter.add_remote_token_messenger([0; 32]);
        token_messenger_minter.remove_remote_token_messenger([0; 32]);
    }
}
