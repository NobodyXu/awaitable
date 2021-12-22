#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Awaitable is not initialized yet.
    #[error("Awaitable is not initialized yet.")]
    Uninitialized,

    /// Awaitable is already consumed but not yet reset.
    #[error("Awaitable is already consumed but not yet reset.")]
    AlreadyConsumed,

    /// Awaitable is marked done twice.
    #[error("Awaitable is marked done twice.")]
    AlreadyDone,
}
