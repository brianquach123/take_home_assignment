/// Tests deposit transaction behavior for a ClientAccount.
/// These tests were generated using ChatGPT and then manually
/// verified.
#[cfg(test)]
mod deposit_tests {
    use crate::{
        account::client_account::ClientAccount,
        transaction::{Transaction, TransactionType},
    };

    fn sample_account() -> ClientAccount {
        ClientAccount::default()
    }

    /// Single deposit increases available and total funds by the transaction amount.
    /// The transaction ID should be recorded in the account's history and details.
    #[test]
    fn test_single_deposit() {
        let mut account = sample_account();
        let tx = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 50.0,
        };

        account.handle_deposit(tx).unwrap();

        assert_eq!(account.account_details.available_funds, 50.0);
        assert_eq!(account.account_details.total_funds, 50.0);
        assert!(account.account_transaction_archive.history.contains(&1));
        assert_eq!(
            account.account_transaction_archive.details.get(&1),
            Some(&(50.0, TransactionType::Deposit))
        );
    }

    /// Deposit with fractional amount updates balances correctly.
    /// Ensures that decimal amounts are handled precisely.
    #[test]
    fn test_fractional_deposit() {
        let mut account = sample_account();
        let tx = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 2,
            amount: 25.5,
        };
        account.handle_deposit(tx).unwrap();

        assert_eq!(account.account_details.available_funds, 25.5);
        assert_eq!(account.account_details.total_funds, 25.5);
    }

    /// Multiple deposits accumulate correctly in available and total funds.
    /// Each transaction ID should appear in the account's history.
    #[test]
    fn test_multiple_deposits_accumulate() {
        let mut account = sample_account();
        let tx1 = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 10.0,
        };
        let tx2 = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 2,
            amount: 15.0,
        };

        account.handle_deposit(tx1).unwrap();
        account.handle_deposit(tx2).unwrap();

        assert_eq!(account.account_details.available_funds, 25.0);
        assert_eq!(account.account_details.total_funds, 25.0);
        assert!(account.account_transaction_archive.history.contains(&1));
        assert!(account.account_transaction_archive.history.contains(&2));
    }

    /// Depositing with a duplicate transaction ID does not overwrite the previous amount in details.
    /// The history set should only contain the transaction ID once.
    #[test]
    fn test_duplicate_transaction_id_does_not_overwrite_previous_transaction() {
        let mut account = sample_account();
        let tx1 = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 10.0,
        };
        let tx2 = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 1, // same tx ID
            amount: 20.0,
        };

        account.handle_deposit(tx1).unwrap();
        account.handle_deposit(tx2).unwrap();

        // The last deposit does not overwrite the amount in details
        assert_eq!(
            account.account_transaction_archive.details.get(&1),
            Some(&(10.0, TransactionType::Deposit))
        );

        // History still only contains tx ID once
        assert_eq!(account.account_transaction_archive.history.len(), 1);
        assert!(account.account_transaction_archive.history.contains(&1));
    }

    /// A deposit with zero amount leaves balances unchanged.
    /// The transaction ID is still recorded in history.
    /// Ensures zero-value deposits are logged but do not affect funds.
    #[test]
    fn test_zero_amount_deposit() {
        let mut account = sample_account();
        let tx = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 3,
            amount: 0.0,
        };
        account.handle_deposit(tx).unwrap();

        assert_eq!(account.account_details.available_funds, 0.0);
        assert_eq!(account.account_details.total_funds, 0.0);
        assert!(account.account_transaction_archive.history.contains(&3));
    }

    /// Very large deposits update balances correctly without overflow.
    /// Confirms that extreme values are handled safely.
    /// Ensures the system can process high-value transactions.
    #[test]
    fn test_large_deposit() {
        let mut account = sample_account();
        let tx = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 4,
            amount: 1e12,
        };

        account.handle_deposit(tx).unwrap();

        assert_eq!(account.account_details.available_funds, 1e12);
        assert_eq!(account.account_details.total_funds, 1e12);
    }
}
