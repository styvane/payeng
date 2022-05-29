use std::io;

use crossbeam::channel;
use csv::{ByteRecord, ReaderBuilder, Trim, WriterBuilder};

use crate::error::Error;
use crate::prelude::{AccountManager, AccountRegistry, TransactionData};
use crate::transport::{self, Receiver};
use crate::Result;

use super::TransactionType;
/// A transaction reader configured with the underline csv reader.
///
/// We can only construct this struct using the [`from_reader`] method.
#[derive(Debug)]
pub struct Reader<R> {
    reader: csv::Reader<R>,
    outgoing_transaction: channel::Sender<TransactionData>,
}

/// A summary of transaction writer configured with the underline
/// CSV writer.
pub struct Writer<W: io::Write> {
    writer: csv::Writer<W>,
    registry: AccountRegistry,
    incoming_transaction: channel::Receiver<TransactionData>,
}

impl<R> Reader<R>
where
    R: io::Read,
{
    /// Creates new [`Reader`] with the underline reader.
    pub fn from_reader(reader: R, outgoing_transaction: channel::Sender<TransactionData>) -> Self {
        let reader = ReaderBuilder::new()
            .flexible(true)
            .trim(Trim::All)
            .from_reader(reader);

        Self {
            reader,
            outgoing_transaction,
        }
    }
}

impl<R> transport::Sender for Reader<R>
where
    R: io::Read,
{
    #[tracing::instrument(name = "send transaction", skip(self))]
    fn send(&mut self) -> Result<()> {
        let headers = self.reader.byte_headers()?.clone();
        let mut record = ByteRecord::new();

        loop {
            match self.reader.read_byte_record(&mut record) {
                Ok(has_more) => {
                    if !has_more {
                        break;
                    }
                    match record.deserialize::<TransactionData>(Some(&headers)) {
                        Ok(data) => {
                            self.outgoing_transaction
                                .send(data)
                                .map_err(|e| Error::SendError(e.to_string()))?;
                        }
                        Err(err) => tracing::error!(err.cause_chain = ?err),
                    };
                }
                Err(err) => tracing::error!(err.cause_chain = ?err),
            }
        }

        Ok(())
    }
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

    #[tracing::instrument(name = "write account report", skip(self))]
    pub fn write(&mut self) {
        self.process_transaction();
        for account in self.registry.iter() {
            if let Err(err) = self.writer.serialize(&*account.inner_ref().lock()) {
                tracing::error!(err.cause_chain=?err);
            }
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
                    break;
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
