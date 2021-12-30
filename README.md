# Terraleaf token

This is a basic implementation of a cw20 contract extended with custom features.

Implements:

- [x] CW20 Base
- [x] Mintable extension
- [x] Allowances extension

Custom features:

- [x] Authorized burn
- [x] Migrate function
- [x] Withdrawal locked tokens

## Compiling and running tests

```sh
# this will produce a wasm build in ./target/wasm32-unknown-unknown/release/YOUR_NAME_HERE.wasm
cargo wasm

# this runs unit tests with helpful backtraces
RUST_BACKTRACE=1 cargo unit-test
```

## Preparing the Wasm bytecode for production

```sh
# This produces an `artifacts` directory with a `PROJECT_NAME.wasm`, as well as `checksums.txt`, containing the Sha256 hash of the wasm file.
sh build-terra-contract.sh
```

## Deploy token 

Follow the `README.md` in the `/deployment` directory
