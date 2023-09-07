use crate::{
    pool::{RedisConnection, SharedPool},
    RedisError,
};
#[cfg(feature = "axum")]
use async_trait::async_trait;
#[cfg(feature = "axum")]
use axum_core::extract::{FromRef, FromRequestParts};
#[cfg(feature = "axum")]
use http::request::Parts;

use redis::Client;
use std::{fmt, sync::Arc};

#[derive(Clone)]
pub struct RedisConnectionManager {
    pub(crate) shared: Arc<SharedPool>,
}

impl RedisConnectionManager {
    /// Constructs a RedisConnectionManager.
    ///
    /// limit is the max size of how many connections we will allow to exist within the pool.
    /// All other connections will get dropped after use if the pool is full.
    ///
    /// # Examples
    /// ```rust no_run
    /// use redis_pool::{RedisConnectionManager};
    ///
    /// let client = redis::Client::open("redis://default:YourSecretPassWord@127.0.0.1:6379/0")
    ///     .expect("Error while trying to connect");
    /// let redis_manager = RedisConnectionManager::new(client, 5);
    /// ```
    ///
    pub fn new(client: Client, limit: u32) -> Self {
        Self {
            shared: SharedPool::new_arc(client, limit),
        }
    }

    /// Aquires an RedisConnection.
    ///
    /// # Examples
    /// ```rust no_run
    /// use redis_pool::{RedisConnectionManager};
    ///
    /// let redis_connection = redis_manager.aquire().await.unwrap();
    /// ```
    ///
    pub async fn aquire(&self) -> Result<RedisConnection, RedisError> {
        let shared: Arc<SharedPool> = Arc::clone(&self.shared);
        shared.aquire().await
    }
}

#[cfg(feature = "axum")]
#[async_trait]
impl<S> FromRequestParts<S> for RedisConnectionManager
where
    S: Send + Sync,
    RedisConnectionManager: FromRef<S>,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(RedisConnectionManager::from_ref(state))
    }
}

impl fmt::Debug for RedisConnectionManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RedisConnectionManager").finish()
    }
}

