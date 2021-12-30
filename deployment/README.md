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