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
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    /// Client ID.
    pub client: u16,
    /// Transaction ID. Assumed type from assignment spec.
    pub tx: u32,
    /// Transaction amount. Assumed type from assignment spec.
    #[serde(serialize_with = "serialize_up_to_four_decimal_places", default)]
    pub amount: Option<f64>,
}

/// Output formatting for a transaction, based on the spec doc.
impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {:.4}",
            self.tx_type,
            self.client,
            self.tx,
            self.amount.unwrap()
        )
    }
}

/// Custom serializer function for floats. The spec doc states that
/// decimal precisions are assumed to be up to four places and should
/// output values with the same level of precison. This function handles
/// that decismal precision for output.
fn serialize_up_to_four_decimal_places<S>(x: &Option<f64>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(val) => {
            // Format to 4 decimals, then trim trailing zeros and dot
            let formatted = format!("{:.4}", val);
            let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
            let result = if trimmed.is_empty() { "0" } else { trimmed };
            s.serialize_str(result)
        }
        None => s.serialize_str(""), // serialize None as empty string
    }
}

#[cfg(test)]
mod transaction_serialization_tests {
    use super::*;
    use serde::Serialize;
    use serde_json;

    #[derive(Serialize)]
    struct TestStruct<'a> {
        #[serde(serialize_with = "serialize_up_to_four_decimal_places")]
        value: &'a Option<f64>,
    }

    #[test]
    fn test_serialize_some_rounding_more_than_four_decimals() {
        let x = Some(3.14159265);
        let wrapper = TestStruct { value: &x };
        let serialized = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(serialized, r#"{"value":"3.1416"}"#);
    }

    #[test]
    fn test_serialize_some_exact_four_decimals() {
        let x = Some(2.7182);
        let wrapper = TestStruct { value: &x };
        let serialized = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(serialized, r#"{"value":"2.7182"}"#);
    }

    #[test]
    fn test_serialize_some_fewer_than_four_decimals() {
        let x = Some(1.5);
        let wrapper = TestStruct { value: &x };
        let serialized = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(serialized, r#"{"value":"1.5"}"#);
    }

    #[test]
    fn test_serialize_some_integer() {
        let x = Some(100.0);
        let wrapper = TestStruct { value: &x };
        let serialized = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(serialized, r#"{"value":"100"}"#);
    }

    #[test]
    fn test_serialize_some_small_decimal() {
        let x = Some(0.0001);
        let wrapper = TestStruct { value: &x };
        let serialized = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(serialized, r#"{"value":"0.0001"}"#);
    }

    #[test]
    fn test_serialize_some_zero() {
        let x = Some(0.0);
        let wrapper = TestStruct { value: &x };
        let serialized = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(serialized, r#"{"value":"0"}"#);
    }

    #[test]
    fn test_serialize_some_four_decimals_with_trailing_zeros() {
        let x = Some(2.5000);
        let wrapper = TestStruct { value: &x };
        let serialized = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(serialized, r#"{"value":"2.5"}"#);
    }

    #[test]
    fn test_serialize_none() {
        let x: Option<f64> = None;
        let wrapper = TestStruct { value: &x };
        let serialized = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(serialized, r#"{"value":""}"#); // None serializes as empty string
    }
}
