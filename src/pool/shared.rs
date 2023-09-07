use crate::pool::RedisConnection;
use crate::RedisError;
use crossbeam_queue::ArrayQueue;
pub use redis::{aio::Connection, Client};
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) struct SharedPool {
    pub(crate) pool: Mutex<ArrayQueue<Connection>>,
    pub(crate) client: Client,
}

impl SharedPool {
    pub(crate) fn new_arc(client: Client, limit: u32) -> Arc<Self> {
        let pool = Self {
            pool: Mutex::new(ArrayQueue::new(limit as usize)),
            client,
        };

        Arc::new(pool)
    }

    pub(crate) async fn aquire(self: Arc<Self>) -> Result<RedisConnection, RedisError> {
        {
            let pool = self.pool.lock().await;

            if let Some(connection) = pool.pop() {
                return Ok(RedisConnection::new(Arc::clone(&self), connection));
            }
        }

        let connection = self.client.get_async_connection().await?;

        Ok(RedisConnection::new(Arc::clone(&self), connection))
    }
}
