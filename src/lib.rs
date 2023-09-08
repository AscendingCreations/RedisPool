#![allow(dead_code)]
#![doc = include_str!("../README.md")]

pub mod connection;
pub mod errors;
pub mod factory;
pub mod pool;

#[cfg(feature = "cluster")]
pub mod cluster;
