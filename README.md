# Dealing with Banking Data in Rust: a Proof of Concept

This crate is a simple proof of concept for updating banking data in Rust, focusing on the core logic. 

**This is a proof of concept only, not to use with real data. This crate does not implement any encryption nor other data protection mechanism, and is thus not suitable for dealing with private or otherwise secret data.** 

# Requirements

Building this crate requires: 

* A Rust 2021 compiler, and optionally `cargo`. Instructions for how to install the latest stable Rust toolchain can be found on the [rust-lang](https://www.rust-lang.org/) website.
* The `itertools` version 0.10) and `atty` (version 0.2) crates (they may be downloaded by cargo at compile time).

The compiled executable can be run on any terminal or terminal emulator.

# Build

The canonical way to build the executable is to run the command 

`cargo build --release`

from the crate main directory. 

# Run

The executable takes the name of a file with transactions as parameter (see below). For instance, to analyse transactions in the file `transactions.csv`, run 

`./target/release/banking_exercise transactions.csv`

Alternatively, the command

`cargo run --release -- filename`

builds and run the executable. 

By default, results are printed to `stdout` and warnings to `stderr`. They dan be redirected to files `output_file` and `error_file` by appending `> output_file` and `2> error_file` to the command. For instance, 

`./target/release/banking_exercise transactions.csv > client_data.csv`

will save the results in the file `client_data.csv`, and 

`./target/release/banking_exercise transactions.csv > client_data.csv 2> log.txt`

will additionally save the errors to `log.txt`. 

# How does it work? 

## High-level 

The executable goes through the lines of the file passed as parameter, one by one. It tries to parse each line as a valid transaction and, if successful, updates the data accordingly. If the client ID does not exist, a new `Client` instance is created before performing the transaction, with `0.` available and held funds, an unlocked account, and an empty transaction history. After the last line has been analysed, the data is printed to `stdout`. 

Warnings are printed to `stderr` if a row can not be parsed as a valid transaction or contains more fields than expected. By default, these warnings are printed in bold red. This behaviour can be overridden by building with the `no_color` feature, by compiling with the `--no-default-features` flag, or by redirecting `stderr` to a file, in which case warnings are printed using the default terminal colour and font family.

**Warning:** No warning is printed if the first line does not represent a valid transaction.

## Transaction file format

The input file should contain transactions separated by newlines, with the fields in each transaction separated by commas. Possible transactions are: `deposit`, `withdrawal`, `dispute`, `resolve`, and `chargeback`, with the following fields: 

* `deposit` or `withdrawal`: `transaction_id` (ID of the current transaction), `client_id` (ID of the client), and `amount` (amount deposited or withdrawn); 
* `dispute`, `resolve`, or `chargeback`: `transaction_id` (ID of the transaction which is disputed, resolved, or charged back) and `client_id` (ID of the client). 

## Client data

For each client, we show an ID (`u16`), amounts of available, held, and total funds (`f64`), and whether the account is locked (`bool`).

## Some implementation details

The crate defines the following structures: 

* `Transaction`: an `enum` type of the form `Deposit(amount)`, `Withdrawal(amount)`, `Dispute(transaction_id)`, `Resolve(transaction_id)`, or `Chargeback(transaction_id)`
* `Client`: a structure storing the client's ID, the available and held amounts in their account, a boolean value indicating whether the account is locked, a transaction history (implemented as a hashmap with transaction IDs as keys and transactions as values), and a list of disputed transactions (implemented as a set of transaction IDs)
* `ClientMap`: a `HashMap` with client IDs as keys and `Client`s as values
* `TransactionID`: a transaction ID (wrapper around a `u32`)
* `ClientID`: a client ID (wrapper around a `u16`)

The total amount in a client's account is not stored explicitly, but computed as the sum of the available and held amounts when needed.

Transactions without an explicit ID (`Dispute`, `Resolve`, and `Chargeback`) are assigned the ID `0`. They are not included in the client's trasaction history. 

## Assumptions

* Each client has a unique ID. 
* Two different `Deposit` or `Withdrawal` transactions for the same client have different transaction IDs.
* No explicit transaction ID is 0. 
