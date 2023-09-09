#!/bin/bash

ports=(7000 7001 7002 7003 7004 7005)
for port in "${ports[@]}"; do
    sudo kill $(sudo lsof -t -i:$port)
done