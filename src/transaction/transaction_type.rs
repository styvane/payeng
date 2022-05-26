//! Transaction operation type.
//!
//! This module defines the different operations for a transaction lifecycle.
//!

use std::fmt;

use serde::{Deserialize, Serialize};

/// [`TransactionType`] is a type that represents the different possible operations.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    ChargeBack,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Deposit => "deposit",
            Self::Withdrawal => "withdrawal",
            Self::Dispute => "dispute",
            Self::Resolve => "resolve",
            Self::ChargeBack => "chargeback",
        };
        write!(f, "{}", value)
    }
}
