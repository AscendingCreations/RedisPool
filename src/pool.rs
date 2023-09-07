#[cfg(feature = "cluster")]
mod clusterconnection;
#[cfg(feature = "cluster")]
mod clustershared;
mod connection;
mod shared;

pub use connection::RedisConnection;
pub(crate) use shared::SharedPool;

#[cfg(feature = "cluster")]
pub use clusterconnection::RedisClusterConnection;
#[cfg(feature = "cluster")]
pub(crate) use clustershared::SharedClusterPool;
