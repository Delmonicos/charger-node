#!/bin/bash
echo 'Updating Rust'
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

