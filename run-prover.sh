#!/bin/bash

cargo build --release
cd ./target/release/
wget https://storage.googleapis.com/zkevm-circuits-keys/19.bin -P ./
wget https://storage.googleapis.com/zkevm-circuits-keys/21.bin -P ./
wget https://storage.googleapis.com/zkevm-circuits-keys/kzg_bn254_21.srs -P ./
 ./zkpool-prover -k PYFkD1n6Q6btC3VcPJ29POm0DOWT7SXT -u 123456789 -p 35.201.232.215:18081 #replace the parameter with your own