/// This file defines the payments engine interface and behavior
/// for processing a deserialized `Transaction`.
use anyhow::Result;
use std::collections::HashMap;
use std::fmt;

use crate::account::ClientAccount;
use crate::errors::PaymentsTransactionError;
use crate::transaction::{Transaction, TransactionType};

/// Representation of the payments engine.
#[derive(Debug)]
pub struct PaymentsEngine {
    pub client_account_lookup: HashMap<u16, ClientAccount>,
}

impl fmt::Display for PaymentsEngine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "client, available, held, total, locked")?;
        for (client_id, client_account) in &self.client_account_lookup {
            writeln!(f, "{}, {}", client_id, client_account.account_details)?;
        }
        Ok(())
    }
}

impl PaymentsEngine {
    /// Processes a `Transaction` based on its `TransactionType`.
    pub fn process_transaction(&mut self, tx: Transaction) -> Result<(), PaymentsTransactionError> {
        // First check if this client ID has been seen before. If not, create
        // a new client account. Then get a mutable reference to the underlying
        // `ClientAccount` for transaction processing.
        let selected_account = self.client_account_lookup.entry(tx.client).or_default();

        // Ignore duplicate transaction IDs that have been seen before.
        let transaction_id_not_seen_before = !selected_account
            .account_transaction_archive
            .history
            .contains(&tx.tx);
        if transaction_id_not_seen_before {
            match tx.tx_type {
                TransactionType::Deposit => selected_account.handle_deposit(tx)?,
                TransactionType::Withdrawal => {
                    // If a client doesn't have enough funds, a withdrawal will fail.
                    // Operationally, isn't of stopping, we'll ack the erroneous withdrawal
                    // in a log and ingore it with a .ok() and continue processing other transactions.
                    selected_account.handle_withdrawal(tx).ok();
                    return Ok(());
                }
                TransactionType::Dispute => selected_account.handle_dispute(tx)?,
                TransactionType::Resolve => selected_account.handle_resolve(tx)?,
                TransactionType::Chargeback => selected_account.handle_chargeback(tx)?,
            }
        }
        Ok(())
    }
}
