use async_trait::async_trait;
use redis::{
    aio::{ConnectionLike, MultiplexedConnection},
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
impl ConnectionFactory<MultiplexedConnection> for Client {
    async fn create(&self) -> RedisResult<MultiplexedConnection> {
        self.get_multiplexed_async_connection().await
    }
}
