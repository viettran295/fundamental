pub mod dragonfly_cache;

use async_trait::async_trait;

#[derive(Debug)]
pub enum DataManagerError {
    General(String),
    NotFound,
    ConnectionFailed,
    SerDerialize,
}

#[async_trait]
pub trait DataManager<K, V> {
    async fn init(uri: String) -> Result<Self, DataManagerError>
    where
        Self: Sized;
    async fn set(&mut self, key: K, value: V);
    async fn get(&mut self, key: K) -> Result<V, DataManagerError>;
    async fn is_empty(&mut self) -> Result<bool, DataManagerError>;
}
