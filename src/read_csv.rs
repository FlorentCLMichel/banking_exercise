use std::fs::File;
use std::io::{ prelude::*, BufReader };
#[cfg(feature = "atty")]
use atty::Stream;
use crate::client::*;
use crate::transaction::*;
use crate::style::warning_style;


/// Open a csv file and execute all the transactions
pub fn execute_transactions_from_csv(clients_map: &mut ClientMap, file_name: &str) 
    -> Result<(), Box<dyn std::error::Error>>
{

    // check if stderr is a terminal
    let stderr_is_term = atty::is(Stream::Stderr);

    // open the file using a buffer
    let reader = BufReader::new(File::open(file_name)?);

    // iterate over the lines
    for (n_line, line) in reader.lines().enumerate() {

        let line = line?;

        // if the line i empty, go to the next one
        if line.is_empty() { continue; }

        // parse the line, printing a warning if it is invalid
        if let Ok((transaction_id, client_id, transaction)) = parse_line(&line, n_line, stderr_is_term) {

            // if the client is not already in clients_map, add it
            if !(clients_map.contains_key(&client_id)) {

                // We know that the map does not contain this client ID, so the insert function
                // will not return an error
                clients_map.insert(client_id, Client::default()).unwrap();
            }

            // execute the transaction
            clients_map.execute_transaction(transaction_id, client_id, transaction, stderr_is_term)?;
        } else {
            // print the warning if the line number is not zero
            if n_line > 0 {
                let warning = format!("{} (line {})", InvalidTransactionLineWarning {}, n_line);
                eprintln!("{}", warning_style(warning, stderr_is_term));
            }
        }
    }
    Ok(())
}


/// a warning type for an invalid line
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidTransactionLineWarning {}

impl std::fmt::Display for InvalidTransactionLineWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid transaction line encountered")
    }
}


fn parse_line(line: &str, n_line: usize, stderr_is_term: bool) 
    -> Result<(TransactionId, ClientId, Transaction), InvalidTransactionLineWarning> 
{
    // split the line
    let mut fields = line.split(',');

    // parse the transaction
    let parsed = match fields.next() {
        Some("deposit") => parse_deposit(&mut fields)?,
        Some("withdrawal") => parse_withdrawal(&mut fields)?,
        Some("dispute") => parse_dispute(&mut fields)?,
        Some("resolve") => parse_resolve(&mut fields)?,
        Some("chargeback") => parse_chargeback(&mut fields)?,
        _ => return Err(InvalidTransactionLineWarning {})
    };

    // print a warning if there is more data on the same line
    if fields.next().is_some() {
        let warning = format!("Additional data on line {}", n_line);
        eprintln!("{}", warning_style(warning, stderr_is_term));
    }

    Ok(parsed)
}


fn parse_dispute(fields: &mut std::str::Split<char>) 
    -> Result<(TransactionId, ClientId, Transaction), InvalidTransactionLineWarning> 
{
    let (transaction_id, client_id) = parse_ids(fields)?;
    Ok((TransactionId::default(), client_id, Transaction::Dispute(transaction_id)))
}


fn parse_resolve(fields: &mut std::str::Split<char>) 
    -> Result<(TransactionId, ClientId, Transaction), InvalidTransactionLineWarning> 
{
    let (transaction_id, client_id) = parse_ids(fields)?;
    Ok((TransactionId::default(), client_id, Transaction::Resolve(transaction_id)))
}


fn parse_chargeback(fields: &mut std::str::Split<char>) 
    -> Result<(TransactionId, ClientId, Transaction), InvalidTransactionLineWarning> 
{
    let (transaction_id, client_id) = parse_ids(fields)?;
    Ok((TransactionId::default(), client_id, Transaction::Chargeback(transaction_id)))
}


fn parse_deposit(fields: &mut std::str::Split<char>) 
    -> Result<(TransactionId, ClientId, Transaction), InvalidTransactionLineWarning> 
{
    let (transaction_id, client_id) = parse_ids(fields)?;
    let amount: f64;
    match fields.next() {
        Some(s) => match s.trim().parse::<f64>() {
            Ok(n) => amount = n,
            Err(_) => return Err(InvalidTransactionLineWarning {})
        },
        None => return Err(InvalidTransactionLineWarning {})
    }
    Ok((transaction_id, client_id, Transaction::Deposit(amount)))
}


fn parse_withdrawal(fields: &mut std::str::Split<char>) 
    -> Result<(TransactionId, ClientId, Transaction), InvalidTransactionLineWarning> 
{
    let (transaction_id, client_id) = parse_ids(fields)?;
    let amount: f64;
    match fields.next() {
        Some(s) => match s.trim().parse::<f64>() {
            Ok(n) => amount = n,
            Err(_) => return Err(InvalidTransactionLineWarning {})
        },
        None => return Err(InvalidTransactionLineWarning {})
    }
    Ok((transaction_id, client_id, Transaction::Withdrawal(amount)))
}

fn parse_ids(fields: &mut std::str::Split<char>) 
    -> Result<(TransactionId, ClientId), InvalidTransactionLineWarning>
{

    let transaction_id: TransactionId;
    let client_id: ClientId;
    
    match fields.next() {
        Some(s) => match s.trim().parse::<u16>() {
            Ok(id) => client_id = ClientId(id),
            Err(_) => return Err(InvalidTransactionLineWarning {})
        },
        None => return Err(InvalidTransactionLineWarning {})
    }

    match fields.next() {
        Some(s) => match s.trim().parse::<u32>() {
            Ok(id) => transaction_id = TransactionId(id),
            Err(_) => return Err(InvalidTransactionLineWarning {})
        },
        None => return Err(InvalidTransactionLineWarning {})
    }
    
    Ok((transaction_id, client_id))
}


#[cfg(test)]
mod tests {
    
    use super::*;

    #[test]
    fn parse_line_1() {
        let line = "deposit, 1, 2, 10000";
        let parsed_line = parse_line(line, 0, false);
        assert_eq!(Ok((TransactionId(2), ClientId(1), Transaction::Deposit(10000.))), 
                   parsed_line);
    }
    
    #[test]
    fn parse_line_2() {
        let line = "withdrawal, 1, 2, 10000";
        let parsed_line = parse_line(line, 0, false);
        assert_eq!(Ok((TransactionId(2), ClientId(1), Transaction::Withdrawal(10000.))), 
                   parsed_line);
    }
    
    #[test]
    fn parse_line_3() {
        let line = "dispute, 1, 2";
        let parsed_line = parse_line(line, 0, false);
        assert_eq!(Ok((TransactionId::default(), ClientId(1), Transaction::Dispute(TransactionId(2)))), 
                   parsed_line);
    }
    
    #[test]
    fn parse_line_4() {
        let line = "resolve, 1, 2";
        let parsed_line = parse_line(line, 0, false);
        assert_eq!(Ok((TransactionId::default(), ClientId(1), Transaction::Resolve(TransactionId(2)))), 
                   parsed_line);
    }
    
    #[test]
    fn parse_line_5() {
        let line = "chargeback, 1, 2";
        let parsed_line = parse_line(line, 0, false);
        assert_eq!(Ok((TransactionId::default(), ClientId(1), Transaction::Chargeback(TransactionId(2)))), 
                   parsed_line);
    }
}
