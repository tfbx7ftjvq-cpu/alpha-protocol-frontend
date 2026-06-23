import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
import { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
import * as crypto from "crypto";
import * as fs from "fs";
import * as path from "path";

const IDL_PATH = path.resolve(__dirname, "../target/idl/my_first_solana_program.json");
const DEFAULT_AMOUNT = "100000000";
const PROGRAM_ID = new PublicKey("HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY");
const U64_MAX = new BN("18446744073709551615", 10);

const SEEDS = {
  treasuryConfigV2: "treasury_config_v2",
  treasuryUsdcStateV2: "treasury_usdc_state_v2",
  reliefUsdcVault: "relief_usdc_vault",
  buybackUsdcVault: "buyback_usdc_vault",
  buildersUsdcVault: "builders_usdc_vault",
  stakingUsdcVault: "staking_usdc_vault",
  vaultAuthorityV2: "vault_authority_v2",
} as const;

type RuntimeIdl = {
  address?: string;
  instructions: Array<{
    name: string;
  }>;
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

function readAmount(): BN {
  const rawAmount = process.env.AMOUNT ?? DEFAULT_AMOUNT;

  if (!/^\d+$/.test(rawAmount)) {
    throw new Error("AMOUNT must be a positive integer string");
  }

  const amount = new BN(rawAmount, 10);
  if (amount.lte(new BN(0))) {
    throw new Error("AMOUNT must be greater than 0");
  }
  if (amount.gt(U64_MAX)) {
    throw new Error("AMOUNT must fit in an unsigned 64-bit integer");
  }

  return amount;
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

function derivePda(seed: string, programId: PublicKey): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync([Buffer.from(seed)], programId);
  return pda;
}

function splitAmount(amount: BN, percentage: number): BN {
  return amount.mul(new BN(percentage)).div(new BN(100));
}

async function main(): Promise<void> {
  const idl = loadIdl();
  console.log(
    "Runtime IDL instruction names:",
    idl.instructions.map((ix) => ix.name),
  );

  const discriminator = anchorDiscriminator("deposit_usdc_revenue");
  console.log("deposit_usdc_revenue discriminator hex:", discriminator.toString("hex"));

  const usdcMint = readRequiredPublicKey("USDC_MINT");
  const amount = readAmount();
  const programId = PROGRAM_ID;

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const depositor = provider.wallet.publicKey;
  const depositorUsdcTokenAccount = getAssociatedTokenAddressSync(usdcMint, depositor);

  const treasuryConfig = derivePda(SEEDS.treasuryConfigV2, programId);
  const treasuryUsdcState = derivePda(SEEDS.treasuryUsdcStateV2, programId);
  const reliefUsdcVault = derivePda(SEEDS.reliefUsdcVault, programId);
  const buybackUsdcVault = derivePda(SEEDS.buybackUsdcVault, programId);
  const buildersUsdcVault = derivePda(SEEDS.buildersUsdcVault, programId);
  const stakingUsdcVault = derivePda(SEEDS.stakingUsdcVault, programId);
  const vaultAuthority = derivePda(SEEDS.vaultAuthorityV2, programId);

  const relief = splitAmount(amount, 50);
  const buyback = splitAmount(amount, 20);
  const builders = splitAmount(amount, 20);
  const staking = splitAmount(amount, 10);

  console.log("Program ID:", programId.toBase58());
  console.log("Depositor:", depositor.toBase58());
  console.log("USDC Mint:", usdcMint.toBase58());
  console.log("depositor_usdc_ata:", depositorUsdcTokenAccount.toBase58());
  console.log("treasury_config_v2:", treasuryConfig.toBase58());
  console.log("treasury_usdc_state_v2:", treasuryUsdcState.toBase58());
  console.log("relief_usdc_vault:", reliefUsdcVault.toBase58());
  console.log("buyback_usdc_vault:", buybackUsdcVault.toBase58());
  console.log("builders_usdc_vault:", buildersUsdcVault.toBase58());
  console.log("staking_usdc_vault:", stakingUsdcVault.toBase58());
  console.log("vault_authority_v2:", vaultAuthority.toBase58());
  console.log("Amount:", amount.toString());
  console.log("Expected split:");
  console.log("relief = amount * 50 / 100 =", relief.toString());
  console.log("buyback = amount * 20 / 100 =", buyback.toString());
  console.log("builders = amount * 20 / 100 =", builders.toString());
  console.log("staking = amount * 10 / 100 =", staking.toString());

  const amountBuffer = Buffer.alloc(8);
  amountBuffer.writeBigUInt64LE(BigInt(amount.toString()));

  const ix = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: depositor, isSigner: true, isWritable: true },
      { pubkey: depositorUsdcTokenAccount, isSigner: false, isWritable: true },
      { pubkey: treasuryConfig, isSigner: false, isWritable: false },
      { pubkey: treasuryUsdcState, isSigner: false, isWritable: true },
      { pubkey: usdcMint, isSigner: false, isWritable: false },
      { pubkey: vaultAuthority, isSigner: false, isWritable: false },
      { pubkey: reliefUsdcVault, isSigner: false, isWritable: true },
      { pubkey: buybackUsdcVault, isSigner: false, isWritable: true },
      { pubkey: buildersUsdcVault, isSigner: false, isWritable: true },
      { pubkey: stakingUsdcVault, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.concat([discriminator, amountBuffer]),
  });

  const signature = await provider.sendAndConfirm(new Transaction().add(ix), []);

  console.log("Transaction signature:", signature);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
