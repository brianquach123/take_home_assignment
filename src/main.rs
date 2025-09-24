use csv;
use rand::Rng;
use rand::prelude::IndexedRandom;
use rand::rng;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use strum::{EnumIter, IntoEnumIterator};

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
#[derive(Debug)]
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

/// Writes a randomized test CSV given a number of transactions and clients
/// to initialize the CSV with. Transaction min/max amounts are determined
/// by MIN_TRANSACTION_AMOUNT and MAX_TRANSACTION_AMOUNT.
fn generate_transaction_csv(total_transactions: u32, total_clients: u16) {
    let min_transaction_amount: f64 = 0.00;
    let max_transaction_amount: f64 = 100.00;

    // Open a file and wrap it in a buffered writer
    let file = File::create("transactions.csv").unwrap_or_else(|err| {
        eprintln!("error creating tx file: {}", err);
        std::process::exit(1);
    });
    let buf_writer = BufWriter::new(file);
    let mut wtr = csv::Writer::from_writer(buf_writer);

    let mut rng = rng();
    let tx_types: Vec<TransactionType> = TransactionType::iter().collect();

    for tx in 0..total_transactions {
        let curr_tx = Transaction {
            tx_type: tx_types.choose(&mut rng).unwrap().clone(),
            client: rng.random_range(0..total_clients),
            tx,
            amount: rng.random_range(min_transaction_amount..max_transaction_amount),
        };
        wtr.serialize(curr_tx).unwrap_or_else(|err| {
            eprintln!("error writing tx to csv file: {}", err);
            std::process::exit(1);
        });
        wtr.flush().unwrap_or_else(|err| {
            eprintln!("error flushing csv file: {}", err);
            std::process::exit(1);
        });
    }
}

fn main() {
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
        eprintln!("Usage: {} <transactions_file.csv>", args[0]);
        std::process::exit(1);
    }

    // File must exist and be a CSV.
    let filename = &args[1];
    let path = Path::new(filename);
    if !path.exists() {
        eprintln!("Error: file '{}' does not exist", filename);
        std::process::exit(1);
    }
    if path.extension().and_then(|ext| ext.to_str()) != Some("csv") {
        eprintln!("Error: argument must be a .csv file");
        std::process::exit(1);
    }

    // Maintain a lookup of clients to their associated account values for reporting.
    let mut client_account_lookup: HashMap<u16, ClientAccountDetails> = HashMap::new();

    // Open and process transactions from the csv file.
    let file = File::open(filename).unwrap_or_else(|err| {
        eprintln!("error opening file: {}", err);
        std::process::exit(1);
    });
    let mut reader = csv::Reader::from_reader(file);
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
    }

    println!("CSV file processed.");
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
