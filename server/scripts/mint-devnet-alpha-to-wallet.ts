import * as anchor from "@coral-xyz/anchor";
import {
  createAssociatedTokenAccountInstruction,
  createMintToCheckedInstruction,
  getAssociatedTokenAddressSync,
  getMint,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { PublicKey, Transaction } from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";

const IDL_PATH = path.resolve(__dirname, "../target/idl/my_first_solana_program.json");
const PROGRAM_ID = new PublicKey("HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY");
const DEFAULT_ALPHA_AMOUNT = "1000000000";
const U64_MAX = 18_446_744_073_709_551_615n;

type RuntimeIdl = {
  address?: string;
  instructions: Array<{ name: string }>;
};

function readRequiredPublicKey(name: string): PublicKey {
  const value = process.env[name];
  if (!value) {
    throw new Error(`Missing required environment variable: ${name}`);
  }

  try {
    return new PublicKey(value);
  } catch {
    throw new Error(`Invalid public key in ${name}: ${value}`);
  }
}

function readU64(name: string, defaultValue: string): bigint {
  const raw = process.env[name] ?? defaultValue;
  if (!/^\d+$/.test(raw)) {
    throw new Error(`${name} must be a non-negative integer string`);
  }

  const value = BigInt(raw);
  if (value === 0n) {
    throw new Error(`${name} must be greater than 0`);
  }
  if (value > U64_MAX) {
    throw new Error(`${name} must fit in an unsigned 64-bit integer`);
  }

  return value;
}

function loadIdl(): RuntimeIdl {
  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf8")) as RuntimeIdl;

  if (idl.address && idl.address !== PROGRAM_ID.toBase58()) {
    throw new Error(
      `IDL program ID mismatch. Expected ${PROGRAM_ID.toBase58()}, got ${idl.address}`,
    );
  }

  idl.address = PROGRAM_ID.toBase58();
  return idl;
}

async function main(): Promise<void> {
  const idl = loadIdl();
  console.log(
    "Runtime IDL instruction names:",
    idl.instructions.map((ix) => ix.name),
  );

  const alphaMint = readRequiredPublicKey("ALPHA_MINT");
  const alphaAmount = readU64("ALPHA_AMOUNT", DEFAULT_ALPHA_AMOUNT);

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = provider.wallet.publicKey;
  const mintInfo = await getMint(provider.connection, alphaMint);
  const walletAlphaAta = getAssociatedTokenAddressSync(alphaMint, wallet);
  const ataInfo = await provider.connection.getAccountInfo(walletAlphaAta);
  const tx = new Transaction();

  if (!ataInfo) {
    tx.add(createAssociatedTokenAccountInstruction(wallet, walletAlphaAta, wallet, alphaMint));
  }

  tx.add(
    createMintToCheckedInstruction(
      alphaMint,
      walletAlphaAta,
      wallet,
      alphaAmount,
      mintInfo.decimals,
      [],
      TOKEN_PROGRAM_ID,
    ),
  );

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("Wallet / mint authority:", wallet.toBase58());
  console.log("ALPHA Mint:", alphaMint.toBase58());
  console.log("ALPHA decimals:", mintInfo.decimals);
  console.log("wallet_alpha_ata:", walletAlphaAta.toBase58());
  console.log("ALPHA_AMOUNT:", alphaAmount.toString());
  console.log("ATA existed:", Boolean(ataInfo));

  const signature = await provider.sendAndConfirm(tx, []);
  console.log("Transaction signature:", signature);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
