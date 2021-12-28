import dotenv from 'dotenv';
import {isTxError, LCDClient, MnemonicKey, MsgInstantiateContract, MsgStoreCode,} from '@terra-money/terra.js';
import fs from 'fs';
import sleep from 'sleep';

dotenv.config();

const DEFAULT_MNEMONIC = process.env.MNEMO;
const DEFAULT_NETWORK_URL = process.env.NETWORK_URL;
const DEFAULT_CHAIN_ID = process.env.NETWORK_CHAIN_ID;
const DEFAULT_WALLET_CONFIG = {
  mnemonic: DEFAULT_MNEMONIC,
  url: DEFAULT_NETWORK_URL,
  chainID: DEFAULT_CHAIN_ID,
};

const DEFAULT_SLEEP_TIME_MS = 2000;

export function getWalletDetails(walletConfig = DEFAULT_WALLET_CONFIG) {
  const mk = new MnemonicKey({mnemonic: walletConfig.mnemonic});
  const lcdOptions = {
    chainID: walletConfig.chainID,
    URL: walletConfig.url,
  };

  const terra = new LCDClient(lcdOptions);
  const wallet = terra.wallet(mk);
  const address = wallet.key.accAddress;

  return {wallet, terra, address};
}

export async function instantiateContract({
                                            codeId,
                                            initMsg,
                                            txMemo,
                                            walletConfig = DEFAULT_WALLET_CONFIG,
                                          }) {
  // Prepare wallet
  const {wallet, terra, address} = getWalletDetails(walletConfig);
  console.log(`Contract instantiation from wallet: ${address}`);
  console.log(initMsg);

  // Prepare the instantiate transaction
  const instantiateMsg = new MsgInstantiateContract(
    wallet.key.accAddress,
    wallet.key.accAddress,
    codeId,
    initMsg,
    {});
  const instantiateTx = await wallet.createAndSignTx({
    msgs: [instantiateMsg],
    memo: txMemo,
  });

  // Posting the transaction
  const instantiateTxResult = await terra.tx.broadcast(instantiateTx);

  // Analyzing tx result
  console.log(instantiateTxResult);
  if (isTxError(instantiateTxResult)) {
    throw new Error(
      `Instantiate failed. code: ${instantiateTxResult.code},
        codespace: ${instantiateTxResult.codespace},
        raw_log: ${instantiateTxResult.raw_log}`
    );
  }

  await sleep.msleep(DEFAULT_SLEEP_TIME_MS);

  // Extracting contract address from the tx result
  const events = instantiateTxResult.logs[0].eventsByType;
  const contractAddress = events.instantiate_contract.contract_address;
  console.log(`Deployed contract address: ${contractAddress}`);

  // Returning the deployed contract address
  return contractAddress[0];
}

export async function storeContractCode({
                                          filePath,
                                          txMemo,
                                          walletConfig = DEFAULT_WALLET_CONFIG,
                                        }) {
  // Prepare wallet
  const {wallet, address, terra} = getWalletDetails(walletConfig);
  console.log(`Storing contract code from wallet: ${address}`);

  // Prepare store message
  const storeCode = new MsgStoreCode(
    address,
    fs.readFileSync(filePath).toString('base64')
  );
  const storeCodeTx = await wallet.createAndSignTx({
    msgs: [storeCode],
    memo: txMemo,
  });

  // Tx broadcasting
  const storeCodeTxResult = await terra.tx.broadcast(storeCodeTx);

  // Analyzing tx result
  console.log(storeCodeTxResult);
  if (isTxError(storeCodeTxResult)) {
    throw new Error(
      `Store code failed. code: ${storeCodeTxResult.code},
      codespace: ${storeCodeTxResult.codespace},
      raw_log: ${storeCodeTxResult.raw_log}`
    );
  }

  // Extracting code id from the tx result
  const events = storeCodeTxResult.logs[0].eventsByType;
  const codeId = events.store_code.code_id;
  console.log(`Stored code id: ${codeId}`);

  await sleep.msleep(DEFAULT_SLEEP_TIME_MS);

  return codeId;
}

export default {
  instantiateContract,
  storeContractCode,
  getWalletDetails,
};
