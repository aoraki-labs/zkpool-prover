#!/bin/bash

cargo build --release

cd ./target/release/

if [ ! -f "./19.bin" ];then
    wget https://storage.googleapis.com/zkevm-circuits-keys/19.bin -P ./
else
    echo "kzg parameter 19.bin exist"
fi
if [ ! -f "./21.bin" ];then
    wget https://storage.googleapis.com/zkevm-circuits-keys/21.bin -P ./
else
    echo "kzg parameter 21.bin exist"
fi
if [ ! -f "./kzg_bn254_21.srs" ];then
    wget https://storage.googleapis.com/zkevm-circuits-keys/kzg_bn254_21.srs -P ./
else
    echo "kzg parameter kzg_bn254_21.srs exist"
fi

 ./zkpool-prover -k 123456789 -u 123456789 -p 35.201.232.215:18081 #replace the parameter with your own