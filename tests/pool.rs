use redis_pool::pool::RedisPool;
use testcontainers::{core::WaitFor, images::generic::GenericImage, *};

const REDIS_IMG_NAME: &str = "redis";
const REDIS_IMG_VER: &str = "alpine";
const REDIS_PORT: u16 = 6379;
const REDIS_READY_MSG: &str = "Ready to accept connections tcp";

#[tokio::test]
pub async fn test_simple_get_set() -> anyhow::Result<()> {
    let docker = clients::Cli::default();
    let wait = WaitFor::message_on_stdout(REDIS_READY_MSG);
    let image = GenericImage::new(REDIS_IMG_NAME, REDIS_IMG_VER)
        .with_wait_for(wait)
        .with_exposed_port(REDIS_PORT);
    let redis = docker.run(image);
    let port = redis.get_host_port_ipv4(REDIS_PORT);
    let client = redis::Client::open(format!("redis://localhost:{}", port))
        .expect("Failed to connect to redis");

    let pool = RedisPool::from(client);

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
