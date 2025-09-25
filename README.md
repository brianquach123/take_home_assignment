This is an implementation of a basic payments engine.

The payments engine can be run using:
`cargo run -- <transaction_file>.csv`

If given more time, I would additionally:
- Build an additional CLI flag to generate a configurable test CSV for easier local testing. I wrote a helper function in lieu of doing this.
- Use the Newtype pattern on ID-based fields that take an integral type. This is for developer ergonomics so that there is a guarantee that a parameter being
passed to a function is a guaranteed type, not just a u16 or u32 we'll interpret as a client or transaction ID.
- Introduce a configurable worker thread field in the `PaymentsEngine`'s struct definition to concurrently divide and portion the CSV for processing. One 
approach involves setting this field to X. Then when the engine begins processing payment transactions, spawn X threads responsible for every first, second,..X-th row
in the input CSV.
- Use secure values instead of a u16 and u32 for ID types. If we're not using a database, I would choose to replace each of these with a v4 UUID to ensure uniqueness
among values. It's better than having an sequential ID field that can be susceptible to replay attacks. If we're planning to use a database, I would choose to implement 
some ID generation mechanism to create an ID for client and transaction IDs if this field's planned to be used as a primary key in the database.

-Implement a struct called `TransactionDetail` to replace the `(f64, TransactionType)` tuple in the `ClientTransactionArchive` struct for readability.
-DRY the non-disputed transactions in the `ClientTransactionArchive`. That is, instead of having this:
```
#[derive(Debug, Default)]
pub struct ClientTransactionArchive {
    /// The set of transaction IDs associated with this account.
    pub history: BTreeSet<u32>,
    /// Map of the set of transaction IDs to (amount, type of transaction)
    /// for this account.
    pub details: HashMap<u32, (f64, TransactionType)>,
    /// The set of disputed transactions for this account.
    pub disputes: BTreeSet<u32>,
}
```
I would instead update this struct to this:
```
#[derive(Debug, Default)]
pub struct ClientTransactionArchive {
    /// Map of the set of transaction IDs to (amount, type of transaction)
    /// for this account.
    pub tx_details: HashMap<u32, (f64, TransactionType)>,
    /// The set of disputed transactions for this account.
    pub disputes: BTreeSet<u32>,
}
```
This reduces the number of fields to maintain, and conceptually represents 
the account's past transactions. I didn't notice this until later during the
development process.
