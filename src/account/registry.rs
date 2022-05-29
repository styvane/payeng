//! Account registry.
//!
//! This module defines the account registry type which is a collection of all the accounts
//! in the system.

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use super::Account;
use crate::prelude::Client;

/// [`AccountRegistry]` type. See module level [documentation](self).
#[derive(Default)]
pub struct AccountRegistry(HashMap<Client, Account>);

impl AccountRegistry {
    /// Creates new account registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns an mutable reference to the client account.
    /// If the account is not present, insert a new account and returns the reference
    /// to new inserted account.
    pub fn get_mut_or_insert(&mut self, client: Client) -> &mut Account {
        match self.0.entry(client.clone()) {
            Entry::Occupied(account) => account.into_mut(),
            Entry::Vacant(e) => e.insert(Account::new(&client)),
        }
    }

    /// Returns an iterator over the accounts in the registry.
    pub fn iter(&self) -> impl Iterator<Item = &Account> {
        self.0.values()
    }
}
