#![allow(dead_code)]

pub mod cluster;
pub mod server;

use testcontainers::{clients::Cli, core::WaitFor, images::generic::GenericImage, Container};

const REDIS_IMG_NAME: &str = "redis";
const REDIS_IMG_VER: &str = "alpine";
const REDIS_PORT: u16 = 6379;
const REDIS_READY_MSG: &str = "Ready to accept connections tcp";

fn create_redis_image() -> GenericImage {
    let wait = WaitFor::message_on_stdout(REDIS_READY_MSG);
    GenericImage::new(REDIS_IMG_NAME, REDIS_IMG_VER)
        .with_wait_for(wait)
        .with_exposed_port(REDIS_PORT)
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