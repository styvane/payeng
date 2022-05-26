//! Result type.
//!
//! This module defines an alias for the[`Result`] type with [`payeng::Error`] as `Error`.

use crate::error::Error;

/// Alias for the Result type.
pub type Result<T> = std::result::Result<T, Error>;
