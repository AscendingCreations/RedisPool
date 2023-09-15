use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use crossbeam_queue::ArrayQueue;
use redis::aio::{Connection, Monitor, PubSub};
use redis::{aio::ConnectionLike, Cmd, RedisFuture, Value};
use tokio::sync::OwnedSemaphorePermit;

pub struct RedisPoolConnection<C>
where
    C: redis::aio::ConnectionLike + Send,
{
    // This field can be safley unwrapped because it is always initialized to Some
    // and only set to None when dropped or detached
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

    pub fn detach(mut self) -> C {
        self.con.take().unwrap()
    }
}

impl<C> Drop for RedisPoolConnection<C>
where
    C: redis::aio::ConnectionLike + Send,
{
    fn drop(&mut self) {
        if let Some(con) = self.con.take() {
            let _ = self.queue.push(con);
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

impl RedisPoolConnection<Connection> {
    pub fn into_pubsub(mut self) -> RedisPoolPubSub {
        let permit = self.permit.take();
        let queue = self.queue.clone();
        RedisPoolPubSub::new(self.detach().into_pubsub(), permit, queue)
    }

    pub fn into_monitor(self) -> Monitor {
        self.detach().into_monitor()
    }
}

pub struct RedisPoolPubSub {
    // This field can be safley unwrapped because it is always initialized to Some
    // and only set to None when dropped or detached
    pubsub: Option<PubSub>,
    permit: Option<OwnedSemaphorePermit>,
    queue: Arc<ArrayQueue<Connection>>,
}

impl RedisPoolPubSub {
    pub fn new(
        pubsub: PubSub,
        permit: Option<OwnedSemaphorePermit>,
        queue: Arc<ArrayQueue<Connection>>,
    ) -> Self {
        RedisPoolPubSub {
            pubsub: Some(pubsub),
            permit,
            queue,
        }
    }

    pub fn detach(mut self) -> PubSub {
        self.pubsub.take().unwrap()
    }

    pub async fn into_connection(mut self) -> RedisPoolConnection<Connection> {
        let permit = self.permit.take();
        let queue = self.queue.clone();
        RedisPoolConnection::new(self.detach().into_connection().await, permit, queue)
    }
}

impl Drop for RedisPoolPubSub {
    fn drop(&mut self) {
        if let Some(pubsub) = self.pubsub.take() {
            let permit = self.permit.take();
            let queue = self.queue.clone();
            tokio::spawn(async move {
                let _permit = permit;
                let _ = queue.push(pubsub.into_connection().await);
            });
        }
    }
}

impl Deref for RedisPoolPubSub {
    type Target = PubSub;

    fn deref(&self) -> &Self::Target {
        self.pubsub.as_ref().unwrap()
    }
}

impl DerefMut for RedisPoolPubSub {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.pubsub.as_mut().unwrap()
    }
}
