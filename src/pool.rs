use crate::{connection::RedisPoolConnection, errors::RedisPoolError, factory::ConnectionFactory};
use crossbeam_queue::ArrayQueue;
use redis::{aio::Connection, Client, RedisResult};
use std::sync::Arc;
use tokio::sync::Semaphore;

const DEFAULT_POOL_LIMIT: usize = 16;

#[derive(Clone)]
pub struct RedisPool<F, C>
where
    F: ConnectionFactory<C>,
    C: redis::aio::ConnectionLike + Send + Sync,
{
    pub pool: Arc<RedisPoolShared<F, C>>,
}

pub struct RedisPoolShared<F, C>
where
    F: ConnectionFactory<C>,
    C: redis::aio::ConnectionLike + Send + Sync,
{
    pub client: F,
    queue: Arc<ArrayQueue<C>>,
    sem: Arc<Semaphore>,
}

impl<F, C> RedisPoolShared<F, C>
where
    F: ConnectionFactory<C>,
    C: redis::aio::ConnectionLike + Send + Sync,
{
    pub async fn aquire(&mut self) -> Result<RedisPoolConnection<C>, RedisPoolError> {
        let permit = self.sem.clone().acquire_owned().await?;
        let con = self.aquire_connection().await?;
        let queue = Arc::downgrade(&self.queue);
        Ok(RedisPoolConnection::new(con, queue, permit))
    }

    async fn aquire_connection(&mut self) -> RedisResult<C> {
        if let Some(mut con) = self.queue.as_ref().pop() {
            let (n,) = redis::Pipeline::with_capacity(2)
                .cmd("UNWATCH")
                .ignore()
                .cmd("PING")
                .arg(1)
                .query_async::<_, (usize,)>(&mut con)
                .await?;
            if n == 1 {
                return Ok(con);
            }
        }

        self.client.create().await
    }
}

impl RedisPoolShared<Client, Connection> {
    pub fn from_client(client: Client, limit: usize) -> Self {
        RedisPoolShared {
            client,
            queue: Arc::new(ArrayQueue::new(limit)),
            sem: Arc::new(Semaphore::new(limit)),
        }
    }
}

impl From<Client> for RedisPoolShared<Client, Connection> {
    fn from(value: Client) -> Self {
        RedisPoolShared::from_client(value, DEFAULT_POOL_LIMIT)
    }
}

#[cfg(feature = "cluster")]
use redis::{cluster::ClusterClient, cluster_async::ClusterConnection};

#[cfg(feature = "cluster")]
impl RedisPoolShared<ClusterClient, ClusterConnection> {
    pub fn from_cluster_client(client: ClusterClient, limit: usize) -> Self {
        RedisPoolShared {
            client,
            queue: ArrayQueue::new(limit),
            sem: Semaphore::new(limit),
        }
    }
}

#[cfg(feature = "cluster")]
impl From<ClusterClient> for RedisPoolShared<ClusterClient, ClusterConnection> {
    fn from(value: ClusterClient) -> Self {
        RedisPoolShared::from_cluster_client(value, DEFAULT_POOL_LIMIT)
    }
}
