/// Tests chargeback transaction behavior for a ClientAccount.
/// These tests were generated using ChatGPT and then manually
/// verified.
#[cfg(test)]
mod chargeback_tests {
    use crate::{
        account::client_account::ClientAccount,
        transaction::{Transaction, TransactionType},
    };

    /// Test that a chargeback on a valid disputed transaction
    /// removes the funds from held and total balances and locks the account.
    #[test]
    fn test_chargeback_applies_correctly() {
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
        let chargeback = Transaction {
            tx_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: 0.0,
        };
        acct.handle_deposit(deposit).unwrap();
        acct.handle_dispute(dispute).unwrap();
        acct.handle_chargeback(chargeback).unwrap();

        assert_eq!(acct.account_details.held_funds, 0.0);
        assert_eq!(acct.account_details.total_funds, 0.0);
        assert!(acct.account_details.is_account_locked);
        assert!(!acct.account_transaction_archive.disputes.contains(&1));
    }

    /// Test that a chargeback on a transaction that does not exist
    /// is safely ignored without modifying balances.
    #[test]
    fn test_chargeback_nonexistent_transaction() {
        let mut acct = ClientAccount::default();
        let chargeback = Transaction {
            tx_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: 0.0,
        };

        let result = acct.handle_chargeback(chargeback);
        assert!(result.is_ok());
        assert_eq!(acct.account_details.held_funds, 0.0);
        assert_eq!(acct.account_details.total_funds, 0.0);
        assert!(!acct.account_details.is_account_locked);
    }

    /// Test that a chargeback on a transaction that is not disputed
    /// does not modify balances or the disputes set.
    #[test]
    fn test_chargeback_not_disputed_transaction_is_ignored() {
        let mut acct = ClientAccount::default();

        let deposit = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        let chargeback = Transaction {
            tx_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: 0.0,
        };

        acct.handle_deposit(deposit.clone()).unwrap();
        acct.handle_chargeback(chargeback).unwrap();

        assert_eq!(acct.account_details.held_funds, 0.0);
        assert_eq!(acct.account_details.total_funds, 100.0);
        assert!(!acct.account_details.is_account_locked);
        assert!(!acct.account_transaction_archive.disputes.contains(&1));
    }

    /// Test that a successful chargeback removes the transaction
    /// from the disputes set.
    #[test]
    fn test_chargeback_removes_transaction_from_disputes() {
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
        let chargeback = Transaction {
            tx_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: 0.0,
        };
        acct.handle_deposit(deposit).unwrap();
        acct.handle_dispute(dispute).unwrap();

        assert_eq!(acct.account_details.held_funds, 100.0);
        assert_eq!(acct.account_details.total_funds, 100.0);
        assert_eq!(acct.account_details.available_funds, 0.0);

        acct.handle_chargeback(chargeback).unwrap();

        assert_eq!(acct.account_details.held_funds, 0.0);
        assert_eq!(acct.account_details.total_funds, 0.0);
        assert_eq!(acct.account_details.available_funds, 0.0);
        assert!(acct.account_details.is_account_locked);

        // Disputed transactions should no longer be disputed.
        assert!(!acct.account_transaction_archive.disputes.contains(&1));
    }

    /// Test that multiple chargeback calls on the same transaction
    /// do not further modify balances after the first call.
    #[test]
    fn test_multiple_chargebacks_are_ignored() {
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
        let chargeback = Transaction {
            tx_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: 0.0,
        };
        acct.handle_deposit(deposit).unwrap();
        acct.handle_dispute(dispute).unwrap();

        acct.handle_chargeback(chargeback.clone()).unwrap();
        acct.handle_chargeback(chargeback).unwrap(); // ignored

        assert_eq!(acct.account_details.held_funds, 0.0);
        assert_eq!(acct.account_details.total_funds, 0.0);
        assert!(acct.account_details.is_account_locked);
    }
}
