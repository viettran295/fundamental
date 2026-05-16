use std::collections::HashMap;

use async_trait::async_trait;
use log::{error, warn};
use redis::{aio::MultiplexedConnection, AsyncCommands, RedisResult};

use crate::db::{DataManager, DataManagerError};

pub struct DragonFlyCache {
    connection_: MultiplexedConnection,
}

#[async_trait]
impl DataManager<String, HashMap<String, f64>> for DragonFlyCache {
    async fn init(uri: String) -> Result<Self, DataManagerError> {
        let client = match redis::Client::open(uri) {
            Ok(client) => client,
            Err(_) => {
                error!("Error openning to Dragonfly Db");
                return Err(DataManagerError::ConnectionFailed);
            }
        };
        let connection = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                error!("Error connecting to Dragonfly Db: {}", e);
                DataManagerError::ConnectionFailed
            })
            .unwrap();
        Ok(Self {
            connection_: connection,
        })
    }

    async fn set(&mut self, key: String, value: HashMap<String, f64>) {
        let fields: Vec<(String, f64)> = value.into_iter().collect();
        let result: RedisResult<()> =
            AsyncCommands::hset_multiple(&mut self.connection_, &key, &fields).await;
        match result {
            Ok(_) => {}
            Err(e) => {
                warn!("Error setting key: {:?} in Dragonfly Db - {}", key, e);
            }
        };
    }

    async fn get(&mut self, key: String) -> Result<HashMap<String, f64>, DataManagerError> {
        AsyncCommands::hgetall(&mut self.connection_, &key)
            .await
            .map_err(|_| DataManagerError::NotFound)
    }
}
