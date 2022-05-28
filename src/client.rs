//! Client type.
//!
//! This module defines the [`Client`] data structure and associated operations.
//!

use serde::Deserialize;
/// [`Client`] type. See module level [documentation](self).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize)]
pub struct Client(pub(crate) u16);

impl Client {
    /// Creates new client instance.
    pub fn from(id: u16) -> Self {
        Client(id)
    }

    #[cfg(test)]
    pub(crate) fn inner_ref(&self) -> &u16 {
        &self.0
    }
}
