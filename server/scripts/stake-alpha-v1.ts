import * as anchor from "@coral-xyz/anchor";
import { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  PublicKey,
  SYSVAR_RENT_PUBKEY,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import * as crypto from "crypto";
import * as fs from "fs";
import * as path from "path";

const IDL_PATH = path.resolve(__dirname, "../target/idl/my_first_solana_program.json");
const PROGRAM_ID = new PublicKey("HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY");
const STAKING_USDC_VAULT = new PublicKey("9nAUb7QG3mALgEUQZ26fHRa4p9BkfvKV5xGp6NFXA8wQ");
const DEFAULT_STAKE_AMOUNT = "100000000";
const DEFAULT_LOCK_TIER = "0";
const U64_MAX = 18_446_744_073_709_551_615n;

const SEEDS = {
  stakingPoolV1: "staking_pool_v1",
  alphaStakingVault: "alpha_staking_vault",
  alphaVaultAuthorityV1: "alpha_vault_authority_v1",
  userStakeAccount: "user_stake_account",
} as const;

type RuntimeIdl = {
  address?: string;
  instructions: Array<{ name: string }>;
};

function requireEnv(names: string[]): void {
  const missing = names.filter((name) => !process.env[name]);
  if (missing.length > 0) {
    throw new Error(`Missing required environment variable(s): ${missing.join(", ")}`);
  }
}

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

function readLockTier(): number {
  const raw = process.env.LOCK_TIER ?? DEFAULT_LOCK_TIER;
  if (!/^\d+$/.test(raw)) {
    throw new Error("LOCK_TIER must be an integer from 0 to 4");
  }

  const tier = Number(raw);
  if (!Number.isInteger(tier) || tier < 0 || tier > 4) {
    throw new Error("LOCK_TIER must be an integer from 0 to 4");
  }

  return tier;
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

function anchorDiscriminator(name: string): Buffer {
  return crypto.createHash("sha256").update(`global:${name}`).digest().subarray(0, 8);
}

function derivePda(seed: string): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync([Buffer.from(seed)], PROGRAM_ID);
  return pda;
}

function deriveUserStakeAccount(owner: PublicKey): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.userStakeAccount), owner.toBuffer()],
    PROGRAM_ID,
  );
  return pda;
}

function u64Buffer(value: bigint): Buffer {
  const buffer = Buffer.alloc(8);
  buffer.writeBigUInt64LE(value);
  return buffer;
}

async function main(): Promise<void> {
  const idl = loadIdl();
  console.log(
    "Runtime IDL instruction names:",
    idl.instructions.map((ix) => ix.name),
  );

  const discriminator = anchorDiscriminator("stake_alpha");
  console.log("stake_alpha discriminator hex:", discriminator.toString("hex"));

  requireEnv(["ALPHA_MINT", "USDC_MINT"]);

  const alphaMint = readRequiredPublicKey("ALPHA_MINT");
  const usdcMint = readRequiredPublicKey("USDC_MINT");
  const stakeAmount = readU64("STAKE_AMOUNT", DEFAULT_STAKE_AMOUNT);
  const lockTier = readLockTier();

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const owner = provider.wallet.publicKey;
  const stakingPool = derivePda(SEEDS.stakingPoolV1);
  const userStakeAccount = deriveUserStakeAccount(owner);
  const ownerAlphaAta = getAssociatedTokenAddressSync(alphaMint, owner);
  const alphaVaultAuthority = derivePda(SEEDS.alphaVaultAuthorityV1);
  const alphaStakingVault = derivePda(SEEDS.alphaStakingVault);

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("Owner:", owner.toBase58());
  console.log("ALPHA Mint:", alphaMint.toBase58());
  console.log("USDC Mint:", usdcMint.toBase58());
  console.log("STAKE_AMOUNT:", stakeAmount.toString());
  console.log("LOCK_TIER:", lockTier.toString());
  console.log("staking_pool_v1:", stakingPool.toBase58());
  console.log("user_stake_account:", userStakeAccount.toBase58());
  console.log("owner_alpha_ata:", ownerAlphaAta.toBase58());
  console.log("alpha_vault_authority_v1:", alphaVaultAuthority.toBase58());
  console.log("alpha_staking_vault:", alphaStakingVault.toBase58());
  console.log("staking_usdc_vault:", STAKING_USDC_VAULT.toBase58());

  const ix = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: stakingPool, isSigner: false, isWritable: true },
      { pubkey: userStakeAccount, isSigner: false, isWritable: true },
      { pubkey: owner, isSigner: true, isWritable: true },
      { pubkey: ownerAlphaAta, isSigner: false, isWritable: true },
      { pubkey: alphaMint, isSigner: false, isWritable: false },
      { pubkey: alphaVaultAuthority, isSigner: false, isWritable: false },
      { pubkey: alphaStakingVault, isSigner: false, isWritable: true },
      { pubkey: STAKING_USDC_VAULT, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ],
    data: Buffer.concat([discriminator, u64Buffer(stakeAmount), Buffer.from([lockTier])]),
  });

  const signature = await provider.sendAndConfirm(new Transaction().add(ix), []);
  console.log("Transaction signature:", signature);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
