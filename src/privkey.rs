use aes_gcm_siv::{Aes256GcmSiv, Key, KeyInit, aead::OsRng};
use base64::{Engine, prelude::BASE64_STANDARD};

pub struct PrivateKey(Key<Aes256GcmSiv>);

impl PrivateKey {
    pub fn new() -> Self {
        let random = Aes256GcmSiv::generate_key(&mut OsRng);
        Self(random)
    }

    pub fn from_string<S>(string: S) -> Result<Self, base64::DecodeError>
    where
        S: AsRef<str>,
    {
        let raw: Vec<u8> = BASE64_STANDARD.decode(string.as_ref().as_bytes())?;
        Ok(Self(Key::<Aes256GcmSiv>::clone_from_slice(&raw)))
    }
}

impl ToString for PrivateKey {
    fn to_string(&self) -> String {
        BASE64_STANDARD.encode(self.0)
    }
}

impl Into<Key<Aes256GcmSiv>> for PrivateKey {
    fn into(self) -> Key<Aes256GcmSiv> {
        self.0
    }
}

impl AsRef<Key<Aes256GcmSiv>> for PrivateKey {
    fn as_ref(&self) -> &Key<Aes256GcmSiv> {
        &self.0
    }
}
