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

async function main(): Promise<void> {
  const idl = loadIdl();
  console.log(
    "Runtime IDL instruction names:",
    idl.instructions.map((ix) => ix.name),
  );

  const discriminator = anchorDiscriminator("initialize_usdc_treasury");
  console.log("initialize_usdc_treasury discriminator hex:", discriminator.toString("hex"));

  requireEnv(["USDC_MINT", "ALPHA_MINT"]);

  const usdcMint = readRequiredPublicKey("USDC_MINT");
  const alphaMint = readRequiredPublicKey("ALPHA_MINT");
  const programId = PROGRAM_ID;

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

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

  const data = Buffer.concat([discriminator, usdcMint.toBuffer(), alphaMint.toBuffer()]);
  const ix = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: treasuryConfig, isSigner: false, isWritable: true },
      { pubkey: treasuryUsdcState, isSigner: false, isWritable: true },
      { pubkey: usdcMint, isSigner: false, isWritable: false },
      { pubkey: vaultAuthority, isSigner: false, isWritable: false },
      { pubkey: reliefUsdcVault, isSigner: false, isWritable: true },
      { pubkey: buybackUsdcVault, isSigner: false, isWritable: true },
      { pubkey: buildersUsdcVault, isSigner: false, isWritable: true },
      { pubkey: stakingUsdcVault, isSigner: false, isWritable: true },
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ],
    data,
  });

  const signature = await provider.sendAndConfirm(new Transaction().add(ix), []);

  console.log("Transaction signature:", signature);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
