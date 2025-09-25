use std::{
    collections::{BTreeSet, HashMap},
    fmt,
};

use crate::transaction::TransactionType;

/// Representation of a client's account in the payments engine.
/// A client account is defined by its funds' details and lock status,
/// in addition to the set of transactions ID associated with this client
/// account that the payments engine has previously processed.
#[derive(Debug, Default)]
pub struct ClientAccount {
    /// Balance details and lock status for this account.
    pub account_details: ClientAccountDetails,
    /// Transaction history and details for this account.
    pub account_transaction_archive: ClientTransactionArchive,
}

#[derive(Debug, Default)]
pub struct ClientTransactionArchive {
    /// The set of transaction IDs associated with this account.
    pub history: BTreeSet<u32>,
    /// Map of the set of transaction IDs to (amount, type of transaction)
    /// for this account.
    pub details: HashMap<u32, (f64, TransactionType)>,
    /// The set of disputed transactions for this account.
    pub disputes: BTreeSet<u32>,
}

/// Representation of a client's account details in the engine.
/// The engine uses this for reporting output to stdout.
#[derive(Debug, Default)]
pub struct ClientAccountDetails {
    pub available_funds: f64,
    pub held_funds: f64,
    pub total_funds: f64,
    pub is_account_locked: bool,
}

impl fmt::Display for ClientAccountDetails {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:.4}, {:.4}, {:.4}, {}",
            self.available_funds, self.held_funds, self.total_funds, self.is_account_locked
        )
    }
}
