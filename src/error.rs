#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Awaitable is already consumed but not yet reset.
    #[error("Awaitable is already consumed but not yet reset.")]
    AlreadyConsumed,

    /// Awaitable is marked done twice.
    #[error("Awaitable is marked done twice.")]
    AlreadyDone,

    /// Waker is alreayd installed
    #[error("Waker is already installed")]
    WakerAlreadyInstalled,
}
