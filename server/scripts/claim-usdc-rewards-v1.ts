import * as anchor from "@coral-xyz/anchor";
import { createAssociatedTokenAccountInstruction, getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
import crypto from "crypto";
import fs from "fs";
import path from "path";

type RuntimeIdl = {
  address?: string;
  instructions: Array<{ name: string }>;
};

const IDL_PATH = path.resolve(__dirname, "../target/idl/my_first_solana_program.json");
const PROGRAM_ID = new PublicKey("HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY");
const STAKING_USDC_VAULT = new PublicKey("9nAUb7QG3mALgEUQZ26fHRa4p9BkfvKV5xGp6NFXA8wQ");

const SEEDS = {
  stakingPool: Buffer.from("staking_pool_v1"),
  userStakeAccount: Buffer.from("user_stake_account"),
  vaultAuthorityV2: Buffer.from("vault_authority_v2"),
};

function loadRuntimeIdl(): RuntimeIdl {
  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf8")) as RuntimeIdl;
  idl.address = PROGRAM_ID.toBase58();
  return idl;
}

function anchorDiscriminator(name: string): Buffer {
  return crypto.createHash("sha256").update(`global:${name}`).digest().subarray(0, 8);
}

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
    throw new Error(`Invalid public key for ${name}: ${value}`);
  }
}

function derivePda(seed: Buffer): PublicKey {
  return PublicKey.findProgramAddressSync([seed], PROGRAM_ID)[0];
}

function deriveUserStakeAccount(owner: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync([SEEDS.userStakeAccount, owner.toBuffer()], PROGRAM_ID)[0];
}

async function main(): Promise<void> {
  const idl = loadRuntimeIdl();
  console.log("Runtime IDL instruction names:", idl.instructions.map((ix) => ix.name));

  const discriminator = anchorDiscriminator("claim_usdc_rewards");
  console.log("claim_usdc_rewards discriminator hex:", discriminator.toString("hex"));

  requireEnv(["ALPHA_MINT", "USDC_MINT"]);

  const alphaMint = readRequiredPublicKey("ALPHA_MINT");
  const usdcMint = readRequiredPublicKey("USDC_MINT");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const owner = provider.wallet.publicKey;
  const stakingPool = derivePda(SEEDS.stakingPool);
  const userStakeAccount = deriveUserStakeAccount(owner);
  const vaultAuthorityV2 = derivePda(SEEDS.vaultAuthorityV2);
  const ownerUsdcTokenAccount = getAssociatedTokenAddressSync(usdcMint, owner);

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("Owner:", owner.toBase58());
  console.log("ALPHA Mint:", alphaMint.toBase58());
  console.log("USDC Mint:", usdcMint.toBase58());
  console.log("staking_pool_v1:", stakingPool.toBase58());
  console.log("user_stake_account:", userStakeAccount.toBase58());
  console.log("staking_usdc_vault:", STAKING_USDC_VAULT.toBase58());
  console.log("vault_authority_v2:", vaultAuthorityV2.toBase58());
  console.log("owner_usdc_ata:", ownerUsdcTokenAccount.toBase58());

  const transaction = new Transaction();
  const ownerUsdcAtaInfo = await provider.connection.getAccountInfo(ownerUsdcTokenAccount);
  console.log("owner_usdc_ata exists:", ownerUsdcAtaInfo !== null);

  if (!ownerUsdcAtaInfo) {
    transaction.add(
      createAssociatedTokenAccountInstruction(owner, ownerUsdcTokenAccount, owner, usdcMint)
    );
  }

  transaction.add(
    new TransactionInstruction({
      programId: PROGRAM_ID,
      keys: [
        { pubkey: stakingPool, isSigner: false, isWritable: true },
        { pubkey: userStakeAccount, isSigner: false, isWritable: true },
        { pubkey: owner, isSigner: true, isWritable: false },
        { pubkey: STAKING_USDC_VAULT, isSigner: false, isWritable: true },
        { pubkey: vaultAuthorityV2, isSigner: false, isWritable: false },
        { pubkey: ownerUsdcTokenAccount, isSigner: false, isWritable: true },
        { pubkey: usdcMint, isSigner: false, isWritable: false },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      ],
      data: discriminator,
    })
  );

  const signature = await provider.sendAndConfirm(transaction, []);
  console.log("Transaction signature:", signature);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
