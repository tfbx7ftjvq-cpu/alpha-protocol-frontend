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

function loadIdl(): RuntimeIdl {
  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf8")) as RuntimeIdl;
  idl.address = PROGRAM_ID.toBase58();
  return idl;
}

function anchorDiscriminator(name: string): Buffer {
  return crypto.createHash("sha256").update(`global:${name}`).digest().subarray(0, 8);
}

function deriveGovernanceConfig(): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from("governance_config_v1")], PROGRAM_ID)[0];
}

async function main(): Promise<void> {
  const idl = loadIdl();
  console.log("Runtime IDL instruction names:", idl.instructions.map((ix) => ix.name));

  const discriminator = anchorDiscriminator("unpause_security_layer");
  console.log("unpause_security_layer discriminator hex:", discriminator.toString("hex"));

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const authority = provider.wallet.publicKey;
  const governanceConfig = deriveGovernanceConfig();

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("authority:", authority.toBase58());
  console.log("governance_config_v1:", governanceConfig.toBase58());

  const instruction = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: governanceConfig, isSigner: false, isWritable: true },
      { pubkey: authority, isSigner: true, isWritable: false },
    ],
    data: discriminator,
  });

  const signature = await provider.sendAndConfirm(new Transaction().add(instruction), []);
  console.log("Transaction signature:", signature);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
