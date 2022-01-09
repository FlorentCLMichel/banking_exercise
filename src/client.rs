use std::collections::{ HashMap, HashSet };
use crate::transaction::*;
use crate::style::warning_style;
use itertools::Itertools; // to sort the client hashmap

/// information about a client
///
/// We use 64-bit floating-point numbers for the amounts.Using 32-bit numbers would be enough to
/// give a precision up to four places past the decimal for values up to about 10,000,000. We
/// choose a higher precision to be able to deal with larger numbers if necessary.
#[derive(Debug)]
pub struct Client {
    available: f64, 
    held: f64, 
    locked: bool, 
    history: HashMap<TransactionId, Transaction>,
    disputed_transactions: HashSet<TransactionId>,
}


/// type used for the client ID
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ClientId(pub u16);


/// a hashmap type relating client IDs to clients
#[derive(Debug)]
pub struct ClientMap(HashMap<ClientId, Client>);


/// a warning triggered when overriding an existing client with a new one with the same ID
#[derive(Debug)]
pub struct ExistingClientWarning(Client);


impl Client {

    /// Create a new `[Client]`
    ///
    /// # Example
    ///
    /// ```
    /// use banking_exercise::client::Client;
    ///
    /// // a rich client just joined our bank!
    /// let available_fund: f64 = 10_000_000_000.;
    ///
    /// // the client just joined, so there is presumably no dispute yet
    /// let held_fund: f64 = 0.;
    ///
    /// // no reason to lock the client's account
    /// let locked = false;
    ///
    /// let new_client = Client::new(available_fund, held_fund, locked);
    /// ```
    pub fn new(available: f64, held: f64, locked: bool) -> Self {
        Client { available, held, locked, 
                 history: HashMap::new(), 
                 disputed_transactions: HashSet::new() }
    }
    
    // add to the available funds
    fn add_to_available(&mut self, amount: f64) {
        self.available += amount;
    }
    
    // move from the available funds to the held ones
    fn move_to_held(&mut self, amount: f64) {
        self.available -= amount;
        self.held += amount;
    }
    
    fn remove_from_held(&mut self, amount: f64) {
        self.held -= amount;
    }
    
    // lock the account
    fn lock(&mut self) {
        self.locked = true;
    }
    
    // add a transaction to the history
    fn add_to_history(&mut self, transaction_id: TransactionId, transaction: Transaction) {
        self.history.insert(transaction_id, transaction);
    }
    
    // dispute a transaction
    fn dispute(&mut self, transaction_id: TransactionId) {

        // check if the transaction exists and is not already disputed
        if self.history.contains_key(&transaction_id) 
            && !self.disputed_transactions.contains(&transaction_id) {

            // set the transaction as disputed
            self.disputed_transactions.insert(transaction_id); 

            // if the transaction is a deposit, move the funds to held
            if let Some(&Transaction::Deposit(amount)) = self.history.get(&transaction_id) {
                self.move_to_held(amount);
            }
        }
    }
    
    // resolve a disputed transaction
    fn resolve(&mut self, transaction_id: TransactionId) {
        
        // check if the transaction exists and is disputed
        if self.history.contains_key(&transaction_id)
            && self.disputed_transactions.contains(&transaction_id) {

            // set the transaction as undisputed
            self.disputed_transactions.remove(&transaction_id); 

            // if the transaction is a deposit, move the funds back to available
            if let Some(&Transaction::Deposit(amount)) = self.history.get(&transaction_id) {
                self.move_to_held(-amount);
            }
        }
    }
    
    // chargeback a disputed transaction
    fn chargeback(&mut self, transaction_id: TransactionId) {
        
        // check if the transaction exists and is disputed
        if self.history.contains_key(&transaction_id) 
            && self.disputed_transactions.contains(&transaction_id) {

            // set the transaction as undisputed
            self.disputed_transactions.remove(&transaction_id); 

            // if the transaction is a deposit, remove the funds from the held funds
            if let Some(&Transaction::Deposit(amount)) = self.history.get(&transaction_id) {
                self.remove_from_held(amount);
            }

            // lock the account
            self.lock();
        }
    }
}


impl Default for Client {
    fn default() -> Self {
        Client::new(0., 0., false)
    }
}


impl std::fmt::Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let total = self.available + self.held;
        write!(f, "{}, {}, {}, {}", self.available, self.held, total, self.locked)
    }
}


impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


impl ClientMap {
 
    /// check if a key is in te map
    pub fn contains_key(&self, key: &ClientId) -> bool {
        self.0.contains_key(key)
    }

