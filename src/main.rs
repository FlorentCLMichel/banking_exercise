mod client;
mod style;
mod transaction;
mod read_csv;

use std::env;
use client::ClientMap;
use read_csv::execute_transactions_from_csv;

fn main() {
    
    // get an iterator to the command-line arguments
    let mut args = env::args();

    // skip the first one
    args.next();

    // get the file name, or panic if it is not provided
    let file_name = args.next().expect("ERROR: No file name provided");

    // create a new empty list of clients
    let mut client_list = ClientMap::default();

    // execute the transactions from the file
    execute_transactions_from_csv(&mut client_list, &file_name).unwrap();

    // print the client data
    print!("{}", client_list);
}
