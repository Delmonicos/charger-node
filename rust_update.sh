#!/bin/bash
sudo apt install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev curl libz-dev
echo 'Updating Rust'
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

