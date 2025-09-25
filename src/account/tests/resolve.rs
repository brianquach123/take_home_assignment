/// Tests transaction resolve behavior for a ClientAccount.
/// These tests were generated using ChatGPT and then manually
/// verified.
#[cfg(test)]
mod resolve_tests {
    use crate::{
        account::client_account::ClientAccount,
        transaction::{Transaction, TransactionType},
    };

    /// Test that resolving a valid disputed transaction moves funds
    /// from held back to available balance.
    #[test]
    fn test_resolve_moves_funds() {
        let mut acct = ClientAccount::default();
        let deposit = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        let dispute = Transaction {
            tx_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: 0.0,
        };
        acct.handle_deposit(deposit).unwrap();
        acct.handle_dispute(dispute).unwrap();

        acct.handle_resolve(dispute).unwrap();

        assert_eq!(acct.account_details.available_funds, 100.0);
        assert_eq!(acct.account_details.held_funds, 0.0);
        assert!(!acct.account_transaction_archive.disputes.contains(&1));
    }

    /// Test that resolving a transaction that was never recorded
    /// does not modify balances and is safely ignored.
    #[test]
    fn test_resolve_nonexistent_transaction() {
        let mut acct = ClientAccount::default();
        let resolve = Transaction {
            tx_type: TransactionType::Resolve,
            client: 1,
            tx: 99,
            amount: 0.0,
        };

        let result = acct.handle_resolve(resolve);
        assert!(result.is_ok());
        assert_eq!(acct.account_details.available_funds, 0.0);
        assert_eq!(acct.account_details.held_funds, 0.0);
    }

    /// Test that resolving a transaction that is not currently disputed
    /// does not affect account balances or the disputes set.
    #[test]
    fn test_resolve_not_disputed_transaction_is_ignored() {
        let mut acct = ClientAccount::default();
        let deposit = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        let resolve = Transaction {
            // transaction exists but not disputed
            tx_type: TransactionType::Resolve,
            client: 1,
            tx: 1,
            amount: 0.0,
        };
        acct.handle_deposit(deposit).unwrap();
        acct.handle_resolve(resolve).unwrap();

        assert_eq!(acct.account_details.available_funds, 100.0);
        assert_eq!(acct.account_details.held_funds, 0.0);
        assert!(!acct.account_transaction_archive.disputes.contains(&1));
    }

    /// Test that resolving a transaction removes it from the disputes set.
    /// This confirms that the dispute is properly cleared.
    #[test]
    fn test_resolve_removes_transaction_from_disputes() {
        let mut acct = ClientAccount::default();
        let deposit = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        let dispute = Transaction {
            tx_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        let resolve = Transaction {
            // transaction exists but not disputed
            tx_type: TransactionType::Resolve,
            client: 1,
            tx: 1,
            amount: 0.0,
        };
        acct.handle_deposit(deposit).unwrap();
        acct.handle_dispute(dispute).unwrap();
        acct.handle_resolve(resolve).unwrap();

        assert!(!acct.account_transaction_archive.disputes.contains(&1));
    }

    /// Test that multiple resolve calls on the same transaction
    /// do not incorrectly modify balances after the first resolve.
    #[test]
    fn test_multiple_resolves_are_ignored() {
        let mut acct = ClientAccount::default();
        let deposit = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        let dispute = Transaction {
            tx_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        let resolve = Transaction {
            // transaction exists but not disputed
            tx_type: TransactionType::Resolve,
            client: 1,
            tx: 1,
            amount: 0.0,
        };
        acct.handle_deposit(deposit).unwrap();
        acct.handle_dispute(dispute).unwrap();

        acct.handle_resolve(resolve).unwrap();
        acct.handle_resolve(resolve).unwrap(); // ignored second call

        assert_eq!(acct.account_details.available_funds, 100.0);
        assert_eq!(acct.account_details.held_funds, 0.0);
        assert!(!acct.account_transaction_archive.disputes.contains(&1));
    }
}
