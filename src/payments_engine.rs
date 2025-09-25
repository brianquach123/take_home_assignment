use anyhow::Result;
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

use crate::account::ClientAccount;
use crate::transaction::{Transaction, TransactionType};

#[derive(Debug, Error)]
pub enum PaymentsTransactionError {
    #[error("Not enough available funds for client {0}")]
    NotEnoughAvailableFunds(String),
    #[error("Transaction details not found for transaction {0}")]
    TransactionDetailDoesNotExist(String),
}

/// Representation of the payments engine.
#[derive(Debug, Default)]
pub struct PaymentsEngine {
    /// Maps a client's ID to their `ClientAccount`.
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
    /// Processes a `Transaction`` based on its `TransactionType``.
    pub fn process_transaction(&mut self, tx: Transaction) -> Result<(), PaymentsTransactionError> {
        // First check if this client ID has been seen before. If not, create
        // a new client account. Then get a mutable reference to the underlying
        // `ClientAccount` for transaction processing.
        let selected_account = self.client_account_lookup.entry(tx.client).or_default();

        // Ignore duplicate transaction IDs that have been seen before.
        if !selected_account
            .account_transaction_archive
            .history
            .contains(&tx.tx)
        {
            match tx.tx_type {
                TransactionType::Deposit => {
                    // A deposit is a credit to the client's asset account, meaning it
                    // should increase the available and total funds of the client account.
                    selected_account.account_details.available_funds += tx.amount;
                    selected_account.account_details.total_funds += tx.amount;

                    // Update transaction lookup/history
                    selected_account
                        .account_transaction_archive
                        .details
                        .insert(tx.tx, (tx.amount, tx.tx_type));
                    selected_account
                        .account_transaction_archive
                        .history
                        .insert(tx.tx);
                    return Ok::<(), PaymentsTransactionError>(());
                }
                TransactionType::Withdrawal => {
                    if selected_account.account_details.available_funds >= tx.amount {
                        // A withdraw is a debit to the client's asset account, meaning it
                        // should decrease the available and total funds of the client account.
                        selected_account.account_details.available_funds -= tx.amount;
                        selected_account.account_details.total_funds -= tx.amount;

                        // Update transaction lookup/history
                        selected_account
                            .account_transaction_archive
                            .details
                            .insert(tx.tx, (tx.amount, tx.tx_type));
                        selected_account
                            .account_transaction_archive
                            .history
                            .insert(tx.tx);
                    } else {
                        // If a client does not have sufficient available funds the withdrawal
                        // should fail and the total amount of funds should not change.
                        return Err(PaymentsTransactionError::NotEnoughAvailableFunds(
                            tx.client.to_string(),
                        ));
                    }
                    return Ok(());
                }
                TransactionType::Dispute => {
                    // A dispute references the transaction that is disputed by ID.
                    let disputed_tx = &tx.tx;

                    if selected_account
                        .account_transaction_archive
                        .history
                        .contains(disputed_tx)
                        && !selected_account
                            .account_transaction_archive
                            .disputes
                            .contains(disputed_tx)
                    {
                        // Get the disputed transaction's details first.
                        let tx_archive = &selected_account.account_transaction_archive;
                        let disputed_tx_details = tx_archive.details.get(disputed_tx).ok_or(
                            PaymentsTransactionError::TransactionDetailDoesNotExist(
                                disputed_tx.to_string(),
                            ),
                        )?;
                        let disputed_tx_amount = disputed_tx_details.0;
                        let _disputed_tx_type = &disputed_tx_details.1.clone();

                        // The client's available funds should decrease by the amount disputed.
                        // Held funds should increase by the amount disputed.
                        selected_account.account_details.available_funds -= disputed_tx_amount;
                        selected_account.account_details.held_funds += disputed_tx_amount;

                        // No need to update the transaction history and details here. We're not mutating total funds,
                        // only temporarily holding them. This dispute might get resolved or it might not,
                        // so it doesn't make sense to update history here yet. We'll add this transaction to the
                        // set of disputed ones and return here.
                        selected_account
                            .account_transaction_archive
                            .disputes
                            .insert(*disputed_tx);
                    } else {
                        // If the tx specified by the dispute doesn't exist we will assume this
                        // is an error on our partners side.
                        println!(
                            "Dispute transaction ID {} does not exist for client {}, ignoring.",
                            &tx.tx, &tx.client
                        );
                    }
                    return Ok(());
                }
                TransactionType::Resolve => {
                    // Resolves refer to a transaction that was under dispute by ID.
                    let disputed_tx = &tx.tx;

                    // If the transaction to resolve happened and is currently under dispute
                    if selected_account
                        .account_transaction_archive
                        .history
                        .contains(disputed_tx)
                        && selected_account
                            .account_transaction_archive
                            .disputes
                            .contains(disputed_tx)
                    {
                        // Get the transaction details associated with the dispute being resolved.
                        let tx_archive = &selected_account.account_transaction_archive;
                        let disputed_tx_details = tx_archive.details.get(disputed_tx).ok_or(
                            PaymentsTransactionError::TransactionDetailDoesNotExist(
                                disputed_tx.to_string(),
                            ),
                        )?;

                        let disputed_tx_amount = disputed_tx_details.0;
                        let _disputed_tx_type = disputed_tx_details.1.clone();

                        // The clients held funds should decrease by the amount no longer disputed,
                        // their available funds should increase by the amount no longer disputed,
                        // and their total funds should remain the same.
                        selected_account.account_details.held_funds -= disputed_tx_amount;
                        selected_account.account_details.available_funds += disputed_tx_amount;

                        // Funds that were previously disputed are no longer disputed.
                        selected_account
                            .account_transaction_archive
                            .disputes
                            .remove(disputed_tx);
                    } else {
                        // If the tx isn't under dispute, we can ignore the resolve and assume this
                        // is an error on our partner's side.
                        println!(
                            "Dispute transaction ID {} does not exist for client {}, ignoring.",
                            &tx.tx, &tx.client
                        );
                    }
                    return Ok(());
                }
                TransactionType::Chargeback => {
                    // Chargebacks refer to a transaction that was under dispute by ID.
                    // A chargeback is the final state of a dispute and represents the client reversing a transaction.
                    let disputed_tx = &tx.tx;
                    if selected_account
                        .account_transaction_archive
                        .history
                        .contains(disputed_tx)
                        && selected_account
                            .account_transaction_archive
                            .disputes
                            .contains(disputed_tx)
                    {
                        // If a chargeback occurs the client's account should be immediately frozen.
                        selected_account.account_details.is_account_locked = true;

                        // Get the transaction details associated with the dispute concluding with a chargeback.
                        let tx_archive = &selected_account.account_transaction_archive;
                        let disputed_tx_details = tx_archive.details.get(disputed_tx).ok_or(
                            PaymentsTransactionError::TransactionDetailDoesNotExist(
                                disputed_tx.to_string(),
                            ),
                        )?;

                        let disputed_tx_amount = disputed_tx_details.0;
                        let _disputed_tx_type = disputed_tx_details.1.clone();

                        // The clients held funds and total funds should decrease by the amount previously disputed.
                        selected_account.account_details.held_funds -= disputed_tx_amount;
                        selected_account.account_details.total_funds -= disputed_tx_amount;

                        // Funds that were previously disputed are no longer disputed. A chargeback
                        // is the final state of a dispute.
                        selected_account
                            .account_transaction_archive
                            .disputes
                            .remove(disputed_tx);
                    } else {
                        // If the chargeback tx isn't under dispute or isn't in this account's history,
                        // ignore the resolve and assume this is an error on our partner's side.
                        println!(
                            "Dispute transaction ID {} does not exist for client {}, ignoring.",
                            &tx.tx, &tx.client
                        );
                    }
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}
