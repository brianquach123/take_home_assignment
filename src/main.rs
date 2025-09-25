mod account;
mod transaction;
use transaction::*;
mod payments_engine;
use anyhow::Result;
use anyhow::{Error, bail};
use csv::{self, Reader};
use payments_engine::*;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::Path;

const MAX_CLI_ARGS: usize = 2;

fn main() -> Result<(), Error> {
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
    let file = File::open(filename)?;
    let mut reader = Reader::from_reader(file);
    for res in reader.deserialize() {
        /*
        From the assignment spec:
        - The client ID will be unique per client though are not guaranteed to be ordered.
        - Can assume transactions occur chronologically in the file.
        - Whitespaces and decimal precisions (up to four places past the decimal) must be accepted.
        */
        let curr_transaction: Transaction = res?;
        println!("{:?}", curr_transaction);

        // Process the current transaction.
        let _ = payments_engine.process_transaction(curr_transaction);
    }
    print!("\n\n{}\n\n", payments_engine);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::BufWriter;

    use anyhow::Context;
    use csv::Writer;
    use rand::{Rng, rng, seq::IndexedRandom};
    use strum::IntoEnumIterator;

    use super::*;

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

    #[test]
    fn test_write_test_csv() {
        let num_txes: u32 = 20;
        let num_clients: u16 = 6;
        // Write CSV data into the in-memory buffer
        let _ = generate_transaction_csv(num_txes, num_clients);
    }
}
