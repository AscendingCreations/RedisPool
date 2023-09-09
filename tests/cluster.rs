mod utils;

use futures::future::join_all;
use redis_pool::ClusterRedisPool;
use serial_test::serial;

use crate::utils::cluster::TestClusterContext;

#[tokio::test]
#[serial]
pub async fn test_simple_get_set_series() -> anyhow::Result<()> {
    let cluster = TestClusterContext::new_with_cluster_client_builder(6, 1, |builder| {
        builder.read_from_replicas()
    });
    cluster.wait_for_cluster_up();
    let pool = ClusterRedisPool::from(cluster.client);

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
#[serial]
pub async fn test_simple_get_set_parrallel() -> anyhow::Result<()> {
    let cluster = TestClusterContext::new_with_cluster_client_builder(6, 1, |builder| {
        builder.read_from_replicas()
    });
    cluster.wait_for_cluster_up();
    let pool = ClusterRedisPool::from(cluster.client);
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
