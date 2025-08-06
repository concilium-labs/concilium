use concilium_core::transaction::{TXInput, TXOutput, Transaction};

pub trait TransactionSupport {
    fn new(txid: [u8; 32], from: [u8; 32], signature: [u8; 64], nonce: u64, created_at: i64, vin: Vec<TXInput>, vout: Vec<TXOutput>) -> Transaction;
    fn get_txid(&self) -> &[u8; 32];
    fn get_from(&self) -> &[u8; 32];
    fn get_signature(&self) -> &[u8; 64];
    fn get_nonce(&self) -> u64;
    fn get_created_at(&self) -> i64;
    fn get_vin(&self) -> &Vec<TXInput>;
    fn get_vout(&self) -> &Vec<TXOutput>;
    fn set_txid(&mut self, txid: [u8; 32]);
    fn set_from(&mut self, from: [u8; 32]);
    fn set_signature(&mut self, signature: [u8; 64]);
    fn set_vin(&mut self, vin: Vec<TXInput>);
    fn vin_push(&mut self, tx_input: TXInput);
    fn vin_pop(&mut self) -> Option<TXInput>;
    fn set_vout(&mut self, vout: Vec<TXOutput>);
    fn vout_push(&mut self, tx_output: TXOutput);
    fn vout_pop(&mut self) -> Option<TXOutput>;
    fn set_created_at(&mut self, created_at: i64);
}

impl TransactionSupport for Transaction {
    fn new(txid: [u8; 32], from: [u8; 32], signature: [u8; 64], nonce: u64, created_at: i64, vin: Vec<TXInput>, vout: Vec<TXOutput>) -> Transaction {
        Transaction {
            txid,
            from,
            signature,
            nonce,
            created_at,
            vin,
            vout,
        }
    }

    fn get_txid(&self) -> &[u8; 32] {
        &self.txid
    }
    
    fn get_from(&self) -> &[u8; 32] {
        &self.from
    }

    fn get_signature(&self) -> &[u8; 64] {
        &self.signature
    }

    fn get_nonce(&self) -> u64 {
        self.nonce
    }

    fn get_created_at(&self) -> i64 {
        self.created_at
    }

    fn get_vin(&self) -> &Vec<TXInput> {
        &self.vin
    }

    fn get_vout(&self) -> &Vec<TXOutput> {
        &self.vout
    }

    fn set_txid(&mut self, txid: [u8; 32]) {
        self.txid = txid;
    }

    fn set_from(&mut self, from: [u8; 32]) {
        self.from = from;
    }

    fn set_signature(&mut self, signature: [u8; 64]) {
        self.signature = signature;
    }

    fn set_vin(&mut self, vin: Vec<TXInput>) {
        self.vin = vin;
    }

    fn vin_push(&mut self, tx_input: TXInput) {
        self.vin.push(tx_input);
    }

    fn vin_pop(&mut self) -> Option<TXInput> {
        self.vin.pop()
    }

    fn set_vout(&mut self, vout: Vec<TXOutput>) {
        self.vout = vout;
    }

    fn vout_push(&mut self, tx_output: TXOutput) {
        self.vout.push(tx_output);
    }

    fn vout_pop(&mut self) -> Option<TXOutput> {
        self.vout.pop()
    }

    fn set_created_at(&mut self, created_at: i64) {
        self.created_at = created_at;
    }
}
