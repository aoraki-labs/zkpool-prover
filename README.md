## Introduction

This is a universal modular prover that is built upon various zero-knowledge proof (ZKP) provers. 
It is used to connect with ZKPool in order to obtain proving tasks and generate proofs.

### Supported ZKP projects
1. Taiko A3
2. Taiko A4
3. More is coming

### Hardware Requirements
#### CPU version
1. x86 Ubuntu 20.04/22.04
- 8 or 16 core CPU
- 32 GB memory
2. Intel/M1 Apple macOS

#### GPU version
Coming soon.

## Build from source and run
### Preparation
Download codes via `git clone`. Make sure you have installed rustup, cargo and Go.

### Build

Run `cargo build --release` to build the binary.


### Run
Modify the ./run-prover.sh according to your own config.
```
 ./zkpool-prover -k 123456789 -u 123456789 -p 35.201.232.215:18081 
 ```
The meaning of the parameters of zkpool-prover is like below, and you may replace the access key (get it from zkpool.io website) and device id.
```
-k: the prover access key
-u: the prover device id (optional,program will generate one automatically if it's not set )
```

Run like this:
```
   ./run-prover.sh
```
You can also see more detail in run-prover.sh and refer to the usage help (`target/release/zkpool-prover --help`):

## Run from release binary
Visit https://github.com/aoraki-labs/zkpool-prover/releases and download the latest release or use wget command like below. 
Please ensure that you select the appropriate tar file for your hardware and the correct release version.
```
   wget https://github.com/aoraki-labs/zkpool-prover/releases/download/v1.0/x86_64-unknown-linux-musl.tar.gz
   tar -zxvf x86_64-unknown-linux-musl.tar.gz
```
Download the key files if you do not have them; otherwise, skip this step.
```
   wget https://storage.googleapis.com/zkevm-circuits-keys/19.bin -P ./
   wget https://storage.googleapis.com/zkevm-circuits-keys/21.bin -P ./
   wget https://storage.googleapis.com/zkevm-circuits-keys/kzg_bn254_21.srs -P ./
```
Run like this:
```
   ./zkpool-prover -k 123456789 -u 123456789 -p 35.201.232.215:18081
```


## License

AGPL-3.0-or-later








