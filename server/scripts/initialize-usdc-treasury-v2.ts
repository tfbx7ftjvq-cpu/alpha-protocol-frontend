import * as anchor from "@coral-xyz/anchor";
import { Program, type Idl } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey, SYSVAR_RENT_PUBKEY, SystemProgram } from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";

const IDL_PATH = path.resolve(__dirname, "../target/idl/my_first_solana_program.json");
const PROGRAM_ID = new PublicKey("HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY");

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

type InitializeUsdcTreasuryMethod = (
  usdcMint: PublicKey,
  alphaMint: PublicKey,
) => {
  accountsStrict(accounts: {
    treasuryConfig: PublicKey;
    treasuryUsdcState: PublicKey;
    usdcMintAccount: PublicKey;
    vaultAuthority: PublicKey;
    reliefUsdcVault: PublicKey;
    buybackUsdcVault: PublicKey;
    buildersUsdcVault: PublicKey;
    stakingUsdcVault: PublicKey;
    authority: PublicKey;
    tokenProgram: PublicKey;
    systemProgram: PublicKey;
    rent: PublicKey;
  }): {
    rpc(): Promise<string>;
  };
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

async function main(): Promise<void> {
  const idl = loadIdl();
  console.log(
    "Runtime IDL instruction names:",
    idl.instructions.map((ix) => ix.name),
  );

  requireEnv(["USDC_MINT", "ALPHA_MINT"]);

  const usdcMint = readRequiredPublicKey("USDC_MINT");
  const alphaMint = readRequiredPublicKey("ALPHA_MINT");
  const programId = PROGRAM_ID;

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = new Program(idl as Idl, provider);
  const authority = provider.wallet.publicKey;

  const treasuryConfig = derivePda(SEEDS.treasuryConfigV2, programId);
  const treasuryUsdcState = derivePda(SEEDS.treasuryUsdcStateV2, programId);
  const reliefUsdcVault = derivePda(SEEDS.reliefUsdcVault, programId);
  const buybackUsdcVault = derivePda(SEEDS.buybackUsdcVault, programId);
  const buildersUsdcVault = derivePda(SEEDS.buildersUsdcVault, programId);
  const stakingUsdcVault = derivePda(SEEDS.stakingUsdcVault, programId);
  const vaultAuthority = derivePda(SEEDS.vaultAuthorityV2, programId);

  console.log("Program ID:", programId.toBase58());
  console.log("Authority:", authority.toBase58());
  console.log("USDC Mint:", usdcMint.toBase58());
  console.log("ALPHA Mint:", alphaMint.toBase58());
  console.log("treasury_config_v2:", treasuryConfig.toBase58());
  console.log("treasury_usdc_state_v2:", treasuryUsdcState.toBase58());
  console.log("relief_usdc_vault:", reliefUsdcVault.toBase58());
  console.log("buyback_usdc_vault:", buybackUsdcVault.toBase58());
  console.log("builders_usdc_vault:", buildersUsdcVault.toBase58());
  console.log("staking_usdc_vault:", stakingUsdcVault.toBase58());
  console.log("vault_authority_v2:", vaultAuthority.toBase58());

  const methods = program.methods as Record<string, unknown>;
  const initializeUsdcTreasury =
    typeof methods.initializeUsdcTreasury === "function"
      ? (methods.initializeUsdcTreasury as InitializeUsdcTreasuryMethod)
      : typeof methods.initialize_usdc_treasury === "function"
        ? (methods.initialize_usdc_treasury as InitializeUsdcTreasuryMethod)
        : undefined;

  if (!initializeUsdcTreasury) {
    throw new Error("initialize_usdc_treasury method not found in program.methods");
  }

  const signature = await initializeUsdcTreasury(usdcMint, alphaMint)
    .accountsStrict({
      treasuryConfig,
      treasuryUsdcState,
      usdcMintAccount: usdcMint,
      vaultAuthority,
      reliefUsdcVault,
      buybackUsdcVault,
      buildersUsdcVault,
      stakingUsdcVault,
      authority,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
    })
    .rpc();

  console.log("Transaction signature:", signature);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
