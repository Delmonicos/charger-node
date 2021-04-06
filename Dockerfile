FROM debian:stretch-slim

LABEL maintainer="fcroiseaux@gmail.com"
LABEL description="This image build and run charger-node."

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        gcc \
        libc6-dev \
        wget \
        ; \
    \
    url="https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init"; \
    wget "$url"; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --default-toolchain nightly; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
    \
    apt-get remove -y --auto-remove \
        wget \
        ; \
    rm -rf /var/lib/apt/lists/*;

RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev curl libz-dev

RUN	export PATH="$PATH:$HOME/.cargo/bin" && \
	rustup toolchain install nightly && \
	rustup update && \
	rustup update nightly && \
	rustup target add wasm32-unknown-unknown --toolchain nightly && \
	rustup default stable

RUN useradd rust --user-group --create-home --shell /bin/bash --groups sudo
WORKDIR /home/rust/src
COPY . /home/rust/src
RUN cargo build --release
CMD ./target/release/charger-node --dev --tmp \
  --port 30333 \
  --ws-port 9945 \
  --rpc-port 8080 
