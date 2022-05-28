//! AccountManager trait
//!
//! This module defines the `TransactionManager` which specify the behavior
//! of an account manager.
//!

use crate::prelude::Result;

use super::{TransactionData, TransactionId};

pub trait TransactionProcessor {
    fn make_deposit(&mut self, transaction: TransactionData) -> Result<()>;
    fn withdraw(&mut self, transaction: TransactionData) -> Result<()>;
    fn dispute(&mut self, transaction_id: TransactionId) -> Result<()>;
    fn resolve(&mut self, transaction_id: TransactionId) -> Result<()>;
    fn charge_back(&mut self, transaction_id: TransactionId) -> Result<()>;
}
