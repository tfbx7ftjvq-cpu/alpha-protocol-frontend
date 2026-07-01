import { PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
import {
  PROGRAM_ID,
  deriveGreenLabelConfig,
  explorerLink,
  fetchGreenLabelConfig,
  i64Buffer,
  instructionDiscriminator,
  isDryRun,
  loadProvider,
  sendAndConfirmLabeled,
} from "./common";

function readWindowSecondsEnv(name: string, defaultValue: bigint): bigint {
  const raw = process.env[name] ?? defaultValue.toString();
  if (!/^\d+$/.test(raw)) {
    throw new Error(`${name} must be a positive integer number of seconds. Received: ${raw}`);
  }

  const value = BigInt(raw);
  if (value <= 0n) {
    throw new Error(`${name} must be greater than zero. Received: ${raw}`);
  }

  return value;
}

function buildUpdateGreenLabelWindowsIx(args: {
  authority: PublicKey;
  observationPeriodSeconds: bigint;
  disputeWindowSeconds: bigint;
  responseWindowSeconds: bigint;
}): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: deriveGreenLabelConfig(), isSigner: false, isWritable: true },
      { pubkey: args.authority, isSigner: true, isWritable: false },
    ],
    data: Buffer.concat([
      instructionDiscriminator("update_green_label_windows"),
      i64Buffer(args.observationPeriodSeconds),
      i64Buffer(args.disputeWindowSeconds),
      i64Buffer(args.responseWindowSeconds),
    ]),
  });
}

async function main(): Promise<void> {
  console.log("Green Label V1 Devnet script: update-green-label-windows");
  console.log("WARNING: This script is for Devnet only.");
  console.log("WARNING: It only updates GreenLabelConfigV1 time windows.");
  console.log("WARNING: It does not move funds, modify projects/disputes, or run refund/slash.");
  console.log("DRY_RUN:", String(isDryRun()));

  const provider = loadProvider();
  const authority = provider.wallet.publicKey;
  const config = await fetchGreenLabelConfig(provider);
  if (!config) {
    throw new Error("GreenLabelConfigV1 is not initialized. Run devnet:green-label:setup first.");
  }

  const observationPeriodSeconds = readWindowSecondsEnv("OBSERVATION_SECONDS", 60n);
  const disputeWindowSeconds = readWindowSecondsEnv("DISPUTE_SECONDS", 60n);
  const responseWindowSeconds = readWindowSecondsEnv("RESPONSE_SECONDS", 60n);

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("authority:", authority.toBase58());
  console.log("green_label_config:", deriveGreenLabelConfig().toBase58());
  console.log("old observation_period_seconds:", config.observationPeriodSeconds.toString());
  console.log("old dispute_window_seconds:", config.disputeWindowSeconds.toString());
  console.log("old response_window_seconds:", config.responseWindowSeconds.toString());
  console.log("new observation_period_seconds:", observationPeriodSeconds.toString());
  console.log("new dispute_window_seconds:", disputeWindowSeconds.toString());
  console.log("new response_window_seconds:", responseWindowSeconds.toString());

  const alreadyUpdated =
    config.observationPeriodSeconds === observationPeriodSeconds &&
    config.disputeWindowSeconds === disputeWindowSeconds &&
    config.responseWindowSeconds === responseWindowSeconds;

  if (alreadyUpdated) {
    console.log("Green Label windows already match target values. No transaction will be sent.");
    return;
  }

  const ix = buildUpdateGreenLabelWindowsIx({
    authority,
    observationPeriodSeconds,
    disputeWindowSeconds,
    responseWindowSeconds,
  });
  const signature = await sendAndConfirmLabeled(
    provider,
    "update_green_label_windows",
    new Transaction().add(ix),
  );
  console.log("update_green_label_windows transaction signature:", signature);
  console.log("update_green_label_windows explorer link:", explorerLink(signature));
}

main().catch((error) => {
  console.error("update-green-label-windows failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
