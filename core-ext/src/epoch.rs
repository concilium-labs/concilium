use std::sync::Arc;
use ahash::AHashMap;
use concilium_core::epoch::{Epoch, EpochMap, EpochMapAddOp, EpochPool, EpochPoolRead, EpochPoolReadFactory, EpochPoolWrite};
use left_right::{ReadHandle, ReadHandleFactory, WriteHandle};
use tokio::sync::Mutex;

pub trait EpochSupport {
    fn new(id: u64, last_node_id: u32, final_hash: [u8; 32], random_numbers: Vec<u64>, hashes: AHashMap<[u8; 32], u32>) -> Epoch;
    fn get_id(&self) -> u64;
    fn get_last_node_id(&self) -> u32;
    fn get_final_hash(&self) -> &[u8; 32];
    fn get_random_numbers(&self) -> &[u64];
    fn get_hashes(&self) -> &AHashMap<[u8; 32], u32>;
    fn get_mut_hashes(&mut self) -> &mut AHashMap<[u8; 32], u32>;
    fn set_id(&mut self, id: u64);
    fn set_last_node_id(&mut self, last_node_id: u32);
    fn set_final_hash(&mut self, final_hash: [u8; 32]);
    fn set_random_numbers(&mut self, random_numbers: Vec<u64>);
    fn set_hashes(&mut self, hashes: AHashMap<[u8; 32], u32>);
    fn insert_random_number(&mut self, number: u64);
    fn insert_hash(&mut self, hash: [u8; 32], count: u32);
    fn increase_hash(&mut self, hash: [u8; 32]);
    fn sort_random_numbers(&mut self);
}

impl EpochSupport for Epoch {
    fn new(id: u64, last_node_id: u32, final_hash: [u8; 32], random_numbers: Vec<u64>, hashes: AHashMap<[u8; 32], u32>) -> Epoch {
        Epoch { 
            id, 
            last_node_id, 
            final_hash, 
            random_numbers, 
            hashes 
        }
    }
    
    fn get_id(&self) -> u64 {
        self.id
    }

    fn get_last_node_id(&self) -> u32 {
        self.last_node_id
    }

    fn get_final_hash(&self) -> &[u8; 32] {
        &self.final_hash
    }

    fn get_random_numbers(&self) -> &[u64] {
        &self.random_numbers
    }

    fn get_hashes(&self) -> &AHashMap<[u8; 32], u32> {
        &self.hashes
    }
    
    fn get_mut_hashes(&mut self) -> &mut AHashMap<[u8; 32], u32> {
        &mut self.hashes
    }

    fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    fn set_last_node_id(&mut self, last_node_id: u32) {
        self.last_node_id = last_node_id;
    }

    fn set_final_hash(&mut self, final_hash: [u8; 32]) {
        self.final_hash = final_hash;
    }

    fn set_random_numbers(&mut self, random_numbers: Vec<u64>) {
        self.random_numbers = random_numbers;
    }

    fn set_hashes(&mut self, hashes: AHashMap<[u8; 32], u32>) {
        self.hashes = hashes;
    }

    fn increase_hash(&mut self, hash: [u8; 32]) {
        if let Some(data) = self.hashes.get_mut(&hash) {
            *data += 1;
        }
    }

    fn insert_random_number(&mut self, number: u64) {
        self.random_numbers.push(number);
    }

    fn insert_hash(&mut self, hash: [u8; 32], count: u32) {
        self.hashes.insert(hash, count);
    }

    fn sort_random_numbers(&mut self) {
        self.random_numbers.sort();
    }
}

pub trait EpochPoolWriteSupport {
    fn new(write: WriteHandle<EpochMap, EpochMapAddOp>) -> EpochPoolWrite;
}

impl EpochPoolWriteSupport for EpochPoolWrite {
    fn new(write: WriteHandle<EpochMap, EpochMapAddOp>) -> EpochPoolWrite {
        Self(write)
    }
}

pub trait EpochPoolReadSupport {
    fn new(read: ReadHandle<EpochMap>) -> EpochPoolRead;
}

impl EpochPoolReadSupport for EpochPoolRead {
    fn new(read: ReadHandle<EpochMap>) -> EpochPoolRead {
        Self(read)
    }
}

pub trait EpochPoolReadFactorySupport {
    fn new(factory: ReadHandleFactory<EpochMap>) -> EpochPoolReadFactory;
}

impl EpochPoolReadFactorySupport for EpochPoolReadFactory {
    fn new(factory: ReadHandleFactory<EpochMap>) -> EpochPoolReadFactory {
        Self(factory)
    }
}

pub trait EpochPoolSupport {
    fn new() -> EpochPool;
    fn get_write(&self) -> Arc<Mutex<EpochPoolWrite>>;
    fn get_read(&self) -> EpochPoolRead;
}

impl EpochPoolSupport for EpochPool {
    fn new() -> EpochPool {
        let (write, read) = left_right::new::<EpochMap, EpochMapAddOp>();

        EpochPool { 
            write: Arc::new(Mutex::new(EpochPoolWrite::new(write))),
            read: EpochPoolReadFactory::new(read.factory())
        }   
    }

    fn get_write(&self) -> Arc<Mutex<EpochPoolWrite>> {
        Arc::clone(&self.write)
    }

    fn get_read(&self) -> EpochPoolRead {
        EpochPoolRead::new(self.read.0.handle())
    }
}