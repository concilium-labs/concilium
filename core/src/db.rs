use rocksdb::DB as RocksDB;

pub struct DB {
    pub db: RocksDB,
}