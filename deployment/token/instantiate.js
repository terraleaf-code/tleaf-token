import dotenv from 'dotenv'; dotenv.config(); // It must be called as early as possible
import { saveToEnv, getNumberFromEnv } from './env_store';
import terraBase from './terra_base';
import fs from "fs";

const TOTAL_SUPPLY = String(1000_000_000_000_000); // 1 billion with 6 decimal places

// Preparing init message
const adminAddress = terraBase.getWalletDetails().address;
const initMsg = JSON.parse(fs.readFileSync(`token/instantiate_msg.json`));
initMsg['admins'] = [adminAddress]
initMsg['initial_balances'] = [{
  address: adminAddress,
  amount: TOTAL_SUPPLY,
}];

// Contract instantiation
const contractAddress = await terraBase.instantiateContract({
  codeId: getNumberFromEnv(`${process.env.CONTRACT_TOKEN_NAME}_CODE_ID`),
  initMsg,
  txMemo: `Instatiate token contract - ${process.env.CONTRACT_TOKEN_NAME}`,
});

// Update .env file with the new contract address
saveToEnv(`${process.env.CONTRACT_TOKEN_NAME}_ADDRESS`, contractAddress);
