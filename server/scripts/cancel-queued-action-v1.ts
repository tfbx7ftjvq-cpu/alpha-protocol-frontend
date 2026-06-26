import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
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

function loadIdl(): RuntimeIdl {
  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf8")) as RuntimeIdl;
  idl.address = PROGRAM_ID.toBase58();
  return idl;
}

function anchorDiscriminator(name: string): Buffer {
  return crypto.createHash("sha256").update(`global:${name}`).digest().subarray(0, 8);
}

function readRequiredU64(name: string): bigint {
  const raw = process.env[name];
  if (!raw) {
    throw new Error(`Missing required environment variable: ${name}`);
  }
  if (!/^\d+$/.test(raw)) {
    throw new Error(`${name} must be a non-negative integer. Received: ${raw}`);
  }

  const value = BigInt(raw);
  if (value > U64_MAX) {
    throw new Error(`${name} must fit in an unsigned 64-bit integer.`);
  }

  return value;
}

function u64Buffer(value: bigint): Buffer {
  const buffer = Buffer.alloc(8);
  buffer.writeBigUInt64LE(value);
  return buffer;
}

function deriveGovernanceConfig(): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from("governance_config_v1")], PROGRAM_ID)[0];
}

function deriveExecutionQueueItem(proposalId: bigint): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("execution_queue_item_v1"), u64Buffer(proposalId)],
    PROGRAM_ID,
  )[0];
}

async function main(): Promise<void> {
  const idl = loadIdl();
  console.log("Runtime IDL instruction names:", idl.instructions.map((ix) => ix.name));

  const discriminator = anchorDiscriminator("cancel_queued_action");
  console.log("cancel_queued_action discriminator hex:", discriminator.toString("hex"));

  const proposalId = readRequiredU64("PROPOSAL_ID");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const authorityOrGuardian = provider.wallet.publicKey;
  const governanceConfig = deriveGovernanceConfig();
  const executionQueueItem = deriveExecutionQueueItem(proposalId);

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("authority_or_guardian:", authorityOrGuardian.toBase58());
  console.log("governance_config_v1:", governanceConfig.toBase58());
  console.log("execution_queue_item_v1:", executionQueueItem.toBase58());
  console.log("proposal_id:", proposalId.toString());

  const data = Buffer.concat([discriminator, u64Buffer(proposalId)]);

  const instruction = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: governanceConfig, isSigner: false, isWritable: false },
      { pubkey: executionQueueItem, isSigner: false, isWritable: true },
      { pubkey: authorityOrGuardian, isSigner: true, isWritable: false },
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
