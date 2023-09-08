use redis::{cluster::ClusterClient, cluster_async::ClusterConnection};

impl RedisPoolShared<ClusterClient, ClusterConnection> {
    pub fn from_cluster_client(client: ClusterClient, limit: usize) -> Self {
        RedisPoolShared {
            client,
            queue: ArrayQueue::new(limit),
            sem: Semaphore::new(limit),
        }
    }
}

impl From<ClusterClient> for RedisPoolShared<ClusterClient, ClusterConnection> {
    fn from(value: ClusterClient) -> Self {
        RedisPoolShared::from_cluster_client(value, DEFAULT_POOL_LIMIT)
    }
}
