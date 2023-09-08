use std::sync::Arc;

use crossbeam_queue::ArrayQueue;
use redis::{cluster::ClusterClient, cluster_async::ClusterConnection};
use tokio::sync::Semaphore;

use crate::pool::{RedisPool, DEFAULT_POOL_LIMIT};

impl RedisPool<ClusterClient, ClusterConnection> {
    pub fn from_cluster_client(client: ClusterClient, limit: usize) -> Self {
        RedisPool {
            client,
            queue: Arc::new(ArrayQueue::new(limit)),
            sem: Arc::new(Semaphore::new(limit)),
        }
    }
}

impl From<ClusterClient> for RedisPool<ClusterClient, ClusterConnection> {
    fn from(value: ClusterClient) -> Self {
        RedisPool::from_cluster_client(value, DEFAULT_POOL_LIMIT)
    }
}
