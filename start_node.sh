#!/bin/bash

./target/release/charger-node \
  --alice \
  -d /var/lib/chain \
  --ws-external \
  --rpc-external \
  --rpc-cors=all \
  --port 30333 \
  --ws-port 8080 \
  --rpc-port 440