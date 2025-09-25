/// This file defines a `PaymentsTransactionError` type that is conditionally
/// reported by the engine when a given payments engine error state has occured.
use thiserror::Error;

/// Custom payments engine error type
#[derive(Debug, Error)]
pub enum PaymentsTransactionError {
    #[error("Not enough available funds for client {0}")]
    NotEnoughAvailableFunds(String),
    #[error("Transaction details not found for transaction {0}")]
    TransactionDetailDoesNotExist(String),
}
