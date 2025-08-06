use concilium_core::transaction::{BroadcastTransactionTemp, Transaction};

pub trait BroadcastTransactionTempSupport {
    fn new(transaction: Transaction, signature: [u8; 96]) -> BroadcastTransactionTemp;
    fn get_transaction(&self) -> &Transaction;
    fn get_signature(&self) -> &[u8; 96];
}

impl BroadcastTransactionTempSupport for BroadcastTransactionTemp {
    fn new(transaction: Transaction, signature: [u8; 96]) -> BroadcastTransactionTemp {
        Self {
            transaction,
            signature
        }
    }

    fn get_transaction(&self) -> &Transaction {
        &self.transaction
    }

    fn get_signature(&self) -> &[u8; 96] {
        &self.signature
    }
}