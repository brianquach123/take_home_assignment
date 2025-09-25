#[cfg(test)]
mod tests {
    use crate::PaymentsEngine;
    use crate::transaction::{Transaction, TransactionType};

    /// Helper to create a deposit transaction
    fn make_deposit_tx(id: u32, client: u16, amount: f64) -> Transaction {
        Transaction {
            tx_type: TransactionType::Deposit,
            client,
            tx: id,
            amount,
        }
    }

    /// Helper to create a withdrawal transaction
    fn make_withdrawal_tx(id: u32, client: u16, amount: f64) -> Transaction {
        Transaction {
            tx_type: TransactionType::Withdrawal,
            client,
            tx: id,
            amount,
        }
    }

    /// Test that a deposit transaction creates a new client account
    /// if the client does not exist yet, and updates balances correctly.
    #[test]
    fn test_process_transaction_deposit_creates_client() {
        let mut engine = PaymentsEngine {
            client_account_lookup: Default::default(),
        };

        let deposit = make_deposit_tx(1, 1, 100.0);
        engine.process_transaction(deposit).unwrap();

        let acct = engine.client_account_lookup.get(&1).unwrap();
        assert_eq!(acct.account_details.available_funds, 100.0);
        assert_eq!(acct.account_details.total_funds, 100.0);
    }

    /// Test that a withdrawal transaction deducts funds from an existing client account
    /// when sufficient funds are available.
    #[test]
    fn test_process_transaction_withdrawal_succeeds() {
        let mut engine = PaymentsEngine {
            client_account_lookup: Default::default(),
        };

        let deposit = make_deposit_tx(1, 1, 100.0);
        engine.process_transaction(deposit).unwrap();

        let withdrawal = make_withdrawal_tx(2, 1, 40.0);
        engine.process_transaction(withdrawal).unwrap();

        let acct = engine.client_account_lookup.get(&1).unwrap();
        assert_eq!(acct.account_details.available_funds, 60.0);
        assert_eq!(acct.account_details.total_funds, 60.0);
    }

    /// Test that a withdrawal transaction does not fail the engine even if
    /// the client has insufficient funds; the error is ignored.
    #[test]
    fn test_process_transaction_withdrawal_insufficient_funds() {
        let mut engine = PaymentsEngine {
            client_account_lookup: Default::default(),
        };

        let deposit = make_deposit_tx(1, 1, 50.0);
        engine.process_transaction(deposit).unwrap();

        let withdrawal = make_withdrawal_tx(2, 1, 100.0);
        engine.process_transaction(withdrawal).unwrap(); // should be ignored

        let acct = engine.client_account_lookup.get(&1).unwrap();
        assert_eq!(acct.account_details.available_funds, 50.0);
        assert_eq!(acct.account_details.total_funds, 50.0);
    }

    /// Test that a deposit followed by a withdrawal results in correct
    /// available and total balances for a client.
    #[test]
    fn test_deposit_then_withdrawal_combined() {
        let mut engine = PaymentsEngine {
            client_account_lookup: Default::default(),
        };

        engine
            .process_transaction(make_deposit_tx(1, 1, 200.0))
            .unwrap();
        engine
            .process_transaction(make_withdrawal_tx(2, 1, 50.0))
            .unwrap();

        let acct = engine.client_account_lookup.get(&1).unwrap();
        assert_eq!(acct.account_details.available_funds, 150.0);
        assert_eq!(acct.account_details.total_funds, 150.0);
    }

    /// Test that `process_transaction` ignores duplicate transaction IDs
    /// and does not double-apply the same transaction.
    #[test]
    fn test_duplicate_transaction_is_ignored() {
        let mut engine = PaymentsEngine {
            client_account_lookup: Default::default(),
        };

        let deposit = make_deposit_tx(1, 1, 100.0);
        engine.process_transaction(deposit.clone()).unwrap();
        engine.process_transaction(deposit.clone()).unwrap(); // duplicate

        let acct = engine.client_account_lookup.get(&1).unwrap();
        assert_eq!(acct.account_details.available_funds, 100.0);
        assert_eq!(acct.account_details.total_funds, 100.0);
    }

    /// Test that the `Display` implementation correctly formats
    /// the client ID and account details as CSV-style output.
    #[test]
    fn test_display_outputs_correct_format() {
        let mut engine = PaymentsEngine {
            client_account_lookup: Default::default(),
        };

        engine
            .process_transaction(make_deposit_tx(1, 1, 100.0))
            .unwrap();
        engine
            .process_transaction(make_deposit_tx(2, 2, 200.0))
            .unwrap();

        let output = format!("{}", engine);
        assert!(output.contains("client, available, held, total, locked"));
        assert!(output.contains("1, 100.0000, 0.0000, 100.0000, false"));
        assert!(output.contains("2, 200.0000, 0.0000, 200.0000, false"));
    }

    /// Test that multiple clients can be handled by the engine
    /// and balances are tracked separately for each client.
    #[test]
    fn test_multiple_clients_transactions() {
        let mut engine = PaymentsEngine {
            client_account_lookup: Default::default(),
        };

        engine
            .process_transaction(make_deposit_tx(1, 1, 100.0))
            .unwrap();
        engine
            .process_transaction(make_deposit_tx(2, 2, 300.0))
            .unwrap();

        let acct1 = engine.client_account_lookup.get(&1).unwrap();
        let acct2 = engine.client_account_lookup.get(&2).unwrap();
        assert_eq!(acct1.account_details.available_funds, 100.0);
        assert_eq!(acct2.account_details.available_funds, 300.0);
    }
}
