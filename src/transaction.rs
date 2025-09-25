use serde::{Deserialize, Serialize, Serializer};
use std::fmt;
use strum::EnumIter;

/// Representation of all transaction variants.
#[derive(Debug, Deserialize, Serialize, EnumIter, Clone, Copy)]
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

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {:.4}",
            self.tx_type, self.client, self.tx, self.amount
        )
    }
}

fn serialize_up_to_four_decimal_places<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // The assignment spec notes that we must accept transaction amounts with
    // up to 4 decimal places of precison. This serializer will handle that.
    //
    // Format the float to a string with 4 decimal places before serializing.
    let formatted = format!("{:.4}", x);
    s.serialize_str(&formatted)
}
