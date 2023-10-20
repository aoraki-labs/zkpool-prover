## Introduction

A standalone Aleo prover build upon snarkOS and snarkVM, with multi-threading optimization for CPU support.


This prover only supports to work with ZKPool.

## Building

Install the dependencies:

```
rust (>= 1.7x)
clang
libssl-dev
pkg-config
```

### CPU version

Run `cargo build --release` to build the binary.


## Usage

Please refer to the usage help (`target/release/zkpool-aleo-prover --help`):

### CPU version (How to run )
```
 ./zkpool-aleo-prover -k M5EYY9exoK7TOi1BKHDtSrXiu5IvqUD0 -u test1017 -p aleo-scheduler.zkpool.info:9999
```
REMIND to replace the -k、-u、-p parameter if needed

## License

AGPL-3.0-or-later
