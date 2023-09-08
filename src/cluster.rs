use std::sync::Arc;

use async_trait::async_trait;
use crossbeam_queue::ArrayQueue;
use redis::{cluster::ClusterClient, cluster_async::ClusterConnection, RedisResult};
use tokio::sync::Semaphore;

use crate::{
    factory::ConnectionFactory,
    pool::{RedisPool, DEFAULT_POOL_LIMIT},
};

pub type ClusterRedisPool = RedisPool<ClusterClient, ClusterConnection>;

impl ClusterRedisPool {
    pub fn new(client: ClusterClient, limit: usize) -> Self {
        RedisPool {
            client,
            queue: Arc::new(ArrayQueue::new(limit)),
            sem: Arc::new(Semaphore::new(limit)),
        }
    }
}

impl From<ClusterClient> for ClusterRedisPool {
    fn from(value: ClusterClient) -> Self {
        ClusterRedisPool::new(value, DEFAULT_POOL_LIMIT)
    }
}

#[async_trait]
impl ConnectionFactory<ClusterConnection> for ClusterClient {
    async fn create(&self) -> RedisResult<ClusterConnection> {
        self.get_async_connection().await
    }
}
