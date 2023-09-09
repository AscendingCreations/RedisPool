use std::sync::Arc;

use crossbeam_queue::ArrayQueue;
use redis::{aio::Connection, Client};
use tokio::sync::Semaphore;

use crate::{
    pool::{DEFAULT_CON_LIMIT, DEFAULT_POOL_SIZE},
    RedisPool,
};

pub type SingleRedisPool = RedisPool<Client, Connection>;

impl SingleRedisPool {
    pub fn new(client: Client, pool_size: usize, con_limit: usize) -> Self {
        RedisPool {
            client,
            queue: Arc::new(ArrayQueue::new(pool_size)),
            sem: Arc::new(Semaphore::new(con_limit)),
        }
    }
}

impl From<Client> for SingleRedisPool {
    fn from(value: Client) -> Self {
        SingleRedisPool::new(value, DEFAULT_POOL_SIZE, DEFAULT_CON_LIMIT)
    }
}
