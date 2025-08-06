use std::sync::Arc;
use ahash::AHashMap;
use left_right::{Absorb, ReadHandle, ReadHandleFactory, WriteHandle};
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Epoch {
    pub id: u64,
    pub last_node_id: u32,
    pub final_hash: [u8; 32],
    pub random_numbers: Vec<u64>,
    pub hashes: AHashMap<[u8; 32], u32>, // hash, count
}

pub enum EpochMapAddOp {
    Insert(u64, Arc<Epoch>),
    Update(u64, Arc<Epoch>),
    Remove(u64),
}

#[derive(Default, Debug)]
pub struct EpochMap(AHashMap<u64, Arc<Epoch>>);

pub struct EpochPoolWrite(pub WriteHandle<EpochMap, EpochMapAddOp>);

pub struct EpochPoolRead(pub ReadHandle<EpochMap>);

pub struct EpochPoolReadFactory(pub ReadHandleFactory<EpochMap>);

pub struct EpochPool {
    pub write: Arc<Mutex<EpochPoolWrite>>,
    pub read: EpochPoolReadFactory,
}

impl Absorb<EpochMapAddOp> for EpochMap {
    fn absorb_first(&mut self, operation: &mut EpochMapAddOp, _: &Self) {
        match operation {
            EpochMapAddOp::Insert(k, v) => {
                self.0.insert(*k, Arc::clone(&v));
            },
            EpochMapAddOp::Update(k, v) => {
                self.0.insert(*k, Arc::clone(&v));
            },
            EpochMapAddOp::Remove(k) => {
                self.0.remove(k);
            }
        }
    }

    fn absorb_second(&mut self, operation: EpochMapAddOp, _: &Self) {
        match operation {
            EpochMapAddOp::Insert(k, v) => {
                self.0.insert(k, Arc::clone(&v));
            },
            EpochMapAddOp::Update(k, v) => {
                self.0.insert(k, Arc::clone(&v));
            },
            EpochMapAddOp::Remove(k) => {
                self.0.remove(&k);
            }
        }
    }

    fn drop_first(self: Box<Self>) {}

    fn drop_second(self: Box<Self>) {}

    fn sync_with(&mut self, first: &Self) {
        self.0 = first.0.clone();
    }
}

impl EpochPoolWrite {
    pub fn insert(&mut self, id: u64, epoch: Arc<Epoch>) {
        self.0.append(EpochMapAddOp::Insert(id, epoch));
    }
    
    pub fn update(&mut self, id: u64, epoch: Arc<Epoch>) {
        self.0.append(EpochMapAddOp::Update(id, epoch));
    }
    
    pub fn remove(&mut self, id: u64) {
        self.0.append(EpochMapAddOp::Remove(id));
    }
    
    pub fn publish(&mut self) {
        self.0.publish();
    }
}

impl EpochPoolRead {
    pub fn get(&self, id: &u64) -> Option<Arc<Epoch>> {
        self.0.enter().map(|guard| {
            guard.0.get(id).cloned()
        }).flatten()
    }

    pub fn get_keys(&self) -> Option<Vec<u64>> {
        self.0.enter().map(|guard| {
            guard.0.keys().map(|id| id.clone()).collect::<Vec<u64>>()
        })
    }

    pub fn len(&self) -> usize {
        self.0.enter().map(|guard| guard.0.len()).unwrap_or(0)
    }
}