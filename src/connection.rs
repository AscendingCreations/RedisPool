use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use crossbeam_queue::ArrayQueue;
use redis::{aio::ConnectionLike, Cmd, RedisFuture, Value};
use tokio::sync::OwnedSemaphorePermit;

pub struct RedisPoolConnection<C>
where
    C: redis::aio::ConnectionLike + Send,
{
    // This field can be safley unwrapped because it is always initialized to Some
    // and only set to None when dropped
    con: Option<C>,
    permit: Option<OwnedSemaphorePermit>,
    queue: Arc<ArrayQueue<C>>,
}

impl<C> RedisPoolConnection<C>
where
    C: redis::aio::ConnectionLike + Send,
{
    pub fn new(con: C, permit: Option<OwnedSemaphorePermit>, queue: Arc<ArrayQueue<C>>) -> Self {
        RedisPoolConnection {
            con: Some(con),
            permit,
            queue,
        }
    }
}

impl<C> Drop for RedisPoolConnection<C>
where
    C: redis::aio::ConnectionLike + Send,
{
    fn drop(&mut self) {
        let _ = self.queue.push(self.con.take().unwrap());
    }
}

impl<C> Deref for RedisPoolConnection<C>
where
    C: redis::aio::ConnectionLike + Send,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.con.as_ref().unwrap()
    }
}

impl<C> DerefMut for RedisPoolConnection<C>
where
    C: redis::aio::ConnectionLike + Send,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.con.as_mut().unwrap()
    }
}

impl<C> ConnectionLike for RedisPoolConnection<C>
where
    C: redis::aio::ConnectionLike + Send,
{
    fn req_packed_command<'a>(&'a mut self, cmd: &'a Cmd) -> RedisFuture<'a, Value> {
        self.con.as_mut().unwrap().req_packed_command(cmd)
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        self.con
            .as_mut()
            .unwrap()
            .req_packed_commands(cmd, offset, count)
    }

    fn get_db(&self) -> i64 {
        self.con.as_ref().unwrap().get_db()
    }
}
