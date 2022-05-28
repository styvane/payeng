//! Account type
//!
//! This module module defines the transaction [`Account`] type.
//!

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use parking_lot::Mutex;
use rust_decimal::Decimal;
use serde::Serialize;

use super::{TransactionData, TransactionId, TransactionProcessor};
use crate::error::Error;
use crate::prelude::Result;

/// [`Account`] type. See module level [documentation](self).
#[derive(Debug, Default, Serialize)]
pub(crate) struct AccountData {
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,

    #[serde(skip_serializing)]
    histories: HashMap<TransactionId, Operation>,
}

/// The [`Operation`] type represents a recorded transaction operation.
#[derive(Debug, Clone)]
struct Operation {
    amount: Decimal,
    state: State,
}

// The [`State`] type represents the current state of a recorded transaction.
#[derive(Debug, Eq, PartialEq, Clone)]
enum State {
    Dispute,
    Resolve,
    Final,
    None,
}

impl AccountData {
    /// Updates transaction history.
    fn update_history(&mut self, id: TransactionId, amount: Decimal) {
        self.histories.insert(
            id,
            Operation {
                amount,
                state: State::None,
            },
        );
    }
}
/// [`Account`] represents a client account.
pub struct Account {
    state: Mutex<AccountData>,
}

impl Account {
    /// Creates new account.
    pub fn new() -> Self {
        Account {
            state: Mutex::new(AccountData::default()),
        }
    }
}

impl TransactionProcessor for Account {
    fn make_deposit(&mut self, transaction: TransactionData) -> Result<()> {
        let TransactionData { id, amount, .. } = transaction;
        // Deposit transactions are guarantee to have some amount due to validation.
        // So it's okay to unwrap the value here.
        let amount = amount.unwrap();
        let mut guard = self.state.lock();
        guard.total += amount;
        guard.available += amount;
        guard.update_history(id, amount);

        Ok(())
    }

    fn withdraw(&mut self, transaction: TransactionData) -> Result<()> {
        let TransactionData { id, amount, .. } = transaction;
        // Withdrawal transactions are guarantee to have some amount due to validation.
        // So it's okay to unwrap the value here.
        let amount = amount.unwrap();

        let mut guard = self.state.lock();
        if amount > guard.available {
            return Err(Error::WithdrawalError);
        }
        guard.total -= amount;
        guard.available -= amount;
        guard.update_history(id, amount);
        Ok(())
    }

    fn dispute(&mut self, tx_id: TransactionId) -> Result<()> {
        let mut guard = self.state.lock();
        match guard.histories.entry(tx_id) {
            Entry::Occupied(mut entry) => {
                let operation = entry.get_mut();
                operation.state = State::Dispute;
                let amount = operation.amount;
                guard.held += amount;
                guard.total += amount;
                Ok(())
            }
            _ => Err(Error::DisputeStateError),
        }
    }

    fn resolve(&mut self, tx_id: TransactionId) -> Result<()> {
        let mut guard = self.state.lock();
        match guard.histories.entry(tx_id) {
            Entry::Occupied(mut entry) if entry.get().state == State::Dispute => {
                let operation = entry.get_mut();
                operation.state = State::Resolve;
                let amount = operation.amount;
                guard.held -= amount;
                guard.available += amount;
                Ok(())
            }
            _ => Err(Error::DisputeStateError),
        }
    }

