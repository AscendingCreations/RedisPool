use crate::{
    pool::{RedisClusterConnection, SharedClusterPool},
    RedisError,
};
#[cfg(feature = "axum")]
use async_trait::async_trait;
#[cfg(feature = "axum")]
use axum_core::extract::{FromRef, FromRequestParts};
#[cfg(feature = "axum")]
use http::request::Parts;

use redis::cluster::ClusterClient;
use std::{fmt, sync::Arc};

#[derive(Clone)]
pub struct RedisClusterManager {
    pub(crate) shared: Arc<SharedClusterPool>,
}

impl RedisClusterManager {
    /// Constructs a RedisClusterManager.
    ///
    /// limit is the max size of how many connections we will allow to exist within the pool.
    /// All other connections will get dropped after use if the pool is full.
    ///
    /// # Examples
    /// ```rust no_run
    /// use redis_pool::{RedisClusterManager};
    /// let clusterclient = redis::cluster::ClusterClient::
    ///     new(vec!["redis://default:YourSecretPassWord@127.0.0.1:6379/0",])
    ///     .expect("Error while trying to connect");
    /// let redis_manager = RedisClusterManager::new(clusterclient, 5);
    /// ```
    ///
    pub fn new(client: ClusterClient, limit: u32) -> Self {
        Self {
            shared: SharedClusterPool::new_arc(client, limit),
        }
    }

    /// Aquires an RedisClusterConnection.
    ///
    /// # Examples
    /// ```rust no_run
    /// use redis_pool::{RedisClusterManager};
    ///
    /// let redis_connection = redis_manager.aquire().await.unwrap();
    /// ```
    ///
    pub async fn aquire(&self) -> Result<RedisClusterConnection, RedisError> {
        let shared: Arc<SharedClusterPool> = Arc::clone(&self.shared);
        shared.aquire().await
    }
}

#[cfg(feature = "axum")]
#[async_trait]
impl<S> FromRequestParts<S> for RedisClusterManager
where
    S: Send + Sync,
    RedisClusterManager: FromRef<S>,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(RedisClusterManager::from_ref(state))
    }
}

impl fmt::Debug for RedisClusterManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RedisClusterManager").finish()
    }
}
