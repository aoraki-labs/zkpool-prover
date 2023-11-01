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

### Build and Run

Note:just for some particular test 
```
   cargo build --release
   cd target/release/
   wget https://storage.googleapis.com/zkevm-circuits-keys/kzg_bn254_22.srs -P ./
   ./zkpool-prover
```

## License

AGPL-3.0-or-later








