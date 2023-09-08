use std::sync::Arc;

use async_trait::async_trait;
use crossbeam_queue::ArrayQueue;
use redis::{cluster::ClusterClient, cluster_async::ClusterConnection, RedisResult};
use tokio::sync::Semaphore;

use crate::{
    factory::ConnectionFactory,
    pool::{RedisPool, DEFAULT_CON_LIMIT, DEFAULT_POOL_SIZE},
};

pub type ClusterRedisPool = RedisPool<ClusterClient, ClusterConnection>;

impl ClusterRedisPool {
    pub fn new_cluster(client: ClusterClient, pool_size: usize, con_limit: usize) -> Self {
        RedisPool {
            client,
            queue: Arc::new(ArrayQueue::new(pool_size)),
            sem: Arc::new(Semaphore::new(con_limit)),
        }
    }
}

impl From<ClusterClient> for ClusterRedisPool {
    fn from(value: ClusterClient) -> Self {
        ClusterRedisPool::new_cluster(value, DEFAULT_POOL_SIZE, DEFAULT_CON_LIMIT)
    }
}

#[async_trait]
impl ConnectionFactory<ClusterConnection> for ClusterClient {
    async fn create(&self) -> RedisResult<ClusterConnection> {
        self.get_async_connection().await
    }
}
