//! Account manager trait.
//!

use super::{TransactionData, TransactionId};
use crate::prelude::Result;

/// [`AccountManager`] specifies the behavior of an account manager.
pub trait AccountManager {
    fn make_deposit(&mut self, transaction: TransactionData) -> Result<()>;
    fn withdraw(&mut self, transaction: TransactionData) -> Result<()>;
    fn dispute(&mut self, transaction_id: TransactionId) -> Result<()>;
    fn resolve(&mut self, transaction_id: TransactionId) -> Result<()>;
    fn charge_back(&mut self, transaction_id: TransactionId) -> Result<()>;
}
