/// This file defines structs and methods associated with a client account in
/// the payments engine.
use log::warn;
use std::{
    collections::{BTreeSet, HashMap},
    fmt,
};

use crate::transaction::TransactionType;
use crate::{PaymentsTransactionError, transaction::Transaction};

/// Representation of a client's account in the payments engine.
/// A client account is defined by its funds' details and lock status,
/// the set of transactions and their ID associated with this client,
/// and the set of transaction IDs that are currently under dispute
/// account that the payments engine has previously processed.
#[derive(Debug, Default)]
pub struct ClientAccount {
    /// Balance details and lock status for this account.
    pub account_details: ClientAccountDetails,
    /// Transaction history and details for this account.
    pub account_transaction_archive: ClientTransactionArchive,
}

impl ClientAccount {
    /// A deposit is a credit to the client's asset account, meaning it
    /// should increase the available and total funds of the client account.
    /// Additionally, since total funds are mutated on a successful deposit,
    /// the account's transaction history is updated as well.
    pub fn handle_deposit(&mut self, tx: Transaction) -> Result<(), PaymentsTransactionError> {
        self.account_details.available_funds += tx.amount;
        self.account_details.total_funds += tx.amount;

        self.account_transaction_archive
            .details
            .insert(tx.tx, (tx.amount, tx.tx_type));
        self.account_transaction_archive.history.insert(tx.tx);
        Ok(())
    }

    /// A withdrawal is a debit to the client's asset account, meaning it
    /// should decrease the available and total funds of the client account.
    /// Additionally, since total funds are mutated on a successful withdrawal,
    /// the account's transaction history is updated as well.
    ///
    /// If a client does not have sufficient available funds, the withdrawal
    /// will fail and the total amount of funds will not change.
    pub fn handle_withdrawal(&mut self, tx: Transaction) -> Result<(), PaymentsTransactionError> {
        if self.account_details.available_funds >= tx.amount {
            self.account_details.available_funds -= tx.amount;
            self.account_details.total_funds -= tx.amount;

            self.account_transaction_archive
                .details
                .insert(tx.tx, (tx.amount, tx.tx_type));
            self.account_transaction_archive.history.insert(tx.tx);
        } else {
            return Err(PaymentsTransactionError::NotEnoughAvailableFunds(
                tx.client.to_string(),
            ));
        }
        Ok(())
    }
    /// A dispute references the transaction that is disputed by ID.
    /// The client's available funds should decrease by the amount disputed.
    /// Held funds should increase by the amount disputed. Since an account's
    /// total funds are not impacted by initiating a dispute, a dispute transaction
    /// will not go into a `ClientAccount`'s transaction history.
    pub fn handle_dispute(&mut self, tx: Transaction) -> Result<(), PaymentsTransactionError> {
        let disputed_tx = &tx.tx;
        let has_tx_happened = self
            .account_transaction_archive
            .history
            .contains(disputed_tx);
        let is_tx_not_being_disputed = !self
            .account_transaction_archive
            .disputes
            .contains(disputed_tx);

        if has_tx_happened && is_tx_not_being_disputed {
            // Get the disputed transaction's details first.
            let tx_archive = &self.account_transaction_archive;
            let disputed_tx_details = tx_archive.details.get(disputed_tx).ok_or(
                PaymentsTransactionError::TransactionDetailDoesNotExist(disputed_tx.to_string()),
            )?;
            let disputed_tx_amount = disputed_tx_details.0;

            self.account_details.available_funds -= disputed_tx_amount;
            self.account_details.held_funds += disputed_tx_amount;

            // No need to update the transaction history and details here. We're not mutating total funds,
            // only temporarily holding them. This dispute might get resolved or it might not,
            // so it doesn't make sense to update history here yet. We'll add this transaction to the
            // set of disputed ones and return here.
            self.account_transaction_archive
                .disputes
                .insert(*disputed_tx);
        } else {
            // If the tx specified by the dispute doesn't exist we will assume this
            // is an error on our partners side.
            warn!(
                "Dispute referenced transaction ID {} does not exist for client {}, ignoring.",
                &tx.tx, &tx.client
            );
        }
        Ok(())
    }

    /// Resolves refer to a transaction that was under dispute by ID.
    /// The clients held funds should decrease by the amount no longer disputed,
    /// their available funds should increase by the amount no longer disputed,
    /// and their total funds should remain the same.
    pub fn handle_resolve(&mut self, tx: Transaction) -> Result<(), PaymentsTransactionError> {
        let disputed_tx = &tx.tx;
        let has_tx_happened = self
            .account_transaction_archive
            .history
            .contains(disputed_tx);
        let is_tx_being_disputed = self
            .account_transaction_archive
            .disputes
            .contains(disputed_tx);
        if has_tx_happened && is_tx_being_disputed {
            // Get the transaction details associated with the dispute being resolved.
            let tx_archive = &self.account_transaction_archive;
            let disputed_tx_details = tx_archive.details.get(disputed_tx).ok_or(
                PaymentsTransactionError::TransactionDetailDoesNotExist(disputed_tx.to_string()),
            )?;
            let disputed_tx_amount = disputed_tx_details.0;

            self.account_details.held_funds -= disputed_tx_amount;
            self.account_details.available_funds += disputed_tx_amount;

            // Funds that were previously disputed are no longer disputed.
            self.account_transaction_archive
                .disputes
                .remove(disputed_tx);
        } else {
            // If the tx isn't under dispute, we can ignore the resolve and assume this
            // is an error on our partner's side.
            warn!(
                "Resolve referenced transaction ID {} does not exist for client {}, ignoring.",
                &tx.tx, &tx.client
            );
        }
        Ok(())
    }

    /// Chargebacks refer to a transaction that was under dispute by ID.
    /// A chargeback is the final state of a dispute and represents the client reversing a transaction.
    /// If a chargeback occurs the client's account should be immediately frozen.
    /// The client's held funds and total funds should decrease by the amount previously disputed.
    pub fn handle_chargeback(&mut self, tx: Transaction) -> Result<(), PaymentsTransactionError> {
        let disputed_tx = &tx.tx;
        let has_tx_happened = self
            .account_transaction_archive
            .history
            .contains(disputed_tx);
        let is_tx_being_disputed = self
            .account_transaction_archive
            .disputes
            .contains(disputed_tx);
        if has_tx_happened && is_tx_being_disputed {
            self.account_details.is_account_locked = true;

            // Get the transaction details associated with the dispute concluding with a chargeback.
            let tx_archive = &self.account_transaction_archive;
            let disputed_tx_details = tx_archive.details.get(disputed_tx).ok_or(
                PaymentsTransactionError::TransactionDetailDoesNotExist(disputed_tx.to_string()),
            )?;

            let disputed_tx_amount = disputed_tx_details.0;

            self.account_details.held_funds -= disputed_tx_amount;
            self.account_details.total_funds -= disputed_tx_amount;

            // Funds that were previously disputed are no longer disputed.
            self.account_transaction_archive
                .disputes
                .remove(disputed_tx);
        } else {
            // If the chargeback tx isn't under dispute or isn't in this account's history,
            // ignore the resolve and assume this is an error on our partner's side.
            warn!(
                "Chargeback referenced transaction ID {} does not exist for client {}, ignoring.",
                &tx.tx, &tx.client
            );
        }
        Ok(())
    }
}

/// Representation of a client account's history of processed transactions
/// with their amount totals and type.
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
