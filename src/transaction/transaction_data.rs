//! Transaction data types.
//!
//! This modules defines various transaction data types.
//!
use rust_decimal::Decimal;
use serde::Deserialize;

use super::{TransactionId, TransactionType};
use crate::error::Error;
use crate::prelude::Client;

/// The [`Transaction`] type represents a single transaction.
#[derive(Debug, Deserialize)]
#[serde(try_from = "RawTransactionData")]
pub struct TransactionData {
    pub(crate) client: Client,

    #[serde(rename(deserialize = "type"))]
    pub(crate) tx_type: TransactionType,

    #[serde(rename(deserialize = "tx"))]
    pub(crate) id: TransactionId,

    pub(crate) amount: Option<Decimal>,
}

/// [`RawTransaction`] represents non validated transaction.
#[derive(Clone, Debug, Deserialize)]

struct RawTransactionData {
    client: u16,
    #[serde(rename(deserialize = "type"))]
    tx_type: TransactionType,
    #[serde(rename(deserialize = "tx"))]
    id: u32,
    amount: Option<Decimal>,
}

impl TryFrom<RawTransactionData> for TransactionData {
    type Error = Error;
    fn try_from(raw: RawTransactionData) -> Result<Self, Self::Error> {
        let RawTransactionData {
            client,
            tx_type,
            id,
            amount,
        } = raw;
        let transaction = TransactionData {
            client: Client::from(client),
            tx_type,
            id: TransactionId::from(id),
            amount,
        };
        match transaction.tx_type {
            TransactionType::Deposit | TransactionType::Withdrawal if amount.is_none() => {
                Err(Error::InvalidTransaction)
            }
            TransactionType::ChargeBack | TransactionType::Dispute | TransactionType::Resolve
                if amount.is_some() =>
            {
                Err(Error::InvalidTransaction)
            }
            _ => Ok(transaction),
        }
    }
}

#[cfg(test)]
mod tests {

    use csv::{ByteRecord, ReaderBuilder, Trim};
    use itertools::Itertools;
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;
    use serde::Deserialize;

    use super::TransactionType::*;
    use super::*;

    const TR_KIND: [TransactionType; 5] = [Deposit, Withdrawal, Dispute, Resolve, ChargeBack];

    #[derive(Deserialize, Clone, Debug)]
    struct ValidData(String);

    impl Arbitrary for ValidData {
        fn arbitrary(g: &mut Gen) -> Self {
            let client = u16::arbitrary(g);
            let kind = g.choose(&TR_KIND).unwrap().clone();
            let tx = u32::arbitrary(g);
            let amount = match kind {
                Deposit | Withdrawal => {
                    let amount = u32::arbitrary(g) as f64 * 0.0001;
                    Decimal::try_from(amount).unwrap().to_string()
                }
                _ => String::default(),
            };

            Self(format!("{kind},{client},{tx},{amount}"))
        }
    }

    #[derive(Deserialize, Clone, Debug)]
    struct InvalidData(String);

    impl Arbitrary for InvalidData {
        fn arbitrary(g: &mut Gen) -> Self {
            let client = u16::arbitrary(g);
            let kind = g.choose(&TR_KIND).unwrap().clone();
            let tx = u32::arbitrary(g);
            let amount = match kind {
                Deposit | Withdrawal => String::default(),
                _ => {
                    let amount = u32::arbitrary(g) as f64 * 0.0001;
                    Decimal::try_from(amount).unwrap().to_string()
                }
            };

            Self(format!("{kind},{client},{tx},{amount}"))
        }
    }

    fn check_record(data: String) {
        let headers = "type, client, tx, amount\n";
        let data = format!("{headers}\n{data}");
        let mut raw = ByteRecord::new();

        let mut reader = ReaderBuilder::new()
            .flexible(true)
            .trim(Trim::All)
            .from_reader(data.as_bytes());
        let headers = reader.byte_headers().cloned().unwrap();

        while reader.read_byte_record(&mut raw).unwrap() {
            let _: TransactionData = raw.deserialize(Some(&headers)).unwrap();
        }
    }

    #[quickcheck]
    fn can_serialize_a_valid_transaction(data: Vec<ValidData>) {
        let data = data.into_iter().map(|d| d.0).join("\n");
        check_record(data);
    }

    #[quickcheck]
    #[should_panic]
    fn fail_to_serialize_a_invalid_transaction(data: Vec<InvalidData>) {
        let data = data.into_iter().map(|d| d.0).join("\n");
        //println!("{data:?}");
        check_record(data);
    }
}
