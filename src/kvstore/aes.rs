use std::fmt::Display;

use aes_gcm_siv::{Aes256GcmSiv, KeyInit, Nonce, aead::Aead};
use heed::byteorder::{ByteOrder, LittleEndian};
use teloxide::types::UserId;

use crate::{
    kvstore::{KVError, KVStore},
    privkey::PrivateKey,
};

pub struct AesEncryptedKV<KS: KVStore> {
    inner: KS,
    cipher: Aes256GcmSiv,
}

impl<KS: KVStore> AesEncryptedKV<KS> {
    pub fn new(source: KS, privkey: &PrivateKey) -> Self {
        Self {
            inner: source,
            cipher: Aes256GcmSiv::new(privkey.as_ref()),
        }
    }

    fn decrpt_for(&self, user_id: UserId, value: &[u8]) -> Vec<u8> {
        let nonce = get_nonce_for_user(user_id);
        self.cipher.decrypt(&nonce, value).unwrap()
    }
}

impl<KS: KVStore> KVStore for AesEncryptedKV<KS> {
    type Err = Error<KS::Err>;

    fn get(&self, user_id: UserId) -> Result<Option<Vec<u8>>, Self::Err> {
        Ok(self
            .inner
            .get(user_id)
            .map(|op| op.map(|key| self.decrpt_for(user_id, &key)))?)
    }

    fn set(&self, user_id: UserId, value: &[u8]) -> Result<(), Self::Err> {
        let nonce = get_nonce_for_user(user_id);
        let encrypted_value: Vec<u8> = self.cipher.encrypt(&nonce, value)?;
        Ok(self.inner.set(user_id, &encrypted_value)?)
    }

    fn remove(&self, user_id: UserId) -> Result<Option<Vec<u8>>, Self::Err> {
        Ok(self
            .inner
            .remove(user_id)?
            .map(|key| self.decrpt_for(user_id, &key)))
    }

    fn contains(&self, user_id: UserId) -> Result<bool, Self::Err> {
        Ok(self.inner.contains(user_id)?)
    }
}

#[derive(Debug)]
pub enum Error<Err: KVError> {
    Crypto(aes_gcm_siv::Error),
    KV(Err),
}

impl<Err: KVError> KVError for Error<Err> {}
impl<Err: KVError> Display for Error<Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Crypto(error) => write!(f, "crypto error: {error}"),
            Error::KV(error) => write!(f, "kv error: {error}"),
        }
    }
}
impl<Err: KVError> core::error::Error for Error<Err> {}

impl<Err: KVError> From<aes_gcm_siv::Error> for Error<Err> {
    fn from(value: aes_gcm_siv::Error) -> Self {
        Self::Crypto(value)
    }
}

impl<Err: KVError> From<Err> for Error<Err> {
    fn from(value: Err) -> Self {
        Self::KV(value)
    }
}

fn get_nonce_for_user(user_id: UserId) -> Nonce {
    let mut uid_buffer = Vec::new();
    uid_buffer.resize(8, 0);
    LittleEndian::write_u64_into(&[user_id.0], &mut uid_buffer);
    uid_buffer.resize(12, 0);
    Nonce::clone_from_slice(&uid_buffer)
}
