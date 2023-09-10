use thiserror::Error;
use tokio::sync::AcquireError;

#[derive(Error, Debug)]
pub enum RedisPoolError {
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error(transparent)]
    AcquireError(#[from] AcquireError),
}
