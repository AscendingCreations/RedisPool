use async_trait::async_trait;
use redis::{cluster::ClusterClient, cluster_async::ClusterConnection, RedisResult};

use crate::{
    factory::ConnectionFactory,
    pool::{RedisPool, DEFAULT_CON_LIMIT, DEFAULT_POOL_SIZE},
};

pub type ClusterRedisPool = RedisPool<ClusterClient, ClusterConnection>;

impl From<ClusterClient> for ClusterRedisPool {
    fn from(value: ClusterClient) -> Self {
        RedisPool::new(value, DEFAULT_POOL_SIZE, Some(DEFAULT_CON_LIMIT))
    }
}

#[async_trait]
impl ConnectionFactory<ClusterConnection> for ClusterClient {
    async fn create(&self) -> RedisResult<ClusterConnection> {
        self.get_async_connection().await
    }
}
