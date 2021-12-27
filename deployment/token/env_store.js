import fs from 'fs';
import { stringify } from 'envfile';
import dotenv from 'dotenv';

export const saveToEnv = (variableName, value) => {
  console.log(`Updating ${variableName} in the '.env' file. New value: ${value}`);
  const parsedFile = dotenv.config().parsed;
  parsedFile[variableName] = value;
  fs.writeFileSync('./.env', stringify(parsedFile));
}

export const getFromEnv = (variableName) => {
  const parsedFile = dotenv.config().parsed;
  return parsedFile[variableName];
}

export const getNumberFromEnv = (name) => Number(getFromEnv(name));
