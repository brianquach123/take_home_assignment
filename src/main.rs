use anyhow::bail;
use anyhow::{Context, Result};
use csv::{self, Reader, Writer};
use rand::Rng;
use rand::prelude::IndexedRandom;
use rand::rng;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use strum::{EnumIter, IntoEnumIterator};
use thiserror::Error;

const MAX_CLI_ARGS: usize = 2;

/// Representation of a transaction.
#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    /// Type of Transaction.
    tx_type: TransactionType,
    /// Client ID.
    client: u16,
    /// Transaction ID. Assumed type from assignment spec.
    tx: u32, //
    /// Transaction amount. Assumed type from assignment spec.
    #[serde(serialize_with = "serialize_up_to_four_decimal_places")]
    amount: f64,
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

/// Representation of a state a transaction may be in.
#[derive(Debug, Deserialize, Serialize, EnumIter, Clone)]
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

/// Representation of a client's account details in the engine.
/// The engine uses this for reporting output to stdout.
#[derive(Debug, Default)]
pub struct ClientAccountDetails {
    available_funds: f64,
    held_funds: f64,
    total_funds: f64,
    is_account_locked: bool,
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

#[derive(Debug, Default)]
pub struct ClientTransactionArchive {
    /// The set of transaction IDs associated with this account.
    history: BTreeSet<u32>,
    /// Map of the set of transaction IDs to (amount, type of transaction)
    /// for this account.
    details: HashMap<u32, (f64, TransactionType)>,
    /// The set of disputed transactions for this account.
    disputes: BTreeSet<u32>,
}
/// Representation of the payments engine.
#[derive(Debug, Default)]
pub struct PaymentsEngine {
    /// Maps a client's ID to their `ClientAccount`.
    client_account_lookup: HashMap<u16, ClientAccount>,
}

/// Representation of a client's account in the payments engine.
/// A client account is defined by its funds' details and lock status,
/// in addition to the set of transactions ID associated with this client
/// account that the payments engine has previously processed.
#[derive(Debug, Default)]
pub struct ClientAccount {
    /// Balance details and lock status for this account.
    account_details: ClientAccountDetails,
    /// Transaction history and details for this account.
    account_transaction_archive: ClientTransactionArchive,
}

#[derive(Debug, Error)]
pub enum PaymentsTransactionError {
    #[error("Not enough available funds for client {0}")]
    NotEnoughAvailableFunds(String),
    #[error("Transaction details not found for transaction {0}")]
    TransactionDetailDoesNotExist(String),
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

/// Writes a randomized test CSV given a number of transactions and clients
/// to initialize the CSV with. Transaction min/max amounts are hardcoded.
fn generate_transaction_csv(total_transactions: u32, total_clients: u16) -> Result<()> {
    let min_transaction_amount: f64 = 0.00;
    let max_transaction_amount: f64 = 100.00;

    // Open a file and wrap it in a buffered writer
    let file = File::create("transactions.csv").context("error creating transactions.csv")?;
    let buf_writer = BufWriter::new(file);
    let mut wtr = Writer::from_writer(buf_writer);

    let mut rng = rng();
    let tx_types: Vec<TransactionType> = TransactionType::iter().collect();
    for tx in 0..total_transactions {
        let curr_tx = Transaction {
            tx_type: tx_types.choose(&mut rng).unwrap().clone(),
            client: rng.random_range(0..total_clients),
            tx,
            amount: rng.random_range(min_transaction_amount..max_transaction_amount),
        };
        wtr.serialize(curr_tx)
            .context("Error writing transaction to CSV")?;
        wtr.flush().context("Error flushing CSV writer")?;
    }
    Ok(())
}

fn main() -> Result<()> {
    /*
        This file detection logic was provided by ChatGPT
        and later modified to fit the requirements of the assignment.
        The prompt given to ChatGPT was:
        "Can you show me a sample rust program that takes
        and checks for one and only one cli argument that 
        is supposed to be a CSV file?""
    */
    let args: Vec<String> = env::args().collect();
    if args.len() != MAX_CLI_ARGS {
        bail!("Usage: {} <transactions_file.csv>", args[0]);
    }

    // File must exist and be a CSV.
    let filename = &args[1];
    let path = Path::new(filename);
    if !path.exists() {
        bail!("File '{}' does not exist", filename);
    }
    if path.extension().and_then(|ext| ext.to_str()) != Some("csv") {
        bail!("Argument must be a .csv file");
    }

    let mut payments_engine: PaymentsEngine = PaymentsEngine {
        client_account_lookup: HashMap::new(),
    };

    // Open and process transactions from the csv file.
    let file = File::open(filename).unwrap_or_else(|err| {
        eprintln!("error opening file: {}", err);
        std::process::exit(1);
    });
    let mut reader = Reader::from_reader(file);
    for res in reader.deserialize() {
        /*
        From the assignment spec:
        - The client ID will be unique per client though are not guaranteed to be ordered.
        - Can assume transactions occur chronologically in the file.
        - Whitespaces and decimal precisions (up to four places past the decimal) must be accepted.
        */
        let curr_transaction: Transaction = res.unwrap_or_else(|err| {
            eprintln!("Error reading transaction: {}", err);
            std::process::exit(1) // todo maybe just continue here, don't fail immediately on one bad tx.
        });
        println!("{:?}", curr_transaction);

        // Process the current transaction.
        payments_engine.process_transaction(curr_transaction)?;
    }

    println!("CSV file processed.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_test_csv() {
        let num_txes: u32 = 20;
        let num_clients: u16 = 6;
        // Write CSV data into the in-memory buffer
        generate_transaction_csv(num_txes, num_clients);
    }
}
