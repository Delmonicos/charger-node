#!/bin/bash

./target/release/charger-node \
  --alice \
  --tmp \
  --ws-external \
  --rpc-external \
  --rpc-cors=all \
  --port 30333 \
  --ws-port 8080 \
  --rpc-port 440 \
  --rpc-methods=unsafe \
  -lpallet_session_payment=debug,pallet_charge_session=debug,charger_service=debug
