//! Transport traits.
//!
//! This module defines the transport traits which specifies the behavior for
//! sending and receiving transaction data.

use crate::prelude::TransactionData;
use crate::Result;

/// The [`Sender`] trait specifies the behavior for sending transaction data.
pub trait Sender {
    fn send(&mut self) -> Result<()>;
}

/// The [`Receiver`] trait specifies the behavior for receiving transaction data.
pub trait Receiver {
    fn recv(&mut self) -> Result<TransactionData>;
}
