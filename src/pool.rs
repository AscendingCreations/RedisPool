use crate::{connection::RedisPoolConnection, errors::RedisPoolError, factory::ConnectionFactory};
use crossbeam_queue::ArrayQueue;
use redis::{aio::Connection, Client, RedisResult};
use std::{ops::Deref, sync::Arc};
use tokio::sync::Semaphore;

pub const DEFAULT_POOL_SIZE: usize = 16;
pub const DEFAULT_CON_LIMIT: usize = 512;

pub struct RedisPool<F, C>
where
    F: ConnectionFactory<C> + Send + Sync + Clone,
    C: redis::aio::ConnectionLike + Send,
{
    factory: F,
    queue: Arc<ArrayQueue<C>>,
    sem: Option<Arc<Semaphore>>,
}

impl<F, C> RedisPool<F, C>
where
    F: ConnectionFactory<C> + Send + Sync + Clone,
    C: redis::aio::ConnectionLike + Send,
{
    pub fn new(factory: F, pool_size: usize, con_limit: Option<usize>) -> Self {
        if pool_size > con_limit.unwrap_or(usize::MAX) {
            tracing::warn!("pool size is greater then connection limit");
        }

        return RedisPool {
            factory,
            queue: Arc::new(ArrayQueue::new(pool_size)),
            sem: con_limit.map(|lim| Arc::new(Semaphore::new(lim))),
        };
    }

    pub async fn aquire(&self) -> Result<RedisPoolConnection<C>, RedisPoolError> {
        let permit = match &self.sem {
            Some(sem) => Some(sem.clone().acquire_owned().await?),
            None => None,
        };
        let con = self.aquire_connection().await?;
        Ok(RedisPoolConnection::new(con, permit, self.queue.clone()))
    }

    async fn aquire_connection(&self) -> RedisResult<C> {
        while let Some(mut con) = self.queue.pop() {
            let res = redis::Pipeline::with_capacity(2)
                .cmd("UNWATCH")
                .ignore()
                .cmd("PING")
                .arg(1)
                .query_async::<_, (usize,)>(&mut con)
                .await;

            match res {
                Ok((1,)) => {
                    return Ok(con);
                }
                Ok(_) => {
                    tracing::warn!("connection ping returned wrong value");
                }
                Err(e) => {
                    tracing::warn!("bad redis connection: {}", e);
                }
            }
        }

        self.factory.create().await
    }

    pub fn factory(&self) -> &F {
        &self.factory
    }
}

impl<F, C> Clone for RedisPool<F, C>
where
    F: ConnectionFactory<C> + Send + Sync + Clone,
    C: redis::aio::ConnectionLike + Send,
{
    fn clone(&self) -> Self {
        return RedisPool {
            factory: self.factory.clone(),
            queue: self.queue.clone(),
            sem: self.sem.clone(),
        };
    }
}

impl<F, C> Deref for RedisPool<F, C>
where
    F: ConnectionFactory<C> + Send + Sync + Clone,
    C: redis::aio::ConnectionLike + Send,
{
    type Target = F;

    fn deref(&self) -> &Self::Target {
        &self.factory
    }
}

pub type SingleRedisPool = RedisPool<Client, Connection>;

impl From<Client> for SingleRedisPool {
    fn from(value: Client) -> Self {
        RedisPool::new(value, DEFAULT_POOL_SIZE, Some(DEFAULT_CON_LIMIT))
    }
}

// compile time assert pool thread saftey
const _: () = {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    fn assert_all<F, C>()
    where
        F: ConnectionFactory<C> + Send + Sync + Clone,
        C: redis::aio::ConnectionLike + Send,
    {
        assert_send::<RedisPool<F, C>>();
        assert_sync::<RedisPool<F, C>>();
    }
};
