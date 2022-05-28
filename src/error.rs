//! Error type.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid transaction record")]
    InvalidTransaction,

    #[error("insufficient available funds")]
    WithdrawalError,

    #[error("non disputed transaction")]
    DisputeStateError,
}
