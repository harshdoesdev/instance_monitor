use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to bind the listener")]
    ListenerBindError,

    #[error("Failed to start the server")]
    ServerStartError,

    #[error("Internal server error")]
    InternalError,
}
