import { PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
import {
  PROGRAM_ID,
  deriveGreenLabelConfig,
  explorerLink,
  fetchGreenLabelConfig,
  formatUsdc,
  instructionDiscriminator,
  isDryRun,
  loadProvider,
  readUsdcAmountEnv,
  sendAndConfirmLabeled,
  u64Buffer,
} from "./common";

function buildUpdateGreenLabelMinBaseBondIx(args: {
  authority: PublicKey;
  minBaseBondUsdc: bigint;
}): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: deriveGreenLabelConfig(), isSigner: false, isWritable: true },
      { pubkey: args.authority, isSigner: true, isWritable: false },
    ],
    data: Buffer.concat([
      instructionDiscriminator("update_green_label_min_base_bond"),
      u64Buffer(args.minBaseBondUsdc),
    ]),
  });
}

async function main(): Promise<void> {
  console.log("Green Label V1 Devnet script: update-green-label-min-base-bond");
  console.log("WARNING: This script is for Devnet only.");
  console.log("WARNING: It only updates GreenLabelConfigV1 min_base_bond_usdc.");
  console.log("WARNING: It does not mint USDC, transfer tokens, or run refund/slash.");
  console.log("DRY_RUN:", String(isDryRun()));

  const provider = loadProvider();
  const authority = provider.wallet.publicKey;
  const config = await fetchGreenLabelConfig(provider);
  if (!config) {
    throw new Error("GreenLabelConfigV1 is not initialized. Run devnet:green-label:setup first.");
  }

  const minBaseBondUsdc = readUsdcAmountEnv("MIN_BASE_BOND_USDC", "1");

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("authority:", authority.toBase58());
  console.log("green_label_config:", deriveGreenLabelConfig().toBase58());
  console.log("old min_base_bond_usdc:", formatUsdc(config.minBaseBondUsdc));
  console.log("new min_base_bond_usdc:", formatUsdc(minBaseBondUsdc));

  if (config.minBaseBondUsdc === minBaseBondUsdc) {
    console.log("Green Label min_base_bond_usdc already matches target value. No transaction will be sent.");
    return;
  }

  const ix = buildUpdateGreenLabelMinBaseBondIx({
    authority,
    minBaseBondUsdc,
  });
  const signature = await sendAndConfirmLabeled(
    provider,
    "update_green_label_min_base_bond",
    new Transaction().add(ix),
  );
  console.log("update_green_label_min_base_bond transaction signature:", signature);
  console.log("update_green_label_min_base_bond explorer link:", explorerLink(signature));
}

main().catch((error) => {
  console.error("update-green-label-min-base-bond failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
