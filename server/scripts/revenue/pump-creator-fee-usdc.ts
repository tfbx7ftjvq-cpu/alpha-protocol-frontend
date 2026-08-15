import * as crypto from "crypto";
import { PublicKey } from "@solana/web3.js";

export const MAINNET_USDC_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
export const USDC_DECIMALS = 6;

export type PumpCreatorFeeInput = {
  tokenMint: string;
  revenueWallet: string;
  settlementMint: string;
  usdcMint: string;
  amountBaseUnits: string;
  sourceEvidenceReference: string;
  reliefUsdcVault: string;
  buybackUsdcVault: string;
  buildersUsdcVault: string;
  stakingUsdcVault: string;
};

export type PumpCreatorFeeManifest = {
  schemaVersion: "pump-creator-fee-usdc-v1";
  source: "pump_creator_fee";
  tokenMint: string;
  revenueWallet: string;
  settlementMint: string;
  usdcMint: string;
  usdcDecimals: 6;
  amountBaseUnits: string;
  revenueType: "PlatformRevenue";
  expectedSplit: { relief: string; buyback: string; builders: string; staking: string };
  destinationVaults: { relief: string; buyback: string; builders: string; staking: string };
  sourceEvidenceReference: string;
  batchHash: string;
};

function publicKey(value: string, label: string): string {
  try { return new PublicKey(value).toBase58(); } catch { throw new Error(`${label} must be a Solana public key`); }
}

function nonSecretText(value: string, label: string): string {
  if (!value || value.length > 200 || /[\r\n]/.test(value) || /(secret|private[_ -]?key|jwt|token=|service[_ -]?role)/i.test(value)) {
    throw new Error(`${label} must be a short non-secret reference`);
  }
  return value;
}

export function buildPumpCreatorFeeManifest(input: PumpCreatorFeeInput, environment: NodeJS.ProcessEnv = process.env): PumpCreatorFeeManifest {
  if (Object.keys(environment).some((name) => name.startsWith("VITE_") && /(revenue|wallet)/i.test(name) && environment[name])) {
    throw new Error("revenue wallet configuration must not come from VITE environment variables");
  }
  const tokenMint = publicKey(input.tokenMint, "token mint");
  const revenueWallet = publicKey(input.revenueWallet, "revenue wallet");
  const settlementMint = publicKey(input.settlementMint, "settlement mint");
  const usdcMint = publicKey(input.usdcMint, "USDC mint");
  if (usdcMint !== MAINNET_USDC_MINT || settlementMint !== MAINNET_USDC_MINT) throw new Error("only canonical Mainnet USDC settlement is eligible for routing");
  if (!/^[0-9]+$/.test(input.amountBaseUnits) || BigInt(input.amountBaseUnits) <= 0n) throw new Error("amount base units must be a positive integer");
  const amount = BigInt(input.amountBaseUnits);
  const relief = (amount * 50n) / 100n;
  const buyback = (amount * 20n) / 100n;
  const builders = (amount * 20n) / 100n;
  const staking = amount - relief - buyback - builders;
  const destinationVaults = {
    relief: publicKey(input.reliefUsdcVault, "relief vault"), buyback: publicKey(input.buybackUsdcVault, "buyback vault"),
    builders: publicKey(input.buildersUsdcVault, "builders vault"), staking: publicKey(input.stakingUsdcVault, "staking vault"),
  };
  if (new Set(Object.values(destinationVaults)).size !== 4) throw new Error("four distinct USDC destination vaults are required");
  if (Object.values(destinationVaults).includes(revenueWallet)) throw new Error("revenue wallet cannot be a destination vault");
  const payload = {
    schemaVersion: "pump-creator-fee-usdc-v1" as const, source: "pump_creator_fee" as const, tokenMint, revenueWallet, settlementMint, usdcMint,
    usdcDecimals: 6 as const, amountBaseUnits: amount.toString(), revenueType: "PlatformRevenue" as const,
    expectedSplit: { relief: relief.toString(), buyback: buyback.toString(), builders: builders.toString(), staking: staking.toString() },
    destinationVaults, sourceEvidenceReference: nonSecretText(input.sourceEvidenceReference, "source evidence reference"),
  };
  return { ...payload, batchHash: crypto.createHash("sha256").update(JSON.stringify(payload)).digest("hex") };
}

function readArgs(argv: string[]): PumpCreatorFeeInput {
  if (argv[0] !== "inspect" && argv[0] !== "prepare") throw new Error("use inspect or prepare; no routing operation exists");
  const values = new Map<string, string>();
  for (let index = 1; index < argv.length; index += 2) {
    const flag = argv[index]; const value = argv[index + 1];
    if (!flag?.startsWith("--") || value === undefined || value.startsWith("--")) throw new Error("arguments must use --name value pairs");
    values.set(flag.slice(2), value);
  }
  const keys = ["token-mint", "revenue-wallet", "settlement-mint", "usdc-mint", "amount-base-units", "source-evidence-reference", "relief-usdc-vault", "buyback-usdc-vault", "builders-usdc-vault", "staking-usdc-vault"];
  for (const key of keys) if (!values.has(key)) throw new Error(`missing --${key}`);
  return { tokenMint: values.get("token-mint")!, revenueWallet: values.get("revenue-wallet")!, settlementMint: values.get("settlement-mint")!, usdcMint: values.get("usdc-mint")!, amountBaseUnits: values.get("amount-base-units")!, sourceEvidenceReference: values.get("source-evidence-reference")!, reliefUsdcVault: values.get("relief-usdc-vault")!, buybackUsdcVault: values.get("buyback-usdc-vault")!, buildersUsdcVault: values.get("builders-usdc-vault")!, stakingUsdcVault: values.get("staking-usdc-vault")! };
}

if (require.main === module) {
  try { console.log(JSON.stringify(buildPumpCreatorFeeManifest(readArgs(process.argv.slice(2))))); }
  catch (error) { console.error(error instanceof Error ? error.message : "manifest validation failed"); process.exitCode = 1; }
}
