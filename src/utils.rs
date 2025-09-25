use crate::{
    errors::PaymentsTransactionError,
    transaction::{Transaction, TransactionType},
};
use anyhow::Context;
use anyhow::Result;
/// This file defines general helper funtions for the payments engine.
use csv::Reader;
use csv::Writer;
use rand::{Rng, rng, seq::IndexedRandom};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use strum::IntoEnumIterator;

/// Reads and returns a csv::Reader<File> over a file if
/// the file exists and ends with ".csv".
pub fn initialize_csv_reader(filename: &str) -> Result<Reader<File>, PaymentsTransactionError> {
    let path = Path::new(filename);
    if !path.exists() {
        return Err(PaymentsTransactionError::TransactionCsvDoesNotExist(
            filename.to_string(),
        ));
    }
    if path.extension().and_then(|ext| ext.to_str()) != Some("csv") {
        return Err(PaymentsTransactionError::InvalidTransactionFileExtension(
            filename.to_string(),
        ));
    }
    let file = File::open(filename)?;
    Ok(Reader::from_reader(file))
}

/// Writes a randomized test CSV given a number of transactions and clients
/// to initialize the CSV with. Transaction min/max amounts are hardcoded.
fn _generate_transaction_csv(total_transactions: u32, total_clients: u16) -> Result<()> {
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
            tx_type: *tx_types.choose(&mut rng).unwrap(),
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
