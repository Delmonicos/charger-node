#!/bin/bash
# apt-get update
# apt-get install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev curl libz-dev


echo 'Updating Rust'
rustup default stable
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