    /// insert a new `Client` and its `ClientId`
    ///
    /// # Example
    /// 
    /// ```
    /// use banking_exercise::client::*;
    ///
    /// // define a new empty ClientMap
    /// let mut clients_map = ClientMap::default();
    ///
    /// // Our firt client has just opened an account! 
    /// // Let's give them the index ID.
    /// let client_id = ClientId(1);
    ///
    /// // Our first client deposits 100_000 RustyDollars in their account.
    /// let client = Client::new(100_000., 0., false);
    ///
    /// // add the client to the map
    /// clients_map.insert(client_id, client);
    /// ```
    pub fn insert(&mut self, id: ClientId, client: Client) -> Result<(), ExistingClientWarning> {
        match self.0.insert(id, client) {
            None => Ok(()), 
            Some(client) => Err(ExistingClientWarning(client))
        }
    }

    /// get a reference to a `[Client]` from an ID if such a client exists
    ///
    /// # Return type
    ///
    /// This function returns an `Option<&Client>`, of the form `Some(client)` if `client` has the
    /// right ID, or `None` if no such client exists.
    fn get(&self, id: &ClientId) -> Option<&Client> {
        self.0.get(id)
    }
    
    /// get a mutable reference to a `[Client]` from an ID if such a client exists
    ///
    /// # Return type
    ///
    /// This function returns an `Option<&mut Client>`, of the form `Some(client)` if `client` has 
    /// the right ID, or `None` if no such client exists.
    fn get_mut(&mut self, id: &ClientId) -> Option<&mut Client> {
        self.0.get_mut(id)
    }

    /// exxecute a transaction
    ///
    /// # Errors
    ///
    /// This function returns a `[ClientNotFoundError]` if the client is not found or a
    /// `[LockedAccountError]` if their account is locked.
    /// 
    /// # Example
    /// 
    /// ```
    /// use banking_exercise::client::*;
    /// use banking_exercise::transaction::*;
    ///
    /// // Create an empty ClientMap
    /// let mut clients_map = ClientMap::default();
    ///
    /// // Add a new client with an empty account and ID 1
    /// clients_map.insert(ClientId(1), Client::new(0., 0., false));
    /// 
    /// // Execute a transaction: deposit
    /// clients_map.execute_transaction(TransactionId(1), ClientId(1), 
    ///                                 Transaction::Deposit(10_000.),
    ///                                 false);
    /// 
    /// // Dispute the transaction
    /// clients_map.execute_transaction(TransactionId::default(), ClientId(1), 
    ///                                 Transaction::Dispute(TransactionId(1)),
    ///                                 false);
    /// 
    /// // Resolve the transaction
    /// clients_map.execute_transaction(TransactionId::default(), ClientId(1), 
    ///                                 Transaction::Resolve(TransactionId(1)),
    ///                                 false);
    /// ```
    pub fn execute_transaction(&mut self, 
                           transaction_id: TransactionId, 
                           client_id: ClientId, 
                           transaction: Transaction,
                           is_term: bool)
        -> Result<(), Box<dyn std::error::Error>> 
    {
        // get a reference to the client, or raise a `[ClientNotFoundError]` if the client does not
        // exist 
        if let Some(mut_ref_to_client) = self.get_mut(&client_id) {

            // check that the account is not locked
            if mut_ref_to_client.locked { return Err(Box::new(LockedAccountError {})); }

            // if the transaction is a deposit or Withdrawal, check that its ID is not already in
            // the client history
            match &transaction
            {
                Transaction::Deposit(_) | Transaction::Withdrawal(_) => 
                    if mut_ref_to_client.history.contains_key(&transaction_id) {
                        let warning = format!("Warning: More than one transaction with client ID {} and transaction ID {}; all but the first will be ignored", 
                                              client_id, transaction_id.0);
                        eprintln!("{}", warning_style(warning, is_term));
                        return Ok(());
                    }
                _ => ()
            }

            // execute the transaction
            match transaction {
                Transaction::Deposit(amount) => mut_ref_to_client.add_to_available(amount),
                Transaction::Withdrawal(amount) => mut_ref_to_client.add_to_available(-amount),
                Transaction::Dispute(id) => mut_ref_to_client.dispute(id), 
                Transaction::Resolve(id) => mut_ref_to_client.resolve(id),
                Transaction::Chargeback(id) => mut_ref_to_client.chargeback(id), 
            }
            
            // add the transaction to the client history
            mut_ref_to_client.add_to_history(transaction_id, transaction);
            
            Ok(())
    
        } else {
            Err(Box::new(ClientNotFoundError(client_id)))
        }
        
    }
}


