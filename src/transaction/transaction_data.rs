//! Transaction types.
//!
//! This modules defines various transaction types.
//!
use rust_decimal::Decimal;
use serde::Deserialize;

use super::TransactionType;
use crate::error::Error;
use crate::prelude::Client;

/// The [`Transaction`] type represents a single transaction.
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "RawTransaction")]
pub struct Transaction {
    client: Client,

    #[serde(rename(deserialize = "type"))]
    type_: TransactionType,

    #[serde(rename(deserialize = "tx"))]
    id: TransactionId,

    amount: Option<Decimal>,
}

/// [`RawTransaction`] represents non validated transaction.
#[derive(Clone, Debug, Deserialize)]

struct RawTransaction {
    client: u32,
    #[serde(rename(deserialize = "type"))]
    type_: TransactionType,
    #[serde(rename(deserialize = "tx"))]
    id: u32,
    amount: Option<Decimal>,
}

/// The [`TransactionId`] type is a unique ID associated to each transaction.
#[derive(Clone, Debug, Deserialize)]
pub struct TransactionId(u32);

impl TryFrom<RawTransaction> for Transaction {
    type Error = Error;
    fn try_from(raw: RawTransaction) -> Result<Self, Self::Error> {
        let RawTransaction {
            client,
            type_,
            id,
            amount,
        } = raw;
        let transaction = Transaction {
            client: Client::from(client),
            type_,
            id: TransactionId(id),
            amount,
        };
        match transaction.type_ {
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
            let _: Transaction = raw.deserialize(Some(&headers)).unwrap();
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
