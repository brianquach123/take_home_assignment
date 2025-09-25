mod account;
mod transaction;
use transaction::*;
mod errors;
mod payments_engine;
mod utils;
use anyhow::Error;
use anyhow::Result;
use log::debug;
use std::collections::HashMap;
use std::env;

use crate::payments_engine::engine::PaymentsEngine;
use crate::utils::MAX_CLI_ARGS;
use crate::utils::initialize_csv_reader;

fn main() -> Result<(), Error> {
    /*
        The original file detection logic was provided by ChatGPT
        and later heavily modified to fit the requirements of the
        assignment. The prompt given to ChatGPT was:
        "Can you show me a sample rust program that takes
        and checks for one and only one cli argument that 
        is supposed to be a CSV file?"

        Please reference past commits on this repository's main branch
        to see how this main() logic has evolved.
    */
    let mut payments_engine: PaymentsEngine = PaymentsEngine {
        client_account_lookup: HashMap::new(),
    };

    let args: Vec<String> = env::args().collect();
    if args.len() != MAX_CLI_ARGS {
        panic!("Usage: {} <transactions_file.csv>", args[0]);
    }

    // Open and process transactions from the csv file.
    // The file must exist and be a CSV.
    let filename = &args[1];
    for res in initialize_csv_reader(filename)?.deserialize() {
        /*
        From the assignment spec:
        - The client ID will be unique per client though are not guaranteed to be ordered.
        - Can assume transactions occur chronologically in the file.
        - Whitespaces and decimal precisions (up to four places past the decimal) must be accepted.
        */
        let curr_transaction: Transaction = res?;
        debug!("{:?}", curr_transaction);
        payments_engine.process_transaction(curr_transaction)?;
    }
    println!("{}", payments_engine);
    Ok(())
}
