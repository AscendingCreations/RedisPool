#![allow(dead_code)]
#![doc = include_str!("../README.md")]

mod connection;
mod errors;
mod factory;
mod pool;

#[cfg(feature = "cluster")]
mod cluster_pool;
