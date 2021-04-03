FROM rust:latest
LABEL maintainer="fcroiseaux@gmail.com"
LABEL description="This image build and run charger-node."

RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y aptitude && \
    aptitude install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev curl libz-dev

RUN	export PATH="$PATH:$HOME/.cargo/bin" && \
	rustup toolchain install nightly && \
	rustup update && \
	rustup update nightly && \
	rustup target add wasm32-unknown-unknown --toolchain nightly && \
	rustup default stable

ENV CARGO_HOME=/home/rust/src/cargo_home
RUN useradd rust --user-group --create-home --shell /bin/bash --groups sudo
WORKDIR /home/rust/src
COPY . /home/rust/src
RUN cargo build --release
CMD ./target/release/charger-node --dev --tmp
