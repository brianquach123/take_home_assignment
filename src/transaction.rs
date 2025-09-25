/// This file defines the `Transaction` struct and associated methods and utilities
/// for it in the payments engine.
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use strum::EnumIter;

/// Representation of all transaction variants supported.
#[derive(Debug, Deserialize, Serialize, EnumIter, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")] // Sample tx files have lowercase tx types
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TransactionType::Deposit => {
                write!(f, "deposit")
            }
            TransactionType::Withdrawal => {
                write!(f, "withdrawal")
            }
            TransactionType::Dispute => {
                write!(f, "dispute")
            }
            TransactionType::Resolve => {
                write!(f, "resolve")
            }
            TransactionType::Chargeback => {
                write!(f, "chargeback")
            }
        }
    }
}

/// Representation of a transaction.
#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub struct Transaction {
    /// Type of Transaction.
    pub tx_type: TransactionType,
    /// Client ID.
    pub client: u16,
    /// Transaction ID. Assumed type from assignment spec.
    pub tx: u32,
    /// Transaction amount. Assumed type from assignment spec.
    #[serde(serialize_with = "serialize_up_to_four_decimal_places")]
    pub amount: f64,
}

/// Output formatting for a transaction, based on the spec doc.
impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {:.4}",
            self.tx_type, self.client, self.tx, self.amount
        )
    }
}

/// Custom serializer function for floats. The spec doc states that
/// decimal precisions are assumed to be up to four places and should
/// output values with the same level of precison. This function handles
/// that decismal precision for output.
fn serialize_up_to_four_decimal_places<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let formatted = format!("{:.4}", x);
    s.serialize_str(&formatted)
}
