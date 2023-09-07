use crate::pool::RedisClusterConnection;
use crate::RedisError;
use crossbeam_queue::ArrayQueue;
pub use redis::{cluster::ClusterClient, cluster_async::ClusterConnection};
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) struct SharedClusterPool {
    pub(crate) pool: Mutex<ArrayQueue<ClusterConnection>>,
    pub(crate) client: ClusterClient,
}

impl SharedClusterPool {
    pub(crate) fn new_arc(client: ClusterClient, limit: u32) -> Arc<Self> {
        let pool = Self {
            pool: Mutex::new(ArrayQueue::new(limit as usize)),
            client,
        };

        Arc::new(pool)
    }

    pub(crate) async fn aquire(self: Arc<Self>) -> Result<RedisClusterConnection, RedisError> {
        {
            let pool = self.pool.lock().await;

            if let Some(connection) = pool.pop() {
                return Ok(RedisClusterConnection::new(Arc::clone(&self), connection));
            }
        }

        let connection = self.client.get_async_connection().await?;

        Ok(RedisClusterConnection::new(Arc::clone(&self), connection))
    }
}
