import dotenv from 'dotenv'; dotenv.config();
import { saveToEnv } from './env_store';
import terraBase from './terra_base';

const codeId = await terraBase.storeContractCode({
  filePath: `../artifacts/tleaf_token.wasm`,
  txMemo: `Store token contract - ${process.env.CONTRACT_TOKEN_NAME}`,
});

saveToEnv(`${process.env.CONTRACT_TOKEN_NAME}_CODE_ID`, codeId);
