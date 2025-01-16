<h1 align="center">
    RedisPool
</h1>
<div align="center">
    Library to Provide an <a href="https://github.com/redis-rs/redis-rs/tree/main">Redis</a> pool.
</div>
<br />
<div align="center">
    <a href="https://crates.io/crates/redis_pool"><img src="https://img.shields.io/crates/v/redis_pool?style=plastic" alt="crates.io"></a>
    <a href="https://docs.rs/redis_pool"><img src="https://docs.rs/redis_pool/badge.svg" alt="docs.rs"></a>
    <img src="https://img.shields.io/badge/min%20rust-1.60-green.svg" alt="Minimum Rust Version">
    <a href="https://discord.gg/gVXNDwpS3Z"><img src="https://img.shields.io/discord/81844480201728000?label=&labelColor=6A7EC2&logo=discord&logoColor=ffffff&color=7389D8" alt="Discord"></a>
</div>

## License

This project is licensed under either [Apache License, Version 2.0](LICENSE-APACHE), [zlib License](LICENSE-ZLIB), or [MIT License](LICENSE-MIT), at your option.

## Help

If you need help with this library or have suggestions please go to our [Discord Group](https://discord.gg/gVXNDwpS3Z)

## Install

RedisPool uses [`tokio`] runtime.

[`tokio`]: https://github.com/tokio-rs/tokio

```toml
# Cargo.toml
[dependencies]
redis_pool = "0.6.0"
```

#### Cargo Feature Flags

`cluster`: Enables Redis Cluster Client and connections.

# Example

```rust ignore
use redis_pool::{RedisPool, SingleRedisPool};
use axum::{Router, routing::get, extract::State};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let redis_url = "redis://default:YourSecretPassWord@127.0.0.1:6379/0";
    let client = redis::Client::open(redis_url).expect("Error while testing the connection");
    let pool = RedisPool::from(client);

    // build our application with some routes
    let app = Router::new()
        .route("/test", get(test_pool))
        .with_state(pool);

    // run it
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn test_pool(State(pool): State<SingleRedisPool>) -> String {
    let mut connection = pool.acquire().await.unwrap();
    let _: () = redis::pipe()
            .set(0, "Hello")
            .ignore()
            .query_async(&mut connection)
            .await
            .unwrap();

    redis::cmd("GET").arg(0).query_async(&mut connection).await.unwrap()
}
```

## Running Tests

Docker must be installed because this library utilizes [testcontainers](https://github.com/testcontainers/testcontainers-rs) to spin up redis intances. Additionally, the images contained in the `docker` directory need to be built and accessible in your local registry; this can be accomplished by running `./docker/build.sh`.
