# Use a build stage that has glibc instead of musl
FROM rust:1.64-bullseye as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    git \
    gcc \
    binutils \
    clang \
    libclang-dev \
    llvm-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Set environment variables
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=$CARGO_HOME/bin:$PATH
ENV RUSTFLAGS='-C linker=clang'
ENV CC=clang
ENV AR=llvm-ar

ARG TARGETPLATFORM
RUN \
  case $TARGETPLATFORM in \
  'linux/amd64') arch="x86_64" ;; \
  'linux/arm64') arch="aarch64" ;; \
  esac; \
  echo "${arch}-unknown-linux-gnu" > /tmp/target;

WORKDIR /target/src
COPY rust-toolchain .
RUN rustup-init -y --no-modify-path --profile minimal --default-toolchain $(cat rust-toolchain)
RUN rustup target add $(cat /tmp/target)

# Trigger fetch of crates index
RUN cargo search --limit 1

COPY . .
RUN cargo build --release --target-dir /target --target $(cat /tmp/target) && \
      mv /target/release/zkpool-prover / && rm -rf /target

# Use a base image that includes glibc
FROM debian:bullseye-slim
COPY --from=builder /zkpool-prover /zkpool-prover
ENTRYPOINT ["/zkpool-prover"]
