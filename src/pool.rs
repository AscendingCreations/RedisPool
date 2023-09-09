use crate::{connection::RedisPoolConnection, errors::RedisPoolError, factory::ConnectionFactory};
use crossbeam_queue::ArrayQueue;
use redis::RedisResult;
use std::{ops::Deref, sync::Arc};
use tokio::sync::Semaphore;

pub const DEFAULT_POOL_SIZE: usize = 16;
pub const DEFAULT_CON_LIMIT: usize = 512;

pub struct RedisPool<F, C>
where
    F: ConnectionFactory<C> + Send + Sync + Clone,
    C: redis::aio::ConnectionLike + Send,
{
    pub(crate) client: F,
    pub(crate) sem: Arc<Semaphore>,
    pub(crate) queue: Arc<ArrayQueue<C>>,
}

impl<F, C> RedisPool<F, C>
where
    F: ConnectionFactory<C> + Send + Sync + Clone,
    C: redis::aio::ConnectionLike + Send,
{
    pub async fn aquire(&self) -> Result<RedisPoolConnection<C>, RedisPoolError> {
        let permit = self.sem.clone().acquire_owned().await?;
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
                    tracing::trace!("connection ping returned wrong value");
                }
                Err(e) => {
                    tracing::trace!("bad redis connection: {}", e);
                }
            }
        }

        self.client.create().await
    }
}

impl<F, C> Clone for RedisPool<F, C>
where
    F: ConnectionFactory<C> + Send + Sync + Clone,
    C: redis::aio::ConnectionLike + Send,
{
    fn clone(&self) -> Self {
        return RedisPool {
            client: self.client.clone(),
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
        &self.client
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