impl std::default::Default for ClientMap {
    fn default() -> Self {
        ClientMap(HashMap::<ClientId, Client>::new())
    }
}


impl std::fmt::Display for ClientMap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let first_line = "client, available, held, total, locked";
        writeln!(f, "{}", first_line)?;
        for key in self.0.keys().sorted() {
            if let Some(client) = self.get(key) {
                writeln!(f, "{}, {}", key, client)?;
            }
        }
        Ok(())
    }
}


/// an error raised when a client is not found
#[derive(Debug, Clone)]
pub struct ClientNotFoundError(ClientId);

impl std::fmt::Display for ClientNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Client {} not found", self.0.0)
    }
}

impl std::error::Error for ClientNotFoundError {}


/// an error raised when trying to do a transaction on a locked account
#[derive(Debug, Clone)]
pub struct LockedAccountError {}

impl std::fmt::Display for LockedAccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "The client account is locked")
    }
}

impl std::error::Error for LockedAccountError {}



#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn test_add_funds_1() {
 
        // Our new client deposits 2_022 RustyDollars in their account.
        let mut client = Client::new(2_022., 0., false);
        
        // Our client just remembered they own 100_000 RustyDollars worth of RSACoin, the latest
        // craze among classical tech investors. Unfortunately, cryptographic functions based on RSA
        // are not quantum secure, and they risk osing most of their investment as soon as a 
        // powerful enough quantum computer is built. They thus decide to sell their RSACoins and 
        // deposit the money in their account
        client.add_to_available(100_000.);
        
        // check the client info
        assert_eq!("102022, 0, 102022, false".to_string(), format!("{}", client));
    }

    #[test]
    fn test_lock_1() {
 
        // Our new client deposits 9e99 RustyDollars in their account.
        let mut client = Client::new(9e+99_f64, 0., false);
        
        // Wait a minute... This is more than the number of atoms in the known universe—no one can
        // be quite rich enough to have that many RustyDolars! Surely there is something frudulent
        // here. Let's lock the account and investigate!
        client.lock();
    
        // check the client info
        assert_eq!(format!("{}, 0, {}, true", 9e+99_f64, 9e+99_f64), format!("{}", client));
    }

    #[test]
    fn test_move_to_held_1() {
 
        // Our new client deposits 2_023 RustyDollars in their account.
        let mut client = Client::new(2_023., 0., false);
       
        // Our UberTransactionChecker™ system, using the latest Fourier Transformer Networks, has
        // detected a possible error: depositing 2,023 RustyDollars now sounds one year early! We
        // pre-emptively correct this likely error by moving 1 RustyDollar from the available funds 
        // to the held ones, and make a note to contact the client to enquire about this.
        client.move_to_held(1.);
       
        // check the client info
        assert_eq!("2022, 1, 2023, false".to_string(), format!("{}", client));
    }

    #[test]
    fn add_to_history() {

        // Our new client deposits 2_022 RustyDollars in their account.
        let mut client = Client::new(2_022., 0., false);
        
        // Let us add this first transaction to their history, with the ID 1
        client.add_to_history(TransactionId(1), Transaction::Deposit(2_022.));
    }

    #[test]
    fn test_get() {
        // define a new empty ClientMap
        let mut clients_map = ClientMap::default();
       
        // Our first client has just opened an account! 
        // Let's give them the index ID.
        let client_id = ClientId(1);
       
        // Our first client deposits 100_000 RustyDollars in their account.
        let client = Client::new(100_000., 0., false);
       
        // add the client to the map
        clients_map.insert(client_id, client).unwrap();
        
        // get a reference to our client
        let opt_ref_to_client = clients_map.get(&ClientId(1));
       
        // check that the result is not None
        if let Some(ref_to_client) = opt_ref_to_client {
            
            // check the client info
            assert_eq!("100000, 0, 100000, false".to_string(), format!("{}", ref_to_client));
        
        } else {
            panic!("Could not find our client");
        };
       
        // try to get a reference to a client which does not exist 
        if let Some(_) = clients_map.get(&ClientId(2)) {
            panic!("Found a client which does not exist");
        }
    }

    #[test]
    fn test_get_mut_1() {
    
        // define a new empty ClientMap
        let mut clients_map = ClientMap::default();
        
        // Our first client has just opened an account! 
        // Let's give them the index ID.
        let client_id = ClientId(1);
        
        // Our first client deposits 100_000 RustyDollars in their account.
        let client = Client::new(100_000., 0., false);
        
        // add the client to the map
        clients_map.insert(client_id, client).unwrap();
        
        // get a reference to our client
        let opt_mut_ref_to_client = clients_map.get_mut(&ClientId(1));
        
        // check that the result is not None
        if let Some(mut_ref_to_client) = opt_mut_ref_to_client {
            
            // as a welcome gift, let's give away 100 RustyDollars to our client!
            mut_ref_to_client.add_to_available(100.);
        
            // check the client info
            assert_eq!("100100, 0, 100100, false".to_string(), format!("{}", mut_ref_to_client));
        
        } else {
            panic!("Could not find our client");
        };
        
        // try to get a reference to a client which does not exist 
        if let Some(_) = clients_map.get_mut(&ClientId(2)) {
            panic!("Found a client which does not exist");
        }
    }
    
    #[test]
    fn deposit_1() {

        // Create an empty ClientMap
        let mut clients_map = ClientMap::default();

        // Add a new client with an empty account and ID 1
        clients_map.insert(ClientId(1), Client::new(0., 0., false)).unwrap();
        
        // Execute a transaction: deposit
        clients_map.execute_transaction(TransactionId(1), ClientId(1), 
                                        Transaction::Deposit(2_022.),
                                        false).unwrap();

        // check the client info
        if let Some(ref_to_client) = clients_map.get(&ClientId(1)) {
            assert_eq!("2022, 0, 2022, false".to_string(), 
                       format!("{}", ref_to_client));
        } else {
            panic!("Client not found!");
        }
    }
    
    #[test]
    fn withdrawal_1() {

        // Create an empty ClientMap
        let mut clients_map = ClientMap::default();

        // Add a new client with an empty account and ID 1
        clients_map.insert(ClientId(1), Client::new(0., 0., false)).unwrap();
        
        // Execute a transaction: deposit
        clients_map.execute_transaction(TransactionId(1), ClientId(1), 
                                        Transaction::Deposit(12_022.),
                                        false).unwrap();
        
        // Execute a transaction: withdrawal
        clients_map.execute_transaction(TransactionId(2), ClientId(1), 
                                        Transaction::Withdrawal(2_022.),
                                        false).unwrap();

        // check the client info
        if let Some(ref_to_client) = clients_map.get(&ClientId(1)) {
            assert_eq!("10000, 0, 10000, false".to_string(), 
                       format!("{}", ref_to_client));
        } else {
            panic!("Client not found!");
        }
    }
    
    #[test]
    fn dispute_1() {

        // Create an empty ClientMap
        let mut clients_map = ClientMap::default();

        // Add a new client with an empty account and ID 1
        clients_map.insert(ClientId(1), Client::new(0., 0., false)).unwrap();
        
        // Execute a transaction: deposit
        clients_map.execute_transaction(TransactionId(1), ClientId(1), 
                                        Transaction::Deposit(10_000.),
                                        false).unwrap();
        
        // Dispute the transaction
        clients_map.execute_transaction(TransactionId::default(), ClientId(1), 
                                        Transaction::Dispute(TransactionId(1)),
                                        false).unwrap();

        // check the client info
        if let Some(ref_to_client) = clients_map.get(&ClientId(1)) {
            assert_eq!("0, 10000, 10000, false".to_string(), 
                       format!("{}", ref_to_client));
        } else {
            panic!("Client not found!");
        }
    }
    
    #[test]
    // disputing a non-existent transaction should not change the client information
    fn dispute_2() {

        // Create an empty ClientMap
        let mut clients_map = ClientMap::default();

        // Add a new client with an empty account and ID 1
        clients_map.insert(ClientId(1), Client::new(0., 0., false)).unwrap();
        
        // Execute a transaction: deposit
        clients_map.execute_transaction(TransactionId(1), ClientId(1), 
                                        Transaction::Deposit(10_000.),
                                        false).unwrap();
        
        // Dispute the transaction
        clients_map.execute_transaction(TransactionId::default(), ClientId(1), 
                                        Transaction::Dispute(TransactionId(2)),
                                        false).unwrap();

        // check the client info
        if let Some(ref_to_client) = clients_map.get(&ClientId(1)) {
            assert_eq!("10000, 0, 10000, false".to_string(), 
                       format!("{}", ref_to_client));
        } else {
            panic!("Client not found!");
        }
    }
    
    #[test]
    fn resolve_1() {

        // Create an empty ClientMap
        let mut clients_map = ClientMap::default();

        // Add a new client with an empty account and ID 1
        clients_map.insert(ClientId(1), Client::new(0., 0., false)).unwrap();
        
        // Execute a transaction: deposit
        clients_map.execute_transaction(TransactionId(1), ClientId(1), 
                                        Transaction::Deposit(10_000.),
                                        false).unwrap();
        
        // Dispute the transaction
        clients_map.execute_transaction(TransactionId::default(), ClientId(1), 
                                        Transaction::Dispute(TransactionId(1)),
                                        false).unwrap();
        
        // Resolve the transaction
        clients_map.execute_transaction(TransactionId::default(), ClientId(1), 
                                        Transaction::Resolve(TransactionId(1)),
                                        false).unwrap();

        // check the client info
        if let Some(ref_to_client) = clients_map.get(&ClientId(1)) {
            assert_eq!("10000, 0, 10000, false".to_string(), 
                       format!("{}", ref_to_client));
        } else {
            panic!("Client not found!");
        }
    }
    
    #[test]
    // resolving a transaction which is not disputed should not change the client info
    fn resolve_2() {

        // Create an empty ClientMap
        let mut clients_map = ClientMap::default();

        // Add a new client with an empty account and ID 1
        clients_map.insert(ClientId(1), Client::new(0., 0., false)).unwrap();
        
        // Execute a transaction: deposit
        clients_map.execute_transaction(TransactionId(1), ClientId(1), 
                                        Transaction::Deposit(10_000.),
                                        false).unwrap();
        
        // Execute a transaction: deposit
        clients_map.execute_transaction(TransactionId(2), ClientId(1), 
                                        Transaction::Deposit(5_000.),
                                        false).unwrap();
        
        // Dispute the first transaction
        clients_map.execute_transaction(TransactionId::default(), ClientId(1), 
                                        Transaction::Dispute(TransactionId(1)),
                                        false).unwrap();
        
        // Resolve the second transaction
        clients_map.execute_transaction(TransactionId::default(), ClientId(1), 
                                        Transaction::Resolve(TransactionId(2)),
                                        false).unwrap();

        // check the client info
        if let Some(ref_to_client) = clients_map.get(&ClientId(1)) {
            assert_eq!("5000, 10000, 15000, false".to_string(), 
                       format!("{}", ref_to_client));
        } else {
            panic!("Client not found!");
        }
    }
    
    #[test]
    fn chargeback_1() {

        // Create an empty ClientMap
        let mut clients_map = ClientMap::default();

        // Add a new client with an empty account and ID 1
        clients_map.insert(ClientId(1), Client::new(0., 0., false)).unwrap();
        
        // Execute a transaction: deposit
        clients_map.execute_transaction(TransactionId(1), ClientId(1), 
                                        Transaction::Deposit(10_000.),
                                        false).unwrap();
        
        // Dispute the transaction
        clients_map.execute_transaction(TransactionId::default(), ClientId(1), 
                                        Transaction::Dispute(TransactionId(1)),
                                        false).unwrap();
        
        // Chargeback
        clients_map.execute_transaction(TransactionId::default(), ClientId(1), 
                                        Transaction::Chargeback(TransactionId(1)),
                                        false).unwrap();

        // check the client info
        if let Some(ref_to_client) = clients_map.get(&ClientId(1)) {
            assert_eq!("0, 0, 0, true".to_string(), 
                       format!("{}", ref_to_client));
        } else {
            panic!("Client not found!");
        }
    }
    
    #[test]
    // chargeback on a transaction which is not disputed should not change the client info
    fn chargeback_2() {

        // Create an empty ClientMap
        let mut clients_map = ClientMap::default();

        // Add a new client with an empty account and ID 1
        clients_map.insert(ClientId(1), Client::new(0., 0., false)).unwrap();
        
        // Execute a transaction: deposit
        clients_map.execute_transaction(TransactionId(1), ClientId(1), 
                                        Transaction::Deposit(10_000.),
                                        false).unwrap();
        
        // Execute a transaction: deposit
        clients_map.execute_transaction(TransactionId(2), ClientId(1), 
                                        Transaction::Deposit(5_000.),
                                        false).unwrap();
        
        // Dispute the first transaction
        clients_map.execute_transaction(TransactionId::default(), ClientId(1), 
                                        Transaction::Dispute(TransactionId(1)),
                                        false).unwrap();
        
        // Resolve the second transaction
        clients_map.execute_transaction(TransactionId::default(), ClientId(1), 
                                        Transaction::Chargeback(TransactionId(2)),
                                        false).unwrap();

        // check the client info
        if let Some(ref_to_client) = clients_map.get(&ClientId(1)) {
            assert_eq!("5000, 10000, 15000, false".to_string(), 
                       format!("{}", ref_to_client));
        } else {
            panic!("Client not found!");
        }
    }
}
