#!/bin/bash

launch_redis_and_wait()
{
    sed "s/<PORT>/$2/g" redis.conf.template >> $1/redis.conf
    cd $1
    output=$(mktemp "${TMPDIR:-/tmp/}$(basename $0).XXX")
    redis-server ./redis.conf &> $output &
    until grep -q -i "Ready to accept connections tcp" $output
    do
        sleep 0.5
    done
    cd ..
}

mkdir node1 node2 node3

launch_redis_and_wait node1 6379
launch_redis_and_wait node2 6380
launch_redis_and_wait node3 6381

redis-cli --cluster create "127.0.0.1:6379" "127.0.0.1:6380" "127.0.0.1:6381" --cluster-replicas 0 --cluster-yes

echo CLUSTER READY!

sleep infinity