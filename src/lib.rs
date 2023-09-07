#![doc = include_str!("../README.md")]
#![allow(dead_code)]

mod errors;
mod manager;
mod pool;

pub use errors::RedisError;
pub use manager::RedisConnectionManager;
pub use pool::*;

pub use redis;

#[cfg(feature = "cluster")]
mod clustermanager;

#[cfg(feature = "cluster")]
pub use clustermanager::RedisClusterManager;
