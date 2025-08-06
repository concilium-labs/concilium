use concilium_core::transaction::TXInput;

pub trait TXInputSupport {
    fn new(txid: [u8; 32], vout: usize) -> TXInput;
    fn get_txid(&self) -> &[u8; 32];
    fn get_vout(&self) -> usize;
    fn set_txid(&mut self, txid: [u8; 32]);
    fn set_vout(&mut self, vout: usize);
}

impl TXInputSupport for TXInput {
    fn new(txid: [u8; 32], vout: usize) -> TXInput {
        TXInput { 
            txid, 
            vout,   
        }
    }

    fn get_txid(&self) -> &[u8; 32] {
        &self.txid
    }

    fn get_vout(&self) -> usize {
        self.vout
    }

    fn set_txid(&mut self, txid: [u8; 32]) {
        self.txid = txid;
    }

    fn set_vout(&mut self, vout: usize) {
        self.vout = vout;
    }
}