#!/bin/bash

# Run this script from the directory with the Cargo project.

docker run --rm -it \
	--env CARGO_HOME=/home/rust/src/cargo_home \
	-v "$(pwd)":/home/rust/src charger-node \
	cargo build --release