    fn charge_back(&mut self, tx_id: TransactionId) -> Result<()> {
        let mut guard = self.state.lock();
        match guard.histories.entry(tx_id) {
            Entry::Occupied(mut entry) if entry.get().state == State::Dispute => {
                let operation = entry.get_mut();
                operation.state = State::Final;
                let amount = operation.amount;
                guard.held -= amount;
                guard.total -= amount;
                guard.locked = true;
                Ok(())
            }
            _ => Err(Error::DisputeStateError),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::prelude::TransactionType::*;
    use crate::prelude::*;

    impl TransactionData {
        /// Creates new transaction data from specified values.
        fn from(id: u32, client: u16, amount: Option<f64>, tx_type: TransactionType) -> Self {
            let amount = amount.map(|amount| Decimal::try_from(amount).unwrap());

            Self {
                client: Client::from(client),
                tx_type,
                id: TransactionId::from(id),
                amount,
            }
        }
    }

    fn make_deposit() -> HashMap<Client, Account> {
        [
            TransactionData::from(1, 1, Some(3.5), Deposit),
            TransactionData::from(1, 2, Some(18.0), Deposit),
            TransactionData::from(2, 5, Some(15.1), Deposit),
            TransactionData::from(1, 7, Some(65.0), Deposit),
            TransactionData::from(1, 9, Some(47.1), Deposit),
            TransactionData::from(1, 11, Some(31.0), Deposit),
        ]
        .into_iter()
        .map(|tr| {
            let mut account = Account::new();
            let client = tr.client.clone();
            account.make_deposit(tr).unwrap();
            (client, account)
        })
        .collect()
    }

    /// Returns account state after deposit and withdrawal
    fn make_deposit_and_withdraw() -> HashMap<Client, Account> {
        let mut states = make_deposit();
        [
            TransactionData::from(2, 1, Some(1.5), Withdrawal),
            TransactionData::from(3, 2, Some(5.0), Withdrawal),
            TransactionData::from(4, 5, Some(10.1), Withdrawal),
            TransactionData::from(7, 7, Some(50.0), Withdrawal),
            TransactionData::from(18, 9, Some(27.1), Withdrawal),
            TransactionData::from(11, 11, Some(30.0), Withdrawal),
        ]
        .into_iter()
        .for_each(|tr| {
            if let Some(account) = states.get_mut(&tr.client) {
                account.withdraw(tr).unwrap();
            }
        });
        states
    }

    /// Returns account state after deposit, widthdrawal and dispute.
    fn make_deposit_withdraw_then_dispute() -> HashMap<Client, Account> {
        let mut accounts = make_deposit_and_withdraw();
        for (tx_id, client) in [(2, 1), (7, 7), (18, 9), (11, 11), (47, 1)]
            .map(|(tx_id, client)| (TransactionId::from(tx_id), Client::from(client)))
        {
            if let Some(account) = accounts.get_mut(&client) {
                match account.dispute(tx_id.clone()) {
                    Ok(_) => {
                        assert!(
                            account.state.lock().histories.contains_key(&tx_id),
                            "unrecognized transaction"
                        );
                    }
                    Err(_) => {
                        assert!(
                            !account.state.lock().histories.contains_key(&tx_id),
                            "transaction exists"
                        );
                    }
                }
            }
        }
        accounts
    }

    #[test]
    fn simulate_account_deposit_transactions() {
        let accounts = make_deposit();
        for (client, amount) in [
            (1, 3.5),
            (2, 18.0),
            (5, 15.1),
            (7, 65.0),
            (9, 47.1),
            (11, 31.0),
        ]
        .map(|(client, amount)| (Client::from(client), Decimal::try_from(amount).unwrap()))
        {
            let guard = accounts[&client].state.lock();
            assert_eq!(
                guard.total,
                amount,
                "total is not the expected amount for client #{}",
                client.inner_ref()
            );
            assert!(
                guard.held.is_zero(),
                "held is not the expected amount for client #{}",
                client.inner_ref()
            );
            assert_eq!(
                guard.available,
                amount,
                "available amount is different for client #{}",
                client.inner_ref()
            );
            assert!(
                !guard.locked,
                "lock state is wrong for client #{}",
                client.inner_ref()
            );
        }
    }

    #[test]
    fn simulate_account_deposit_follow_by_withdrawal() {
        let accounts = make_deposit_and_withdraw();
        for (id, total) in [
            (1, 2.0),
            (2, 13.0),
            (5, 5.0),
            (7, 15.0),
            (9, 20.0),
            (11, 1.0),
        ]
        .map(|(id, total)| (Client::from(id), Decimal::try_from(total).unwrap()))
        {
            let guard = accounts[&id].state.lock();
            assert_eq!(
                guard.total,
                total,
                "total is not the expected amount for client #{}",
                id.inner_ref()
            );
            assert!(
                guard.held.is_zero(),
                "held is not the expected amount for client #{}",
                id.inner_ref()
            );
            assert_eq!(
                guard.available,
                total,
                "available amount is different for client #{}",
                id.inner_ref()
            );
            assert!(
                !guard.locked,
                "lock state is wrong for client #{}",
                id.inner_ref()
            );
        }
    }

    #[test]

    fn cannot_withdraw_more_funds_than_available() {
        let mut accounts = make_deposit();
        let transaction = TransactionData::from(7, 7, Some(2000.0), Withdrawal);
        let client = transaction.client.clone();
        match accounts.get_mut(&client).unwrap().withdraw(transaction) {
            Ok(_) => panic!("should never withdraw more fund than available."),
            Err(e) => assert!(matches!(e, Error::WithdrawalError)),
        }
    }

    #[test]
    fn simulate_account_deposit_follow_by_withdrawal_and_dispute() {
        let accounts = make_deposit_withdraw_then_dispute();
        for (client, total, held, available) in [
            // client, total, held, available
            (1, 3.5, 1.5, 2.0),
            (7, 65.0, 50.0, 15.0),
            (9, 47.1, 27.1, 20.0),
            (11, 31.0, 30.0, 1.0),
        ]
        .map(|(client, total, held, available)| {
            (
                Client::from(client),
                Decimal::try_from(total).unwrap(),
                Decimal::try_from(held).unwrap(),
                Decimal::try_from(available).unwrap(),
            )
        }) {
            let guard = accounts[&client].state.lock();
            assert_eq!(
                guard.total,
                total,
                "total is not the expected amount for client #{}",
                client.inner_ref()
            );
            assert_eq!(
                guard.held,
                held,
                "held is not the expected amount for client #{}",
                client.inner_ref()
            );
            assert_eq!(
                guard.available,
                available,
                "available amount is different for client #{}",
                client.inner_ref()
            );
            assert!(
                !guard.locked,
                "lock state is wrong for client #{}",
                client.inner_ref()
            );
        }
    }

    #[test]
    fn simulate_account_deposit_follow_by_withdrawal_dispute_and_resolve() {
        let mut accounts = make_deposit_withdraw_then_dispute();

        for (tx_id, client) in [(2, 1), (18, 9), (71, 11)]
            .map(|(tx_id, client)| (TransactionId::from(tx_id), Client::from(client)))
        {
            if let Some(account) = accounts.get_mut(&client) {
                match account.resolve(tx_id.clone()) {
                    Ok(_) => {
                        let guard = account.state.lock();
                        assert!(
                            guard.histories.contains_key(&tx_id),
                            "unrecognized transaction fro client: #{}",
                            client.inner_ref()
                        );
                        assert!(
                            !guard.locked,
                            "lock state is wrong for client: #{}",
                            client.inner_ref()
                        );
                    }
                    Err(_) => {
                        if let Some(operation) = account.state.lock().histories.get(&tx_id) {
                            assert_ne!(
                                operation.state,
                                State::Dispute,
                                "not disputed transaction: #{}",
                                client.inner_ref()
                            );
                        }
                    }
                }
            }
        }

        for (client, total, held, available) in [
            // client, total, held, available
            (1, 3.5, 0.0, 3.5),
            (7, 65.0, 50.0, 15.0),
            (9, 47.1, 0.0, 47.1),
            (11, 31.0, 30.0, 1.0),
        ]
        .map(|(client, total, held, available)| {
            (
                Client::from(client),
                Decimal::try_from(total).unwrap(),
                Decimal::try_from(held).unwrap(),
                Decimal::try_from(available).unwrap(),
            )
        }) {
            let guard = accounts[&client].state.lock();
            assert_eq!(
                guard.total,
                total,
                "total is not the expected amount for client #{}",
                client.inner_ref()
            );
            assert_eq!(
                guard.held,
                held,
                "held is not the expected amount for client: #{}",
                client.inner_ref()
            );
            assert_eq!(
                guard.available,
                available,
                "available amount is different for client: #{}",
                client.inner_ref()
            );
        }
    }

    #[test]
    fn simulate_account_deposit_follow_by_withdrawal_dispute_and_charge_back() {
        let mut accounts = make_deposit_withdraw_then_dispute();
        for (tx_id, client) in [(2, 1), (18, 9), (71, 11)]
            .map(|(tx_id, client)| (TransactionId::from(tx_id), Client::from(client)))
        {
            if let Some(account) = accounts.get_mut(&client) {
                match account.charge_back(tx_id.clone()) {
                    Ok(_) => {
                        let guard = account.state.lock();
                        assert!(
                            guard.histories.contains_key(&tx_id),
                            "unrecognized transaction for client: #{}",
                            client.inner_ref()
                        );
                        assert!(
                            guard.locked,
                            "lock state is wrong for client: #{}",
                            client.inner_ref()
                        );
                    }
                    Err(_) => {
                        if let Some(operation) = account.state.lock().histories.get(&tx_id) {
                            assert_ne!(
                                operation.state,
                                State::Dispute,
                                "not disputed transaction: #{}",
                                client.inner_ref()
                            );
                        }
                    }
                }
            }
        }

        for (client, total, held, available) in [
            // client, total, held, available
            (1, 2.0, 0.0, 2.0),
            (7, 65.0, 50.0, 15.0),
            (9, 20.0, 0.0, 20.0),
            (11, 31.0, 30.0, 1.0),
        ]
        .map(|(client, total, held, available)| {
            (
                Client::from(client),
                Decimal::try_from(total).unwrap(),
                Decimal::try_from(held).unwrap(),
                Decimal::try_from(available).unwrap(),
            )
        }) {
            let guard = accounts[&client].state.lock();
            assert_eq!(
                guard.total,
                total,
                "total is not the expected amount for client #{}",
                client.inner_ref()
            );
            assert_eq!(
                guard.held,
                held,
                "held is not the expected amount for client: #{}",
                client.inner_ref()
            );
            assert_eq!(
                guard.available,
                available,
                "available amount is different for client: #{}",
                client.inner_ref()
            );
        }
    }
}
