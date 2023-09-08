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
    queue: Arc<ArrayQueue<C>>,
    permit: OwnedSemaphorePermit,
}

impl<C> RedisPoolConnection<C>
where
    C: redis::aio::ConnectionLike + Send,
{
    pub fn new(con: C, permit: OwnedSemaphorePermit, queue: Arc<ArrayQueue<C>>) -> Self {
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
        // the size of queue is equal to number of semaphore permits, so it shouldn't
        // be possible for push to result in an error
        let _ = self
            .queue
            .push(std::mem::replace(&mut self.con, Option::None).unwrap());
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
