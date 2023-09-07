use crate::RedisClusterManager;
use crate::SharedClusterPool;
use redis::cluster_async::ClusterConnection;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

const DEREF_ERR: &str = "Connection already released to pool";

pub struct RedisClusterConnection {
    pub connection: Option<ClusterConnection>,
    pub(crate) pool: Option<Arc<SharedClusterPool>>,
}

impl RedisClusterConnection {
    pub(crate) fn new(pool: Arc<SharedClusterPool>, connection: ClusterConnection) -> Self {
        Self {
            connection: Some(connection),
            pool: Some(pool),
        }
    }

    pub fn detach(mut self) -> ClusterConnection {
        self.connection
            .take()
            .expect("PoolConnection double-dropped")
    }
}

impl Drop for RedisClusterConnection {
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

impl Deref for RedisClusterConnection {
    type Target = ClusterConnection;

    fn deref(&self) -> &Self::Target {
        self.connection.as_ref().expect(DEREF_ERR)
    }
}

impl DerefMut for RedisClusterConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.connection.as_mut().expect(DEREF_ERR)
    }
}

impl AsRef<ClusterConnection> for RedisClusterConnection {
    fn as_ref(&self) -> &ClusterConnection {
        self
    }
}

impl AsMut<ClusterConnection> for RedisClusterConnection {
    fn as_mut(&mut self) -> &mut ClusterConnection {
        self
    }
}

pub trait AttachRedis {
    fn attach(self, manager: RedisClusterManager) -> RedisClusterConnection;
}

impl AttachRedis for ClusterConnection {
    fn attach(self, manager: RedisClusterManager) -> RedisClusterConnection {
        RedisClusterConnection::new(Arc::clone(&manager.shared), self)
    }
}
