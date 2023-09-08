use std::ops::{Deref, DerefMut};
use std::sync::Weak;

use crossbeam_queue::ArrayQueue;
use redis::{aio::ConnectionLike, Cmd, RedisFuture, Value};
use tokio::sync::OwnedSemaphorePermit;

pub struct RedisPoolConnection<C>
where
    C: redis::aio::ConnectionLike + Send,
{
    // this field can be safley unwrapped because it will always be Some until it is dropped
    con: Option<C>,
    queue: Weak<ArrayQueue<C>>,

    // permit is only used as drop will allow more connections to be taken from queue
    #[allow(dead_code)]
    permit: OwnedSemaphorePermit,
}

impl<C> RedisPoolConnection<C>
where
    C: redis::aio::ConnectionLike + Send,
{
    pub fn new(con: C, queue: Weak<ArrayQueue<C>>, permit: OwnedSemaphorePermit) -> Self {
        RedisPoolConnection {
            con: Some(con),
            queue,
            permit,
        }
    }
}

impl<C> Drop for RedisPoolConnection<C>
where
    C: redis::aio::ConnectionLike + Send,
{
    fn drop(&mut self) {
        if let Some(queue) = self.queue.upgrade() {
            // size of queue is equal to number of semaphore permits it should
            // never be possible to push over the size of the queue
            let _ = queue.push(std::mem::replace(&mut self.con, Option::None).unwrap());
        }
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
    C: redis::aio::ConnectionLike + Send + Sync,
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
