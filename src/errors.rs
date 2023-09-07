use thiserror::Error;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error("Lock is poisoned {msg} ")]
    LockError { msg: String },
    #[error(transparent)]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("unknown error")]
    Unknown,
}
