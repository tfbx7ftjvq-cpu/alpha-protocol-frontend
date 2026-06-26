import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";
import * as crypto from "crypto";
import * as fs from "fs";
import * as path from "path";

type RuntimeIdl = {
  address?: string;
  instructions: Array<{ name: string }>;
};

const IDL_PATH = path.resolve(__dirname, "../target/idl/my_first_solana_program.json");
const PROGRAM_ID = new PublicKey("HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY");
const DEFAULT_MIN_EXECUTION_DELAY_SECONDS = "60";
const I64_MIN = -(1n << 63n);
const I64_MAX = (1n << 63n) - 1n;

function loadIdl(): RuntimeIdl {
  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf8")) as RuntimeIdl;
  idl.address = PROGRAM_ID.toBase58();
  return idl;
}

function anchorDiscriminator(name: string): Buffer {
  return crypto.createHash("sha256").update(`global:${name}`).digest().subarray(0, 8);
}

function readI64(name: string, defaultValue: string): bigint {
  const raw = process.env[name] ?? defaultValue;
  if (!/^-?\d+$/.test(raw)) {
    throw new Error(`${name} must be an integer. Received: ${raw}`);
  }

  const value = BigInt(raw);
  if (value < I64_MIN || value > I64_MAX) {
    throw new Error(`${name} must fit in a signed 64-bit integer.`);
  }

  return value;
}

function readOptionalPublicKey(name: string, defaultValue: PublicKey): PublicKey {
  const value = process.env[name];
  if (!value) {
    return defaultValue;
  }

  try {
    return new PublicKey(value);
  } catch {
    throw new Error(`Invalid public key in ${name}: ${value}`);
  }
}

function i64Buffer(value: bigint): Buffer {
  const buffer = Buffer.alloc(8);
  buffer.writeBigInt64LE(value);
  return buffer;
}

function deriveGovernanceConfig(): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from("governance_config_v1")], PROGRAM_ID)[0];
}

async function main(): Promise<void> {
  const idl = loadIdl();
  console.log("Runtime IDL instruction names:", idl.instructions.map((ix) => ix.name));

  const discriminator = anchorDiscriminator("initialize_governance_config");
  console.log("initialize_governance_config discriminator hex:", discriminator.toString("hex"));

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const authority = provider.wallet.publicKey;
  const minExecutionDelaySeconds = readI64(
    "MIN_EXECUTION_DELAY_SECONDS",
    DEFAULT_MIN_EXECUTION_DELAY_SECONDS,
  );
  const emergencyGuardian = readOptionalPublicKey("EMERGENCY_GUARDIAN", authority);
  const governanceConfig = deriveGovernanceConfig();

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("authority:", authority.toBase58());
  console.log("emergency_guardian:", emergencyGuardian.toBase58());
  console.log("min_execution_delay_seconds:", minExecutionDelaySeconds.toString());
  console.log("governance_config_v1:", governanceConfig.toBase58());

  const data = Buffer.concat([
    discriminator,
    i64Buffer(minExecutionDelaySeconds),
    emergencyGuardian.toBuffer(),
  ]);

  const instruction = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: governanceConfig, isSigner: false, isWritable: true },
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  });

  const signature = await provider.sendAndConfirm(new Transaction().add(instruction), []);
  console.log("Transaction signature:", signature);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
