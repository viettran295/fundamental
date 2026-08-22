use std::collections::HashMap;

use async_trait::async_trait;
use log::{error, warn};
use redis::{aio::MultiplexedConnection, AsyncCommands, RedisResult};

use crate::db::{DataManager, DataManagerError};

pub struct DragonFlyCache {
    connection_: MultiplexedConnection,
    timeout_seconds: i64,
}

#[async_trait]
impl DataManager<String, HashMap<String, f64>> for DragonFlyCache {
    async fn init(uri: &str, timeout_seconds: i64) -> Result<Self, DataManagerError> {
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
                error!("Error connecting to Dragonfly Db at {}: {}", uri, e);
                DataManagerError::ConnectionFailed
            })?;
        Ok(Self {
            connection_: connection,
            timeout_seconds: timeout_seconds,
        })
    }

    async fn set(&mut self, key: String, value: HashMap<String, f64>) {
        let fields: Vec<(String, f64)> = value.into_iter().collect();
        let result: RedisResult<()> =
            AsyncCommands::hset_multiple(&mut self.connection_, &key, &fields).await;
        match result {
            Ok(_) => {
                if let Err(e) = AsyncCommands::expire::<String, bool>(
                    &mut self.connection_,
                    key.clone(),
                    self.timeout_seconds,
                )
                .await
                {
                    warn!("Error setting timeout for {:?}: {}", key, e);
                }
            }
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

    async fn is_empty(&mut self) -> Result<bool, DataManagerError> {
        let db_size: usize = match redis::cmd("DBSIZE")
            .query_async(&mut self.connection_)
            .await
        {
            Err(_) => {
                return Err(DataManagerError::General);
            }
            Ok(size) => size,
        };
        // If size is 0, the DB is empty
        Ok(db_size == 0)
    }
}
