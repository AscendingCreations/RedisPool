#!/bin/bash

# This runs script allows redis container to respect start and stop commands

# Define cleanup function
stop_redis() {
    echo "==========   Stopping Redis server...   =========="
    redis-cli shutdown
    exit 0
}

# Trap SIGINT and SIGTERM signals and run cleanup function
trap stop_redis SIGINT
trap stop_redis SIGTERM

# Start redis
redis-server &
wait %?redis-server