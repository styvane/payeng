//! Transaction ID type.
//!

use serde::Deserialize;

/// The [`TransactionId`] type is a unique ID associated to each transaction.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct TransactionId(u32);

impl TransactionId {
    /// Creates new [`TransactionId`] with given `id`.
    pub fn from(id: u32) -> Self {
        Self(id)
    }

    /// Returns a reference to the inner transaction id value.
    pub fn inner_ref(&self) -> &u32 {
        &self.0
    }
}
