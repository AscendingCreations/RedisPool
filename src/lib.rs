#![allow(dead_code)]
#![doc = include_str!("../README.md")]

pub mod connection;
pub mod errors;
pub mod factory;
pub mod pool;

pub use pool::RedisPool;
pub use pool::SingleRedisPool;

#[cfg(feature = "cluster")]
pub mod cluster;

#[cfg(feature = "cluster")]
pub use cluster::ClusterRedisPool;
