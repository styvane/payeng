//! Error type.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid transaction record")]
    InvalidTransaction,
}
