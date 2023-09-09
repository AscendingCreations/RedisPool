#!/bin/bash

cd "$(dirname "$0")"
cd ./redis
docker build . --tag redis-single
cd ..
cd ./cluster
docker build . --tag redis-cluster

