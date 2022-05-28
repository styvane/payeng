use std::io;

use crossbeam::channel::Sender;
use csv::{ByteRecord, ReaderBuilder, Trim};

use crate::error::Error;
use crate::prelude::TransactionData;
use crate::{transport, Result};

/// A transaction reader configured with the underline csv reader.
///
/// We can only construct this struct using the [`from_reader`] method.
#[derive(Debug)]
pub struct Reader<R> {
    reader: csv::Reader<R>,
    outgoing_transaction: Sender<TransactionData>,
}

impl<R> Reader<R>
where
    R: io::Read,
{
    /// Creates new [`Reader`] with the underline reader.
    pub fn from_reader(reader: R, outgoing_transaction: Sender<TransactionData>) -> Self {
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
