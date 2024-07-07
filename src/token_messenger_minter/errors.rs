/// Error enum for the TokenMessengerMinter contract.
#[odra::odra_error]
pub enum Error {
    InsufficientRights = 40000,
    ContractIsPaused = 40001,
}
