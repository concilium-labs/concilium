use std::sync::Arc;
use concilium_core::temporary_node_ids::{TemporaryNodeIds, TemporaryNodeIdsMap, TemporaryNodeIdsMapAddOp, TemporaryNodeIdsRead, TemporaryNodeIdsReadFactory, TemporaryNodeIdsWrite};
use left_right::{ReadHandle, ReadHandleFactory, WriteHandle};
use tokio::sync::Mutex;

pub trait TemporaryNodeIdsWriteSupport {
    fn new(write: WriteHandle<TemporaryNodeIdsMap, TemporaryNodeIdsMapAddOp>) -> TemporaryNodeIdsWrite;
}

impl TemporaryNodeIdsWriteSupport for TemporaryNodeIdsWrite {
    fn new(write: WriteHandle<TemporaryNodeIdsMap, TemporaryNodeIdsMapAddOp>) -> TemporaryNodeIdsWrite {
        Self(write)
    }
}

pub trait TemporaryNodeIdsReadSupport {
    fn new(read: ReadHandle<TemporaryNodeIdsMap>) -> TemporaryNodeIdsRead;
}

impl TemporaryNodeIdsReadSupport for TemporaryNodeIdsRead {
    fn new(read: ReadHandle<TemporaryNodeIdsMap>) -> TemporaryNodeIdsRead {
        Self(read)
    }
}

pub trait TemporaryNodeIdsReadFactorySupport {
    fn new(factory: ReadHandleFactory<TemporaryNodeIdsMap>) -> TemporaryNodeIdsReadFactory;
}

impl TemporaryNodeIdsReadFactorySupport for TemporaryNodeIdsReadFactory {
    fn new(factory: ReadHandleFactory<TemporaryNodeIdsMap>) -> TemporaryNodeIdsReadFactory {
        Self(factory)
    }
}

pub trait TemporaryNodeIdsSupport {
    fn new() -> TemporaryNodeIds;
    fn get_write(&self) -> Arc<Mutex<TemporaryNodeIdsWrite>>;
    fn get_read(&self) -> TemporaryNodeIdsRead;
}

impl TemporaryNodeIdsSupport for TemporaryNodeIds {
    fn new() -> TemporaryNodeIds {
        let (write, read) = left_right::new::<TemporaryNodeIdsMap, TemporaryNodeIdsMapAddOp>();

        TemporaryNodeIds { 
            write: Arc::new(Mutex::new(TemporaryNodeIdsWrite::new(write))),
            read: TemporaryNodeIdsReadFactory::new(read.factory())
        }   
    }

    fn get_write(&self) -> Arc<Mutex<TemporaryNodeIdsWrite>> {
        Arc::clone(&self.write)
    }

    fn get_read(&self) -> TemporaryNodeIdsRead {
        TemporaryNodeIdsRead::new(self.read.0.handle())
    }
}