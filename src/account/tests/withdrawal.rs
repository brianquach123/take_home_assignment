/// Tests withdrawal behavior for a ClientAccount.
/// These tests were generated using ChatGPT and then manually
/// verified.
#[cfg(test)]
mod withdrawal_tests {
    use crate::{
        account::client_account::ClientAccount,
        errors::PaymentsTransactionError,
        transaction::{Transaction, TransactionType},
    };

    fn sample_account_with_balance(balance: f64) -> ClientAccount {
        let mut account = ClientAccount::default();
        account.account_details.available_funds = balance;
        account.account_details.total_funds = balance;
        account
    }

    /// Single withdrawal decreases available and total funds by the transaction amount.
    /// The transaction ID should be recorded in the account's history and details.
    /// This is the basic happy path test for withdrawals.
    #[test]
    fn test_single_withdrawal() {
        let mut account = sample_account_with_balance(100.0);
        let tx = Transaction {
            tx_type: TransactionType::Withdrawal,
            client: 1,
            tx: 1,
            amount: 40.0,
        };
        account.handle_withdrawal(tx).unwrap();

        assert_eq!(account.account_details.available_funds, 60.0);
        assert_eq!(account.account_details.total_funds, 60.0);
        assert!(account.account_transaction_archive.history.contains(&1));
        assert_eq!(
            account.account_transaction_archive.details.get(&1),
            Some(&(40.0, TransactionType::Withdrawal))
        );
    }

    /// Withdrawal fails if the account does not have enough available funds.
    /// The function should return a `NotEnoughAvailableFunds` error.
    /// This prevents accounts from going negative.
    #[test]
    fn test_withdrawal_insufficient_funds() {
        let mut account = sample_account_with_balance(20.0);
        let tx = Transaction {
            tx_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: 50.0,
        };
        let result = account.handle_withdrawal(tx);

        assert!(matches!(
            result,
            Err(PaymentsTransactionError::NotEnoughAvailableFunds(_))
        ));
        assert_eq!(account.account_details.available_funds, 20.0);
        assert_eq!(account.account_details.total_funds, 20.0);
        // A failed withdrawal should not go into the set of successful withdrawals and deposits.
        assert!(!account.account_transaction_archive.history.contains(&2));
    }

    /// Multiple withdrawals reduce balances correctly when funds are available.
    /// Each withdrawal should add its transaction ID to the history.
    /// Confirms successive withdrawals accumulate properly.
    #[test]
    fn test_multiple_withdrawals_accumulate() {
        let mut account = sample_account_with_balance(100.0);
        let tx1 = Transaction {
            tx_type: TransactionType::Withdrawal,
            client: 1,
            tx: 3,
            amount: 30.0,
        };
        let tx2 = Transaction {
            tx_type: TransactionType::Withdrawal,
            client: 1,
            tx: 4,
            amount: 20.0,
        };

        account.handle_withdrawal(tx1).unwrap();
        assert_eq!(account.account_details.available_funds, 70.0);
        assert_eq!(account.account_details.total_funds, 70.0);
        assert!(account.account_transaction_archive.history.contains(&3));

        account.handle_withdrawal(tx2).unwrap();
        assert_eq!(account.account_details.available_funds, 50.0);
        assert_eq!(account.account_details.total_funds, 50.0);
        assert!(account.account_transaction_archive.history.contains(&3));
        assert!(account.account_transaction_archive.history.contains(&4));
    }

    /// A withdrawal with zero amount leaves balances unchanged.
    /// The transaction ID is still recorded in the account's history.
    /// Ensures zero-value withdrawals are logged but do not affect funds.
    #[test]
    fn test_zero_amount_withdrawal() {
        let mut account = sample_account_with_balance(100.0);
        let tx = Transaction {
            tx_type: TransactionType::Withdrawal,
            client: 1,
            tx: 5,
            amount: 0.0,
        };
        account.handle_withdrawal(tx).unwrap();

        assert_eq!(account.account_details.available_funds, 100.0);
        assert_eq!(account.account_details.total_funds, 100.0);
        assert!(account.account_transaction_archive.history.contains(&5));
    }

    /// Very large withdrawal works as long as there are enough funds.
    /// Confirms that extreme values are handled safely.
    /// Ensures the system can process high-value withdrawals.
    #[test]
    fn test_large_withdrawal() {
        let mut account = sample_account_with_balance(1e12);
        let tx = Transaction {
            tx_type: TransactionType::Withdrawal,
            client: 1,
            tx: 6,
            amount: 5e11,
        };
        account.handle_withdrawal(tx).unwrap();

        assert_eq!(account.account_details.available_funds, 5e11);
        assert_eq!(account.account_details.total_funds, 5e11);
    }

    /// Duplicate withdrawal transaction IDs do not update balances a second time.
    /// Instead, a warning is logged and history does not change.
    /// Confirms that duplicate transactions are ignored safely.
    #[test]
    fn test_duplicate_withdrawal_transaction_id() {
        let mut account = sample_account_with_balance(100.0);

        let tx1 = Transaction {
            tx_type: TransactionType::Withdrawal,
            client: 1,
            tx: 7,
            amount: 25.0,
        };
        let tx2 = Transaction {
            tx_type: TransactionType::Withdrawal,
            client: 1,
            tx: 7, // same tx id
            amount: 20.0,
        };

        account.handle_withdrawal(tx1).unwrap();
        assert!(account.handle_withdrawal(tx2).is_err());
        // Only the first withdrawal should apply
        assert_eq!(account.account_details.available_funds, 75.0);
        assert_eq!(account.account_details.total_funds, 75.0);

        // History contains tx ID once
        assert_eq!(account.account_transaction_archive.history.len(), 1);
        assert!(account.account_transaction_archive.history.contains(&7));

        // Details match the first withdrawal
        assert_eq!(
            account.account_transaction_archive.details.get(&7),
            Some(&(25.0, TransactionType::Withdrawal))
        );
    }
}
