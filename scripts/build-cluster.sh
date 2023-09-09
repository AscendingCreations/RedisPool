#!/bin/bash

cd "$(dirname "$0")"
cd ../docker
docker build . --tag redis-cluster