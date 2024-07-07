/// Error enum for the MessageTransmitter contract.
#[odra::odra_error]
pub enum Error {
    InsufficientRights = 50000,
    ContractIsPaused = 50001,
}
