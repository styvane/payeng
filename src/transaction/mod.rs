//! Transaction module.
//!
//! This module defines the data structures which represents different part of
//! a transaction.
//!
mod transaction_data;
mod transaction_type;

pub use transaction_data::{Transaction, TransactionId};
pub use transaction_type::TransactionType;
