/// a structure storing transactions
///
/// Transactions without IDs will be assigned the ID 0
#[derive(Debug, PartialEq)]
pub enum Transaction {
    Deposit(f64),
    Withdrawal(f64),
    Dispute(TransactionId),
    Resolve(TransactionId),
    Chargeback(TransactionId)
}


/// a transaction ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionId(pub u32);

impl Default for TransactionId {
    fn default() -> Self {
        TransactionId(0)
    }
}
