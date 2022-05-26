//! Client type.
//!
//! This module defines the [`Client`] data structure and associated operations.
//!

use serde::Deserialize;
/// [`Client`] type. See module level [documentation](self).
#[derive(Debug, Clone, Deserialize)]
pub struct Client(u32);

impl Client {
    /// Creates new client instance.
    pub fn from(id: u32) -> Self {
        Client(id)
    }
}
