import * as anchor from "@coral-xyz/anchor";
import { BN, Program, type Idl } from "@coral-xyz/anchor";
import { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";
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

type IdlWithAddress = Idl & {
  address: string;
};

type DepositUsdcRevenueMethod = (amount: BN) => {
  accountsStrict(accounts: {
    depositor: PublicKey;
    depositorUsdcTokenAccount: PublicKey;
    treasuryConfig: PublicKey;
    treasuryUsdcState: PublicKey;
    usdcMint: PublicKey;
    vaultAuthority: PublicKey;
    reliefUsdcVault: PublicKey;
    buybackUsdcVault: PublicKey;
    buildersUsdcVault: PublicKey;
    stakingUsdcVault: PublicKey;
    tokenProgram: PublicKey;
  }): {
    rpc(): Promise<string>;
  };
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

function loadIdl(): IdlWithAddress {
  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf8")) as IdlWithAddress;

  if (idl.address && idl.address !== PROGRAM_ID.toBase58()) {
    throw new Error(
      `IDL program ID mismatch. Expected ${PROGRAM_ID.toBase58()}, got ${idl.address}`,
    );
  }

  idl.address = PROGRAM_ID.toBase58();
  return idl;
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

  const usdcMint = readRequiredPublicKey("USDC_MINT");
  const amount = readAmount();
  const programId = PROGRAM_ID;

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = new Program(idl as Idl, provider);
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

  const methods = program.methods as Record<string, unknown>;
  const depositUsdcRevenue =
    typeof methods.depositUsdcRevenue === "function"
      ? (methods.depositUsdcRevenue as DepositUsdcRevenueMethod)
      : typeof methods.deposit_usdc_revenue === "function"
        ? (methods.deposit_usdc_revenue as DepositUsdcRevenueMethod)
        : undefined;

  if (!depositUsdcRevenue) {
    throw new Error("deposit_usdc_revenue method not found in program.methods");
  }

  const signature = await depositUsdcRevenue(new BN(amount.toString(), 10))
    .accountsStrict({
      depositor,
      depositorUsdcTokenAccount,
      treasuryConfig,
      treasuryUsdcState,
      usdcMint,
      vaultAuthority,
      reliefUsdcVault,
      buybackUsdcVault,
      buildersUsdcVault,
      stakingUsdcVault,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();

  console.log("Transaction signature:", signature);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
