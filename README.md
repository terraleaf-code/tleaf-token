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

### Prerequisites
- You should have Node.js, NPM and yarn installed on your system
- Terra wallet (mnemonic key) with some UST tokens to pay for deployment

### Initial steps
- Install dependencies with `yarn install`
- Configure `.env` file. To configure `.env` file you can copy and edit [.sample.env](.sample.env).
Don't forget to update the `MNEMO` variable in the .env file with your mnemonic key.
Don't forget to update the `NETWORK_URL` and `NETWORK_CHAIN_ID` to select your network (testnet is configured in sample file).

### Commands

Follow the commands below:

```bash
# Store token code in the blockchain network
yarn store_token

# Instantiate token contract based on deployment/token/instantiate_msg.json configuration and deployment/token/instantiate.js script
yarn instantiate_token
```

The `.env` file will be updated with `*_CODE_ID` and `*_ADDRESS`.
The first variable is code id and the second is contract address.
