use concilium_core::transaction::SenderTransactionTemp;

pub trait SenderTransactionTempSupport {
    fn new(public_key: [u8; 32], epoch_number: i64) -> SenderTransactionTemp;
}

impl SenderTransactionTempSupport for SenderTransactionTemp {
    fn new(public_key: [u8; 32], epoch_number: i64) -> SenderTransactionTemp {
        Self {
            public_key,
            epoch_number
        }
    }
}