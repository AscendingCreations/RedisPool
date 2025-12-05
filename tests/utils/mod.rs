#![allow(dead_code)]

use std::ops::Deref;

use async_trait::async_trait;
use futures::FutureExt;
use redis::{
    aio::{ConnectionLike, MultiplexedConnection},
    Client, Cmd, ErrorKind, RedisError, RedisFuture, RedisResult, Value,
};
use redis_pool::factory::ConnectionFactory;
use testcontainers::{
    clients::Cli, core::WaitFor, images::generic::GenericImage, Container, RunnableImage,
};

const REDIS_IMG_NAME: &str = "redis-single";
const REDIS_IMG_VER: &str = "latest";
const REDIS_PORT: u16 = 6379;
const REDIS_READY_MSG: &str = "Ready to accept connections tcp";

const REDIS_CLUSTER_IMG_NAME: &str = "redis-cluster";
const REDIS_CLUSTER_IMG_VER: &str = "latest";
const REDIS_CLUSTER_PORTS: [u16; 3] = [6379, 6380, 6380];
const REDIS_CLUSTER_READY_MSG: &str = "CLUSTER READY!";

fn create_redis_image() -> GenericImage {
    let wait = WaitFor::message_on_stdout(REDIS_READY_MSG);
    GenericImage::new(REDIS_IMG_NAME, REDIS_IMG_VER)
        .with_wait_for(wait)
        .with_exposed_port(REDIS_PORT)
}

fn create_redis_cluster_image() -> RunnableImage<GenericImage> {
    let wait = WaitFor::message_on_stdout(REDIS_CLUSTER_READY_MSG);
    let image =
        GenericImage::new(REDIS_CLUSTER_IMG_NAME, REDIS_CLUSTER_IMG_VER).with_wait_for(wait);
    RunnableImage::from(image).with_network("host")
}

pub struct TestRedis<'a> {
    container: Container<'a, GenericImage>,
}

impl<'a> TestRedis<'a> {
    pub fn new(docker: &'a Cli) -> Self {
        TestRedis {
            container: docker.run(create_redis_image()),
        }
    }

    pub fn port(&self) -> u16 {
        self.container.get_host_port_ipv4(REDIS_PORT)
    }

    pub fn client(&self) -> redis::Client {
        redis::Client::open(format!("redis://127.0.0.1:{}/", self.port()))
            .expect("Client failed to connect")
    }
}

impl<'a> Deref for TestRedis<'a> {
    type Target = Container<'a, GenericImage>;

    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

pub struct TestClusterRedis<'a> {
    container: Container<'a, GenericImage>,
}

impl<'a> TestClusterRedis<'a> {
    pub fn new(docker: &'a Cli) -> Self {
        TestClusterRedis {
            container: docker.run(create_redis_cluster_image()),
        }
    }

    pub fn ports(&self) -> Vec<u16> {
        REDIS_CLUSTER_PORTS.into_iter().collect()
    }

    pub fn client(&self) -> redis::cluster::ClusterClient {
        redis::cluster::ClusterClient::builder(
            self.ports()
                .iter()
                .map(|port| format!("redis://127.0.0.1:{}/", port))
                .collect::<Vec<_>>(),
        )
        .build()
        .expect("Client failed to connect")
    }
}

#[derive(Clone)]
pub struct ClosableConnectionFactory(pub Client);

#[async_trait]
impl ConnectionFactory<ClosableConnection> for ClosableConnectionFactory {
    async fn create(&self) -> RedisResult<ClosableConnection> {
        Ok(ClosableConnection::new(
            self.0.get_multiplexed_async_connection().await?,
        ))
    }
}

pub struct ClosableConnection {
    con: MultiplexedConnection,
    open: bool,
}

impl ClosableConnection {
    pub fn new(con: MultiplexedConnection) -> Self {
        ClosableConnection { con, open: true }
    }

    pub fn close(&mut self) {
        self.open = false;
    }
}

impl ConnectionLike for ClosableConnection {
    fn req_packed_command<'a>(&'a mut self, cmd: &'a Cmd) -> RedisFuture<'a, Value> {
        if !self.open {
            (async move { Err(RedisError::from((ErrorKind::Io, "closed connection"))) }).boxed()
        } else {
            self.con.req_packed_command(cmd)
        }
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        if !self.open {
            (async move { Err(RedisError::from((ErrorKind::Io, "closed connection"))) }).boxed()
        } else {
            self.con.req_packed_commands(cmd, offset, count)
        }
    }

    fn get_db(&self) -> i64 {
        if !self.open {
            0
        } else {
            self.con.get_db()
        }
    }
}
