use serde::{Serialize, de::DeserializeOwned};
use teloxide::types::UserId;

use crate::kvstore::KVStore;

pub struct SerdeKV<Inner>
where
    Inner: KVStore,
{
    inner: Inner,
}

impl<Inner, Err> SerdeKV<Inner>
where
    Inner: KVStore<Err = Err>,
{
    pub fn new(kv: Inner) -> Self {
        Self { inner: kv }
    }

    pub fn set<Value>(&self, key: UserId, value: &Value) -> Result<(), Err>
    where
        Value: Serialize,
    {
        self.inner.set(
            key,
            &postcard::to_allocvec(value).expect("Serialization failed"),
        )
    }

    pub fn get<'a, Value>(&self, key: UserId) -> Result<Option<Value>, Err>
    where
        Value: DeserializeOwned,
    {
        let Some(data) = self.inner.get(key)? else {
            return Ok(None);
        };
        let value = postcard::from_bytes::<'_, Value>(&data).expect("Deserialization failed");
        Ok(Some(value))
    }

    pub fn remove<Value>(&self, key: UserId) -> Result<Option<Value>, Err>
    where
        Value: DeserializeOwned,
    {
        let Some(data) = self.inner.remove(key)? else {
            return Ok(None);
        };
        let value = postcard::from_bytes::<'_, Value>(&data).expect("Deserialization failed");
        Ok(Some(value))
    }

    pub fn contains(&self, key: UserId) -> Result<bool, Err> {
        self.inner.contains(key)
    }
}
