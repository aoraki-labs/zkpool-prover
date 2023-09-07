## Introduction

A standalone uniform zkpool prover build upon zkp circuit


### CPU version

Run `cargo build --release` to build the binary.


## Usage

Please refer to the usage help (`target/release/zkpool-prover --help`):


### CPU version
1.run from the run-prover.sh
```
   ./run-prover.sh  #REMIND to replace the parameter in the script by your own,below is the explaination:
```
```
-k: the prover access key
-u: the prover device id (optional,program will generate one automatically if not set )
-p: the zkpool scheduler pool address
you can replace it by your own config
```

2.run from release binary
```
   #Download the binary from the release page:<https://github.com/aoraki-labs/zkpool-prover/releases>,for example:
   wget https://github.com/aoraki-labs/zkpool-prover/releases/download/v1.0/x86_64-unknown-linux-musl.tar.gz 
   tar -zxvf x86_64-unknown-linux-musl.tar.gz

   #Download the param file If not done,otherwise ignore it
   wget https://storage.googleapis.com/zkevm-circuits-keys/19.bin -P ./
   wget https://storage.googleapis.com/zkevm-circuits-keys/21.bin -P ./
   wget https://storage.googleapis.com/zkevm-circuits-keys/kzg_bn254_21.srs -P ./

   #Run
   ./zkpool-prover -k 123456789 -u 123456789 -p 35.201.232.215:18081 #replace the parameter with your own,refer to the upper explaination 

```


## License

AGPL-3.0-or-later








