use async_trait::async_trait;
use log::{error, warn};
use redis::{AsyncTypedCommands, aio::MultiplexedConnection};

use crate::data::{DataManager, DataManagerError};

pub struct DragonFlyCache {
    client_: redis::Client,
    connection_: MultiplexedConnection,
}

#[async_trait]
impl DataManager<String, String> for DragonFlyCache {
    async fn init(uri: String) -> Result<Self, DataManagerError> {
        let client = match redis::Client::open(uri) {
            Ok(client) => client,
            Err(_) => {
                error!("Error openning to Dragonfly Db");
                return Err(DataManagerError::ConnectionFailedError);
            }
        };
        let connection = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| {
                error!("Error connecting to Dragonfly Db");
                DataManagerError::ConnectionFailedError
            })
            .unwrap();
        Ok(Self {
            client_: client,
            connection_: connection,
        })
    }
    async fn set(&mut self, key: String, value: String) {
        match self.connection_.set(&key, value).await {
            Ok(_) => {}
            Err(e) => {
                warn!("Error setting key: {:?} in Dragonfly Db - {}", key, e);
            }
        };
    }
    async fn get(&mut self, key: String) -> Result<String, DataManagerError> {
        let stored_value = match self.connection_.get(&key).await {
            Ok(value) => value,
            Err(e) => {
                warn!("Not found value of key: {:?} - {}", key, e);
                return Err(DataManagerError::NotFoundError);
            }
        };
        Ok(stored_value.unwrap_or_default())
    }
}
