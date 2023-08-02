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

2.how to run (example)
```
   ./zkpool-prover -k PYFkD1n6Q6btC3VcPJ29POm0DOWT7SXT -u 123456789 -p 35.201.232.215:18081
```
```
-k: the prover access key
-u: the prover device id (optional,program will generate one automatically if not set )
-p: the zkpool scheduler pool address
you can replace it by your own config
```







