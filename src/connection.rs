use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use async_trait::async_trait;
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
        RedisPoolConnectionVariant::new(self.detach().into_pubsub(), permit, queue)
    }

    pub fn into_monitor(mut self) -> RedisPoolMonitor {
        let permit = self.permit.take();
        let queue = self.queue.clone();
        RedisPoolConnectionVariant::new(self.detach().into_monitor(), permit, queue)
    }
}

type RedisPoolPubSub = RedisPoolConnectionVariant<Connection, PubSub>;
type RedisPoolMonitor = RedisPoolConnectionVariant<Connection, Monitor>;

#[async_trait]
pub trait IntoConnection<C>
where
    C: redis::aio::ConnectionLike + Send,
{
    async fn into_connection(self) -> C;
}

#[async_trait]
impl IntoConnection<Connection> for PubSub {
    async fn into_connection(self) -> Connection {
        self.into_connection().await
    }
}

#[async_trait]
impl IntoConnection<Connection> for Monitor {
    async fn into_connection(self) -> Connection {
        self.into_connection().await
    }
}

pub struct RedisPoolConnectionVariant<C, T>
where
    C: redis::aio::ConnectionLike + Send + 'static,
    T: IntoConnection<C> + Send + 'static,
{
    // This field can be safley unwrapped because it is always initialized to Some
    // and only set to None when dropped or detached
    variant: Option<T>,
    permit: Option<OwnedSemaphorePermit>,
    queue: Arc<ArrayQueue<C>>,
}

impl<C, T> RedisPoolConnectionVariant<C, T>
where
    C: redis::aio::ConnectionLike + Send + 'static,
    T: IntoConnection<C> + Send + 'static,
{
    pub fn new(
        variant: T,
        permit: Option<OwnedSemaphorePermit>,
        queue: Arc<ArrayQueue<C>>,
    ) -> Self {
        RedisPoolConnectionVariant {
            variant: Some(variant),
            permit,
            queue,
        }
    }

    pub fn detach(mut self) -> T {
        self.variant.take().unwrap()
    }

    pub async fn into_connection(mut self) -> RedisPoolConnection<C> {
        let permit = self.permit.take();
        let queue = self.queue.clone();
        RedisPoolConnection::new(self.detach().into_connection().await, permit, queue)
    }
}

impl<C, T> Drop for RedisPoolConnectionVariant<C, T>
where
    C: redis::aio::ConnectionLike + Send + 'static,
    T: IntoConnection<C> + Send + 'static,
{
    fn drop(&mut self) {
        if let Some(variant) = self.variant.take() {
            let permit = self.permit.take();
            let queue = self.queue.clone();
            tokio::spawn(async move {
                let _permit = permit;
                let _ = queue.push(variant.into_connection().await);
            });
        }
    }
}

impl<C, T> Deref for RedisPoolConnectionVariant<C, T>
where
    C: redis::aio::ConnectionLike + Send,
    T: IntoConnection<C> + Send,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.variant.as_ref().unwrap()
    }
}

impl<C, T> DerefMut for RedisPoolConnectionVariant<C, T>
where
    C: redis::aio::ConnectionLike + Send,
    T: IntoConnection<C> + Send,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.variant.as_mut().unwrap()
    }
}
