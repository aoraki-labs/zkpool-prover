## Introduction

This is a universal modular prover that is built upon various zero-knowledge proof (ZKP) provers. 
It is used to connect with ZKPool in order to obtain proving tasks and generate proofs.

### Supported ZKP projects

1. Taiko A5
2. More is coming

### Hardware Requirements

#### CPU version

1. x86 Ubuntu 20.04/22.04
- 8 or 16 core CPU
- 32 GB memory
2. Intel/M1 Apple macOS

#### GPU version

Coming soon.

## Run from the prebuilt binary

### Download the prebuilt binary

Visit https://github.com/aoraki-labs/zkpool-prover/releases and download the latest release or use wget command like below. 
Please ensure that you select the appropriate tar file for your hardware and the correct release version.
```
   wget https://github.com/aoraki-labs/zkpool-prover/releases/download/v1.0-taiko-a5/x86_64-unknown-linux-musl.tar.gz
   tar -zxvf x86_64-unknown-linux-musl.tar.gz
```

### Download the running script

Download the script run_prover.sh from https://github.com/aoraki-labs/zkpool-prover/tree/taiko-a5-testnet

### Run

Modify the ./run-prover.sh according to your own config.
```
ACCESS_KEY=123456789 #replace the parameter with what we get in zkpool.io
DEVICE_ID=123456789 #replace the parameter with the id name you want to set
```

Run like this:
```
   chmod +x ./run-prover.sh
   ./run-prover.sh
```

## Build from source and run

### Preparation

Download codes via `git clone`. Make sure you have installed rustup, cargo and Go.
```
   git clone https://github.com/aoraki-labs/zkpool-prover.git
   cd zkpool-prover
```


### Build

Run `cargo build --release` to build the binary.
And then, 
`cp ./target/release/zkpool-prover .`

### Run

Modify the ./run-prover.sh according to your own config.
```
ACCESS_KEY=123456789 #replace the parameter with what we get in zkpool.io
DEVICE_ID=123456789 #replace the parameter with the id name you want to set
```

Run like this:
```
   chmod +x ./run-prover.sh
   ./run-prover.sh
```
You can also see more detail in run-prover.sh and refer to the usage help (`./zkpool-prover --help`):

## Test

### Run TestCase (one a5 proof task in actuality)

```
wget https://storage.googleapis.com/zkevm-circuits-keys/kzg_bn254_22.srs -P ./
cargo test --release -- --nocapture
```



## License

AGPL-3.0-or-later








