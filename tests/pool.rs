mod utils;

use anyhow::Context;
use futures::future::join_all;
use redis::aio::ConnectionLike;
use redis_pool::{pool::RedisPool, SingleRedisPool};
use testcontainers::clients::{self, Cli};
use utils::TestRedis;

use crate::utils::ClosableConnectionFactory;

#[tokio::test]
pub async fn test_simple_get_set_series() -> anyhow::Result<()> {
    let docker = clients::Cli::default();
    let redis = TestRedis::new(&docker);
    let pool = RedisPool::from(redis.client());

    for i in 0..50 {
        let mut con = pool.aquire().await?;
        let (value,) = redis::Pipeline::with_capacity(2)
            .set("test", i)
            .ignore()
            .get("test")
            .query_async::<_, (i64,)>(&mut con)
            .await?;
        assert_eq!(i, value);
    }

    Ok(())
}

const DATA_SIZE: usize = 1_048_576;
const DATA: [u8; DATA_SIZE] = [1; DATA_SIZE];

#[tokio::test]
pub async fn test_simple_get_set_parrallel() -> anyhow::Result<()> {
    let docker = Cli::docker();
    let redis = TestRedis::new(&docker);
    let pool = RedisPool::from(redis.client());

    for value in join_all((0..1000).map(|i| {
        let i = i.to_string();
        let pool = pool.clone();
        tokio::spawn(async move { get_set_byte_array_from_pool(&i, &pool).await })
    }))
    .await
    {
        let value = value.unwrap().unwrap();
        assert_eq!(&value[..], &DATA[..]);
    }

    Ok(())
}

async fn get_set_byte_array_from_pool(
    key: &str,
    pool: &SingleRedisPool,
) -> anyhow::Result<Vec<u8>> {
    let mut con = pool
        .aquire()
        .await
        .context("Failed to establish connection")?;

    get_set_byte_array(key, &mut con).await
}

async fn get_set_byte_array<C: ConnectionLike>(key: &str, con: &mut C) -> anyhow::Result<Vec<u8>> {
    let (value,) = redis::Pipeline::with_capacity(2)
        .set(&key, &DATA[..])
        .ignore()
        .get(&key)
        .query_async::<_, (Vec<u8>,)>(con)
        .await
        .context("Failed to set/get from redis")?;

    Ok(value)
}

#[tokio::test]
pub async fn test_bad_connection_eviction() -> anyhow::Result<()> {
    let docker = Cli::docker();
    let redis = TestRedis::new(&docker);
    let pool = RedisPool::new(ClosableConnectionFactory(redis.client()), 1, 1);
    let mut con = pool.aquire().await.context("Failed to open connection")?;

    con.close();

    get_set_byte_array("foo", &mut con)
        .await
        .err()
        .context("Should fail")?;

    drop(con);

    let mut con = pool.aquire().await.context("Should give new connection")?;

    get_set_byte_array("foo", &mut con)
        .await
        .context("Shouldn't fail")?;

    Ok(())
}
