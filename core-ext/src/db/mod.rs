use rocksdb::DBWithThreadMode;
use rocksdb::SingleThreaded;
use rocksdb::DB as RocksDB;
use concilium_error::Error as Error;
use concilium_core::db::DB;

pub trait DBSupport {
    fn new() -> Result<DB, Error>;
    fn put<T>(&self, key: &str, value: &T) -> Result<(), Error> where T: AsRef<[u8]>;
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, Error>;
    fn delete<T: AsRef<[u8]>>(&self, key: T) -> Result<(), Error>;
    fn exist<T: AsRef<[u8]>>(&self, key: T) -> bool;
    fn get_db(&self) -> &DBWithThreadMode<SingleThreaded>; 
}

impl DBSupport for DB {
    fn new() -> Result<DB, Error> {
        Ok(
            Self {
                db: RocksDB::open_default(".concilium/db")?,
            }
        )
    }

    fn put<T>(&self, key: &str, value: &T) -> Result<(), Error>
    where
        T: AsRef<[u8]>,
    {
        Ok(self.db.put(key, value)?)
    }

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, Error> {
        if let Some(result) = self.db.get(key)? {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    fn delete<T: AsRef<[u8]>>(&self, key: T) -> Result<(), Error> {
        self.db.delete(key)?;
        Ok(())
    }

    fn exist<T: AsRef<[u8]>>(&self, key: T) -> bool {
        self.db.key_may_exist(key)
    }

    fn get_db(&self) -> &DBWithThreadMode<SingleThreaded> {
        &self.db
    }
}