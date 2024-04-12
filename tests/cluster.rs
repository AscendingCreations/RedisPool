mod utils;

use anyhow::Context;
use futures::future::join_all;
use redis_pool::{ClusterRedisPool, RedisPool};
use serial_test::serial;
use testcontainers::clients::Cli;
use utils::TestClusterRedis;

#[tokio::test]
#[serial]
pub async fn test_simple_get_set_series() -> anyhow::Result<()> {
    let docker = Cli::docker();
    let cluster = TestClusterRedis::new(&docker);
    let pool = RedisPool::from(cluster.client());

    for i in 0..1000 {
        let mut con = pool.aquire().await?;
        let (value,) = redis::Pipeline::with_capacity(2)
            .set(i, i)
            .ignore()
            .get(i)
            .query_async::<_, (i64,)>(&mut con)
            .await?;
        assert_eq!(i, value);
    }

    Ok(())
}

const DATA_SIZE: usize = 1_048_576;
const DATA: [u8; DATA_SIZE] = [1; DATA_SIZE];

#[tokio::test]
#[serial]
pub async fn test_simple_get_set_parrallel() -> anyhow::Result<()> {
    let docker = Cli::docker();
    let cluster = TestClusterRedis::new(&docker);
    let pool = RedisPool::from(cluster.client());

    for value in join_all((0..1000).map(|i| {
        let pool = pool.clone();
        tokio::spawn(async move { get_set_byte_array(i, &pool).await })
    }))
    .await
    {
        let value = value.unwrap().unwrap();
        assert_eq!(&value[..], &DATA[..]);
    }

    Ok(())
}

async fn get_set_byte_array(i: usize, pool: &ClusterRedisPool) -> anyhow::Result<Vec<u8>> {
    let mut con = pool
        .aquire()
        .await
        .context("Failed to establish connection")?;

    let (value,) = redis::Pipeline::with_capacity(2)
        .set(i, &DATA[..])
        .ignore()
        .get(i)
        .query_async::<_, (Vec<u8>,)>(&mut con)
        .await
        .context("Failed to set/get from redis")?;

    Ok(value)
}
