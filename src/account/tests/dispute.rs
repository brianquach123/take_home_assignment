/// Tests transaction dispute behavior for a ClientAccount.
/// These tests were generated using ChatGPT and then manually
/// verified.
#[cfg(test)]
mod dispute_tests {
    use crate::{
        account::client_account::ClientAccount,
        transaction::{Transaction, TransactionType},
    };

    /// Test that disputing a valid past transaction correctly moves its funds
    /// from the account's available balance into the held balance.
    #[test]
    fn test_dispute_moves_funds() {
        let mut acct: ClientAccount = ClientAccount::default();
        let deposit = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(100.0),
        };

        acct.handle_deposit(deposit).unwrap();

        let dispute = Transaction {
            tx_type: TransactionType::Dispute,
            ..deposit
        };

        acct.handle_dispute(dispute).unwrap();

        assert_eq!(acct.account_details.available_funds, 0.0);
        assert_eq!(acct.account_details.held_funds, 100.0);
    }

    /// Test that disputing a transaction that was never recorded returns an error
    /// and does not change any account balances.
    #[test]
    fn test_dispute_nonexistent_transaction() {
        let mut acct = ClientAccount::default();
        let dispute = Transaction {
            tx_type: TransactionType::Dispute,
            client: 1,
            tx: 99,
            amount: Some(50.0),
        };

        let res = acct.handle_dispute(dispute);
        assert!(res.is_err());
    }

    /// Test that disputing the same transaction more than once does not apply
    /// the effects again, so balances remain correct after the first dispute.
    #[test]
    fn test_duplicate_dispute_is_ignored() {
        let mut acct = ClientAccount::default();
        let deposit = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(100.0),
        };
        acct.handle_deposit(deposit.clone()).unwrap();

        let dispute = Transaction {
            tx_type: TransactionType::Dispute,
            ..deposit.clone()
        };

        acct.handle_dispute(dispute).unwrap();
        assert!(acct.handle_dispute(dispute).is_err()); // second dispute ignored

        assert_eq!(acct.account_details.held_funds, 100.0);
        assert!(acct.account_transaction_archive.disputes.contains(&1));
        assert_eq!(acct.account_transaction_archive.disputes.len(), 1);
    }

    /// Test that a successfully disputed transaction gets recorded
    /// in the account's set of disputed transaction IDs.
    #[test]
    fn test_dispute_adds_to_disputes_set() {
        let mut acct = ClientAccount::default();
        let deposit = Transaction {
            tx_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(100.0),
        };
        acct.handle_deposit(deposit).unwrap();

        let dispute = Transaction {
            tx_type: TransactionType::Dispute,
            ..deposit.clone()
        };

        acct.handle_dispute(dispute).unwrap();
        assert!(acct.account_transaction_archive.disputes.contains(&1));
    }
}
