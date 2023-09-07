use crate::RedisConnectionManager;
use crate::SharedPool;
use redis::aio::Connection;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

const DEREF_ERR: &str = "Connection already released to pool";

pub struct RedisConnection {
    pub connection: Option<Connection>,
    pub(crate) pool: Option<Arc<SharedPool>>,
}

impl RedisConnection {
    pub(crate) fn new(pool: Arc<SharedPool>, connection: Connection) -> Self {
        Self {
            connection: Some(connection),
            pool: Some(pool),
        }
    }

    pub fn detach(mut self) -> Connection {
        self.connection
            .take()
            .expect("PoolConnection double-dropped")
    }
}

impl Drop for RedisConnection {
    fn drop(&mut self) {
        if self.connection.is_some() && self.pool.is_some() {
            let shared = self.pool.take().unwrap();
            let connection = self.connection.take().unwrap();
            {
                let pool = shared.pool.blocking_lock();

                if pool.len() < pool.capacity() {
                    // pool has space lets insert it back into the Pool
                    if pool.push(connection).is_err() {
                        panic!("Queue was maxed out");
                    }
                }
            }
        }
    }
}

impl Deref for RedisConnection {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        self.connection.as_ref().expect(DEREF_ERR)
    }
}

impl DerefMut for RedisConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.connection.as_mut().expect(DEREF_ERR)
    }
}

impl AsRef<Connection> for RedisConnection {
    fn as_ref(&self) -> &Connection {
        self
    }
}

impl AsMut<Connection> for RedisConnection {
    fn as_mut(&mut self) -> &mut Connection {
        self
    }
}

pub trait AttachRedis {
    fn attach(self, manager: RedisConnectionManager) -> RedisConnection;
}

impl AttachRedis for Connection {
    fn attach(self, manager: RedisConnectionManager) -> RedisConnection {
        RedisConnection::new(Arc::clone(&manager.shared), self)
    }
}
