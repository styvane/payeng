//! Error type.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid transaction record")]
    InvalidTransaction,

    #[error("insufficient available funds")]
    WithdrawalError,

    #[error("non disputed transaction")]
    DisputeStateError,

    #[error("failed to send transaction: {0}")]
    SendError(String),

    #[error(transparent)]
    RecvError(#[from] crossbeam::channel::RecvError),

    #[error(transparent)]
    CsvError(#[from] csv::Error),

    #[error(transparent)]
    TracerError(#[from] tracing::subscriber::SetGlobalDefaultError),
    #[error("account already exists")]
    AccountError,

    #[error(transparent)]
    IoError(std::io::Error),

    #[error("expected 1 argument, found none")]
    InvalidArgumentError,
}
