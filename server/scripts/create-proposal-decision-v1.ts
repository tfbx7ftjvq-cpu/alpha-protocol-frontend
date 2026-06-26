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
const U64_MAX = (1n << 64n) - 1n;
const I64_MIN = -(1n << 63n);
const I64_MAX = (1n << 63n) - 1n;
const PROPOSAL_TYPES = [
  "GreenLabelSlash",
  "GreenLabelRefund",
  "PayrollEmployeeImpeach",
  "PayrollPayout",
  "TreasuryParamChange",
  "EmergencyPause",
] as const;
const PROPOSAL_DECISIONS = ["Pending", "Approved", "Rejected", "Partial"] as const;

function loadIdl(): RuntimeIdl {
  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf8")) as RuntimeIdl;
  idl.address = PROGRAM_ID.toBase58();
  return idl;
}

function anchorDiscriminator(name: string): Buffer {
  return crypto.createHash("sha256").update(`global:${name}`).digest().subarray(0, 8);
}

function readU64(name: string, defaultValue: string): bigint {
  const raw = process.env[name] ?? defaultValue;
  if (!/^\d+$/.test(raw)) {
    throw new Error(`${name} must be a non-negative integer. Received: ${raw}`);
  }

  const value = BigInt(raw);
  if (value > U64_MAX) {
    throw new Error(`${name} must fit in an unsigned 64-bit integer.`);
  }

  return value;
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

function readEnum(
  envName: string,
  defaultValue: string,
  variants: readonly string[],
): { name: string; index: number } {
  const raw = process.env[envName] ?? defaultValue;
  const index = variants.findIndex((variant) => variant.toLowerCase() === raw.toLowerCase());
  if (index < 0) {
    throw new Error(`${envName} must be one of: ${variants.join(", ")}. Received: ${raw}`);
  }

  return { name: variants[index], index };
}

function u64Buffer(value: bigint): Buffer {
  const buffer = Buffer.alloc(8);
  buffer.writeBigUInt64LE(value);
  return buffer;
}

function i64Buffer(value: bigint): Buffer {
  const buffer = Buffer.alloc(8);
  buffer.writeBigInt64LE(value);
  return buffer;
}

function deriveGovernanceConfig(): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from("governance_config_v1")], PROGRAM_ID)[0];
}

function deriveProposalDecision(proposalId: bigint): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("proposal_decision_v1"), u64Buffer(proposalId)],
    PROGRAM_ID,
  )[0];
}

async function main(): Promise<void> {
  const idl = loadIdl();
  console.log("Runtime IDL instruction names:", idl.instructions.map((ix) => ix.name));

  const discriminator = anchorDiscriminator("create_proposal_decision");
  console.log("create_proposal_decision discriminator hex:", discriminator.toString("hex"));

  const now = BigInt(Math.floor(Date.now() / 1000));
  const proposalId = readU64("PROPOSAL_ID", "1");
  const proposalType = readEnum("PROPOSAL_TYPE", "TreasuryParamChange", PROPOSAL_TYPES);
  const decision = readEnum("DECISION", "Approved", PROPOSAL_DECISIONS);
  const yesWeight = readU64("YES_WEIGHT", "100");
  const noWeight = readU64("NO_WEIGHT", "0");
  const startTs = readI64("START_TS", now.toString());
  const endTs = readI64("END_TS", (startTs + 60n).toString());

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const authority = provider.wallet.publicKey;
  const governanceConfig = deriveGovernanceConfig();
  const proposalDecision = deriveProposalDecision(proposalId);

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("authority:", authority.toBase58());
  console.log("governance_config_v1:", governanceConfig.toBase58());
  console.log("proposal_decision_v1:", proposalDecision.toBase58());
  console.log("proposal_id:", proposalId.toString());
  console.log("proposal_type:", proposalType.name);
  console.log("decision:", decision.name);
  console.log("yes_weight:", yesWeight.toString());
  console.log("no_weight:", noWeight.toString());
  console.log("start_ts:", startTs.toString());
  console.log("end_ts:", endTs.toString());

  const data = Buffer.concat([
    discriminator,
    u64Buffer(proposalId),
    Buffer.from([proposalType.index]),
    Buffer.from([decision.index]),
    u64Buffer(yesWeight),
    u64Buffer(noWeight),
    i64Buffer(startTs),
    i64Buffer(endTs),
  ]);

  const instruction = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: governanceConfig, isSigner: false, isWritable: true },
      { pubkey: proposalDecision, isSigner: false, isWritable: true },
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
