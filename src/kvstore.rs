use std::error::Error;

use teloxide::types::UserId;

pub trait KVStore {
    type Err: KVError;
    /// Get the post auth key for a user, or None if unset.
    fn get(&self, user_id: UserId) -> Result<Option<Vec<u8>>, Self::Err>;
    /// Update the post auth key for a user.
    fn set(&self, user_id: UserId, value: &[u8]) -> Result<(), Self::Err>;
    /// Remove the post auth key for a user, returning the original key if removed, or None if
    /// unset.
    fn remove(&self, user_id: UserId) -> Result<Option<Vec<u8>>, Self::Err>;
    /// Whether the post auth key for a user is set.
    fn contains(&self, user_id: UserId) -> Result<bool, Self::Err> {
        if let Some(_) = self.get(user_id)? {
            return Ok(true);
        }
        return Ok(false);
    }
}

pub trait KVError: Error {}

pub mod aes;
pub mod heed;
pub mod structured;
