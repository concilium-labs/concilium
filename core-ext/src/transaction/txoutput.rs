use concilium_core::transaction::TXOutput;

pub trait TXOutputSupport {
    fn new(value: f32, public_key: [u8; 32]) -> TXOutput;
    fn get_value(&self) -> f32;
    fn get_public_key(&self) -> &[u8; 32];
    fn set_value(&mut self, value: f32);
    fn set_public_key(&mut self, public_key: [u8; 32]);
}

impl TXOutputSupport for TXOutput {
    fn new(value: f32, public_key: [u8; 32]) -> TXOutput {
        TXOutput { 
            value, 
            public_key
        }
    }

    fn get_value(&self) -> f32 {
        self.value
    }

    fn get_public_key(&self) -> &[u8; 32] {
        &self.public_key
    }

    fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    fn set_public_key(&mut self, public_key: [u8; 32]) {
        self.public_key = public_key;
    }
}