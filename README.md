## Introduction

A standalone uniform zkpool prover build upon zkp circuit


### CPU version

Run `cargo build --release` to build the binary.


## Usage

Please refer to the usage help (`target/release/taiko-prover --help`):

### CPU version
1.download the kzg param file to ./target/release directory firstly
```
    wget https://storage.googleapis.com/zkevm-circuits-keys/19.bin -P ./
    wget https://storage.googleapis.com/zkevm-circuits-keys/21.bin -P ./
    wget https://storage.googleapis.com/zkevm-circuits-keys/kzg_bn254_21.srs -P ./
```

2.how to run 
```
   ./zkpool-prover  -n aoraki.taiko-miner  -p 35.234.20.15:18081 
```
-n: the prover name
-p: the zkpool scheduler pool address 







