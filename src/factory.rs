use async_trait::async_trait;
use redis::{
    aio::{Connection, ConnectionLike},
    Client, RedisResult,
};

#[async_trait]
pub trait ConnectionFactory<C>
where
    C: ConnectionLike,
{
    async fn create(&self) -> RedisResult<C>;
}

#[async_trait]
impl ConnectionFactory<Connection> for Client {
    async fn create(&self) -> RedisResult<Connection> {
        self.get_async_connection().await
    }
}

// #[cfg(feature = "cluster")]
use redis::{cluster::ClusterClient, cluster_async::ClusterConnection};

// #[cfg(feature = "cluster")]
#[async_trait]
impl ConnectionFactory<ClusterConnection> for ClusterClient {
    async fn create(&self) -> RedisResult<ClusterConnection> {
        self.get_async_connection().await
    }
}
