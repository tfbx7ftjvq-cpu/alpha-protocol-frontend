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
const DEFAULT_PUBLIC_KEY = new PublicKey("11111111111111111111111111111111");
const U64_MAX = (1n << 64n) - 1n;
const ACTION_TYPES = [
  "Noop",
  "GreenLabelSlash",
  "GreenLabelRefund",
  "PayrollEmployeeImpeach",
  "PayrollPayout",
  "TreasuryParamChange",
  "EmergencyPause",
] as const;

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

function readPublicKey(name: string, defaultValue: PublicKey): PublicKey {
  const raw = process.env[name];
  if (!raw) {
    return defaultValue;
  }

  try {
    return new PublicKey(raw);
  } catch {
    throw new Error(`Invalid public key in ${name}: ${raw}`);
  }
}

function u64Buffer(value: bigint): Buffer {
  const buffer = Buffer.alloc(8);
  buffer.writeBigUInt64LE(value);
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

function deriveExecutionQueueItem(proposalId: bigint): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("execution_queue_item_v1"), u64Buffer(proposalId)],
    PROGRAM_ID,
  )[0];
}

function readExecuteAfter(data: Buffer): bigint | null {
  const executeAfterOffset = 122;
  if (data.length < executeAfterOffset + 8) {
    return null;
  }

  return data.readBigInt64LE(executeAfterOffset);
}

async function main(): Promise<void> {
  const idl = loadIdl();
  console.log("Runtime IDL instruction names:", idl.instructions.map((ix) => ix.name));

  const discriminator = anchorDiscriminator("queue_execution");
  console.log("queue_execution discriminator hex:", discriminator.toString("hex"));

  const proposalId = readU64("PROPOSAL_ID", "1");
  const actionType = readEnum("ACTION_TYPE", "Noop", ACTION_TYPES);
  const payloadText = process.env.PAYLOAD_TEXT ?? "alpha-security-v1-noop";
  const payloadHash = crypto.createHash("sha256").update(payloadText).digest();
  const targetProgram = readPublicKey("TARGET_PROGRAM", PROGRAM_ID);
  const targetAccount = readPublicKey("TARGET_ACCOUNT", DEFAULT_PUBLIC_KEY);

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const authority = provider.wallet.publicKey;
  const governanceConfig = deriveGovernanceConfig();
  const proposalDecision = deriveProposalDecision(proposalId);
  const executionQueueItem = deriveExecutionQueueItem(proposalId);

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("authority:", authority.toBase58());
  console.log("governance_config_v1:", governanceConfig.toBase58());
  console.log("proposal_decision_v1:", proposalDecision.toBase58());
  console.log("execution_queue_item_v1:", executionQueueItem.toBase58());
  console.log("proposal_id:", proposalId.toString());
  console.log("action_type:", actionType.name);
  console.log("target_program:", targetProgram.toBase58());
  console.log("target_account:", targetAccount.toBase58());
  console.log("payload_text:", payloadText);
  console.log("payload_hash hex:", payloadHash.toString("hex"));

  const data = Buffer.concat([
    discriminator,
    u64Buffer(proposalId),
    Buffer.from([actionType.index]),
    targetProgram.toBuffer(),
    targetAccount.toBuffer(),
    payloadHash,
  ]);

  const instruction = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: governanceConfig, isSigner: false, isWritable: false },
      { pubkey: proposalDecision, isSigner: false, isWritable: false },
      { pubkey: executionQueueItem, isSigner: false, isWritable: true },
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  });

  const signature = await provider.sendAndConfirm(new Transaction().add(instruction), []);
  console.log("Transaction signature:", signature);

  const accountInfo = await provider.connection.getAccountInfo(executionQueueItem);
  const executeAfter = accountInfo ? readExecuteAfter(accountInfo.data) : null;
  if (executeAfter !== null) {
    console.log("execute_after:", executeAfter.toString());
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
