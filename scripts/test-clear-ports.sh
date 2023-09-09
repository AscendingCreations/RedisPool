#!/bin/bash

ports=(6379 6380 6381)
for port in "${ports[@]}"; do
    sudo kill $(sudo lsof -t -i:$port)
done