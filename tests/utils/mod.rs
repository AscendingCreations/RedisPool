#![allow(dead_code)]

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

fn create_redis_cluster_images() -> Vec<GenericImage> {
    vec![]
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

pub struct TestClusterRedis<'a> {
    containers: Vec<Container<'a, GenericImage>>,
}

impl<'a> TestClusterRedis<'a> {
    pub fn new(docker: &'a Cli) -> Self {
        TestClusterRedis {
            containers: create_redis_cluster_images()
                .into_iter()
                .map(|image| docker.run(image))
                .collect(),
        }
    }

    pub fn ports(&self) -> Vec<u16> {
        self.containers
            .iter()
            .map(|container| container.get_host_port_ipv4(REDIS_PORT))
            .collect()
    }

    pub fn client(&self) -> redis::cluster::ClusterClient {
        redis::cluster::ClusterClient::new(
            self.ports()
                .iter()
                .map(|port| format!("redis://127.0.0.1:{}/", port))
                .collect(),
        )
        .expect("Client failed to connect")
    }
}
