#!/bin/bash
ACCESS_KEY=123456789 #replace the parameter with what we get in zkpool.io
DEVICE_ID=123456789 #replace the parameter with the id name you want to set
POOL_ENDPOINT=lb-mxc4v2nk-v6o3ht41qwmbf0jg.clb.na-siliconvalley.tencentclb.com:18081


chmod +x ./zkpool-prover
 ./zkpool-prover -k $ACCESS_KEY -u $DEVICE_ID -p $POOL_ENDPOINT
