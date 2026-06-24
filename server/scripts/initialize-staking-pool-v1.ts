import * as anchor from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
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
const DEFAULT_MIN_CLAIM_USDC = "100000";
const U64_MAX = 18_446_744_073_709_551_615n;

const SEEDS = {
  stakingPoolV1: "staking_pool_v1",
  alphaStakingVault: "alpha_staking_vault",
  alphaVaultAuthorityV1: "alpha_vault_authority_v1",
  vaultAuthorityV2: "vault_authority_v2",
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

function anchorDiscriminator(name: string): Buffer {
  return crypto.createHash("sha256").update(`global:${name}`).digest().subarray(0, 8);
}

function derivePda(seed: string): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync([Buffer.from(seed)], PROGRAM_ID);
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

  const discriminator = anchorDiscriminator("initialize_staking_pool");
  console.log("initialize_staking_pool discriminator hex:", discriminator.toString("hex"));

  requireEnv(["ALPHA_MINT", "USDC_MINT"]);

  const alphaMint = readRequiredPublicKey("ALPHA_MINT");
  const usdcMint = readRequiredPublicKey("USDC_MINT");
  const minClaimUsdc = readU64("MIN_CLAIM_USDC", DEFAULT_MIN_CLAIM_USDC);

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const authority = provider.wallet.publicKey;
  const stakingPool = derivePda(SEEDS.stakingPoolV1);
  const alphaStakingVault = derivePda(SEEDS.alphaStakingVault);
  const alphaVaultAuthority = derivePda(SEEDS.alphaVaultAuthorityV1);
  const vaultAuthorityV2 = derivePda(SEEDS.vaultAuthorityV2);

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("Authority:", authority.toBase58());
  console.log("ALPHA Mint:", alphaMint.toBase58());
  console.log("USDC Mint:", usdcMint.toBase58());
  console.log("MIN_CLAIM_USDC:", minClaimUsdc.toString());
  console.log("staking_pool_v1:", stakingPool.toBase58());
  console.log("alpha_staking_vault:", alphaStakingVault.toBase58());
  console.log("alpha_vault_authority_v1:", alphaVaultAuthority.toBase58());
  console.log("staking_usdc_vault:", STAKING_USDC_VAULT.toBase58());
  console.log("vault_authority_v2:", vaultAuthorityV2.toBase58());

  const ix = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: stakingPool, isSigner: false, isWritable: true },
      { pubkey: alphaMint, isSigner: false, isWritable: false },
      { pubkey: usdcMint, isSigner: false, isWritable: false },
      { pubkey: alphaVaultAuthority, isSigner: false, isWritable: false },
      { pubkey: alphaStakingVault, isSigner: false, isWritable: true },
      { pubkey: vaultAuthorityV2, isSigner: false, isWritable: false },
      { pubkey: STAKING_USDC_VAULT, isSigner: false, isWritable: false },
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ],
    data: Buffer.concat([discriminator, u64Buffer(minClaimUsdc)]),
  });

  const signature = await provider.sendAndConfirm(new Transaction().add(ix), []);
  console.log("Transaction signature:", signature);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
