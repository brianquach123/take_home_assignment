use csv;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::path::Path;

/// Representation of a transaction.
#[derive(Debug, Deserialize)]
pub struct Transaction {
    tx_type: TransactionType,
    client_id: u16, // Assumed type from assignment spec
    tx_td: u32,     // Assumed type from assignment spec
    amount: f64,
}

/// Representation of a state a transaction may be in.
#[derive(Debug, Deserialize)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
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
    if args.len() != 2 {
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
