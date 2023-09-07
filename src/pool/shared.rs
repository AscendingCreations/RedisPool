use crate::pool::RedisConnection;
use crate::RedisError;
use crossbeam_queue::ArrayQueue;
pub use redis::{aio::Connection, Client};
use std::sync::Arc;

pub(crate) struct SharedPool {
    pub(crate) pool: ArrayQueue<Connection>,
    pub(crate) client: Client,
}

impl SharedPool {
    pub(crate) fn new_arc(client: Client, limit: u32) -> Arc<Self> {
        let pool = Self {
            pool: ArrayQueue::new(limit as usize),
            client,
        };

        Arc::new(pool)
    }

    pub(crate) async fn aquire(self: Arc<Self>) -> Result<RedisConnection, RedisError> {
        {
            if let Some(connection) = self.pool.pop() {
                return Ok(RedisConnection::new(Arc::clone(&self), connection));
            }
        }

        let connection = self.client.get_async_connection().await?;

        Ok(RedisConnection::new(Arc::clone(&self), connection))
    }
}
