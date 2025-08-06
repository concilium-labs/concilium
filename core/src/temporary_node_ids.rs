use std::sync::Arc;
use ahash::AHashMap;
use left_right::{Absorb, ReadHandle, ReadHandleFactory, WriteHandle};
use tokio::sync::Mutex;

pub enum TemporaryNodeIdsMapAddOp {
    Insert(u64, Arc<AHashMap<u32, u32>>),
    Remove(u64),
}

#[derive(Default, Debug)]
pub struct TemporaryNodeIdsMap(AHashMap<u64, Arc<AHashMap<u32, u32>>>);

pub struct TemporaryNodeIdsWrite(pub WriteHandle<TemporaryNodeIdsMap, TemporaryNodeIdsMapAddOp>);

pub struct TemporaryNodeIdsRead(pub ReadHandle<TemporaryNodeIdsMap>);

pub struct TemporaryNodeIdsReadFactory(pub ReadHandleFactory<TemporaryNodeIdsMap>);

pub struct TemporaryNodeIds {
    pub write: Arc<Mutex<TemporaryNodeIdsWrite>>,
    pub read: TemporaryNodeIdsReadFactory,
}

impl Absorb<TemporaryNodeIdsMapAddOp> for TemporaryNodeIdsMap {
    fn absorb_first(&mut self, operation: &mut TemporaryNodeIdsMapAddOp, _: &Self) {
        match operation {
            TemporaryNodeIdsMapAddOp::Insert(k, v) => {
                self.0.insert(*k, Arc::clone(&v));
            },
            TemporaryNodeIdsMapAddOp::Remove(k) => {
                self.0.remove(k);
            }
        }
    }

    fn absorb_second(&mut self, operation: TemporaryNodeIdsMapAddOp, _: &Self) {
        match operation {
            TemporaryNodeIdsMapAddOp::Insert(k, v) => {
                self.0.insert(k, Arc::clone(&v));
            },
            TemporaryNodeIdsMapAddOp::Remove(k) => {
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

impl TemporaryNodeIdsWrite {
    pub fn insert(&mut self, epoch_id: u64, node_ids_mapping: Arc<AHashMap<u32, u32>>) {
        self.0.append(TemporaryNodeIdsMapAddOp::Insert(epoch_id, node_ids_mapping));
    }
    
    pub fn remove(&mut self, epoch_id: u64) {
        self.0.append(TemporaryNodeIdsMapAddOp::Remove(epoch_id));
    }
    
    pub fn publish(&mut self) {
        self.0.publish();
    }
}

impl TemporaryNodeIdsRead {
    pub fn get(&self, epoch_id: &u64) -> Option<Arc<AHashMap<u32, u32>>> {
        self.0.enter().map(|guard| {
            guard.0.get(epoch_id).cloned()
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