use std::path::Path;

use heed::{
    Database, Env, EnvOpenOptions, RwTxn,
    byteorder::LittleEndian,
    types::{Bytes, U64},
};
use teloxide::types::UserId;

use crate::kvstore::{KVError, KVStore};

pub struct HeedKV {
    env: Env,
    name: String,
}

impl HeedKV {
    pub fn new<P: AsRef<Path>, S: AsRef<str>>(file: P, name: S) -> Result<Self, heed::Error> {
        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(5 << 20)
                .max_dbs(8)
                .open(file)?
        }; // 5 MiB
        Ok(Self {
            env,
            name: name.as_ref().to_string(),
        })
    }

    fn create_database(&self) -> Result<(Database<U64<LittleEndian>, Bytes>, RwTxn), heed::Error> {
        let mut wtxn = self.env.write_txn()?;
        let db: Database<U64<LittleEndian>, Bytes> =
            self.env.create_database(&mut wtxn, Some(&self.name))?;
        Ok((db, wtxn))
    }
}

impl KVStore for HeedKV {
    type Err = heed::Error;

    fn get(&self, user_id: UserId) -> Result<Option<Vec<u8>>, Self::Err> {
        let (db, rwtxn) = self.create_database()?;
        let option = db
            .get(&rwtxn, &user_id.0)
            .map(|opt| opt.map(|data| Vec::from(data)))?;
        Ok(option)
    }

    fn set(&self, user_id: UserId, value: &[u8]) -> Result<(), Self::Err> {
        let (db, mut wtxn) = self.create_database()?;
        db.put(&mut wtxn, &user_id.0, value)?;
        wtxn.commit()?;
        Ok(())
    }

    fn remove(&self, user_id: UserId) -> Result<Option<Vec<u8>>, Self::Err> {
        let (db, mut wtxn) = self.create_database()?;
        if let Some(key) = self.get(user_id)? {
            db.delete(&mut wtxn, &user_id.0)?;
            wtxn.commit()?;
            return Ok(Some(key));
        }
        return Ok(None);
    }
}

impl KVError for heed::Error {}
