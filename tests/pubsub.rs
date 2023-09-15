mod utils;

use futures::StreamExt;
use redis_pool::pool::RedisPool;
use testcontainers::clients::{self};
use utils::TestRedis;

#[tokio::test]
pub async fn test_pubsub() {
    let docker = clients::Cli::default();
    let redis = TestRedis::new(&docker);
    let pool = RedisPool::from(redis.client());

    let mut rx = pool.aquire().await.unwrap().into_pubsub();
    let _ = rx.subscribe("test_channel").await;

    let mut tx = pool.aquire().await.unwrap();
    let _: () = redis::cmd("PUBLISH")
        .arg("test_channel")
        .arg("test")
        .query_async(&mut tx)
        .await
        .unwrap();

    assert_eq!(
        "test",
        rx.on_message()
            .next()
            .await
            .unwrap()
            .get_payload::<String>()
            .unwrap()
    );
}

#[tokio::test]
pub async fn test_monitor() {
    let docker = clients::Cli::default();
    let redis = TestRedis::new(&docker);
    let pool = RedisPool::from(redis.client());

    let mut rx = pool
        .factory()
        .get_async_connection()
        .await
        .unwrap()
        .into_monitor();
    let _ = rx.monitor().await;

    let mut tx = pool.aquire().await.unwrap();
    let _: () = redis::cmd("PING")
        .arg("test")
        .query_async(&mut tx)
        .await
        .unwrap();

    let monitor = rx.on_message::<String>().next().await.unwrap();
    let monitor = monitor.split(" ").collect::<Vec<_>>();

    assert_eq!("\"PING\"", monitor[3]);
    assert_eq!("\"test\"", monitor[4]);
}
