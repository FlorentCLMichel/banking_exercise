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

The executable goes through the lines of the file passed as parameter, one by one. It tries to parse each line as a valid transaction and, if successful, updates the data accordingly. After the last line has been analysed, the data is printed to `stdout`. 

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

* `Transaction`: 
* `Client`: 
* `ClientMap`: 
* `TransactionID`:  
* `ClientID`:  
