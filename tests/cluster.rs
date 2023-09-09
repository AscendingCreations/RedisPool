mod utils;

use futures::future::join_all;
use redis_pool::ClusterRedisPool;
use testcontainers::clients;
use utils::TestClusterRedis;

#[tokio::test]
pub async fn test_simple_get_set_series() -> anyhow::Result<()> {
    let docker = clients::Cli::default();
    let redis = TestClusterRedis::new(&docker);
    let pool = ClusterRedisPool::from(redis.client());

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

#[tokio::test]
pub async fn test_simple_get_set_parrallel() -> anyhow::Result<()> {
    let docker = clients::Cli::default();
    let redis = TestClusterRedis::new(&docker);
    let pool = ClusterRedisPool::from(redis.client());
    let data: [u8; 512] = [1; 512];

    join_all((0..1000).map(|i| {
        let i = i;
        let pool = pool.clone();
        tokio::spawn(async move {
            let mut con = pool
                .aquire()
                .await
                .expect("failed to get connection from pool");

            let (value,) = redis::Pipeline::with_capacity(2)
                .set(i, &data[..])
                .ignore()
                .get(i)
                .query_async::<_, (Vec<u8>,)>(&mut con)
                .await
                .expect("Failed to set/get from redis");

            assert_eq!(&data[..], &value[..]);
        })
    }))
    .await;

    Ok(())
}
