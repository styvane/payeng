use std::io;

use crossbeam::channel;
use csv::WriterBuilder;

use crate::error::Error;
use crate::prelude::{Account, AccountManager, AccountRegistry, TransactionData};
use crate::transport::Receiver;
use crate::{transport, Result};

use super::TransactionType;

/// A summary of transaction writer configured with the underline
/// CSV writer.
pub struct Writer<W: io::Write> {
    writer: csv::Writer<W>,
    registry: AccountRegistry,
    incoming_transaction: channel::Receiver<TransactionData>,
}

impl<W> Writer<W>
where
    W: io::Write,
{
    /// Creates new [`Writer`].
    #[tracing::instrument(name = "Create writer", skip(writer, incoming_transaction))]
    pub fn from_writer(
        writer: W,
        incoming_transaction: channel::Receiver<TransactionData>,
    ) -> Self {
        let writer = WriterBuilder::new().from_writer(writer);
        Self {
            writer,
            incoming_transaction,
            registry: AccountRegistry::default(),
        }
    }

    #[tracing::instrument(name = "Process transaction", skip(self))]
    pub fn process_transaction(&mut self) {
        loop {
            match self.recv() {
                Ok(data) => {
                    let account = self.registry.get_mut_or_insert(data.client.clone());
                    if let Err(err) = match data.tx_type {
                        TransactionType::Deposit => account.make_deposit(data),
                        TransactionType::Withdrawal => account.withdraw(data),
                        TransactionType::Dispute => account.dispute(data.id),
                        TransactionType::Resolve => account.resolve(data.id),
                        TransactionType::ChargeBack => account.charge_back(data.id),
                    } {
                        tracing::error!(err.cause_chain=?err)
                    }
                }
                Err(err) => {
                    tracing::error!(err.cause_chain=?err);
                }
            }
        }
    }
}

impl<W> transport::Receiver for Writer<W>
where
    W: io::Write,
{
    #[tracing::instrument(name = "Receive transaction", skip(self))]
    fn recv(&mut self) -> Result<TransactionData> {
        self.incoming_transaction.recv().map_err(Error::RecvError)
    }
}
