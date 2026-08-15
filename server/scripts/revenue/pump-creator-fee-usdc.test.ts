import assert from "assert";
import { Keypair } from "@solana/web3.js";
import { MAINNET_USDC_MINT, buildPumpCreatorFeeManifest } from "./pump-creator-fee-usdc";

const keys = Array.from({ length: 6 }, () => Keypair.generate().publicKey.toBase58());
const input = { tokenMint: keys[0], revenueWallet: keys[1], settlementMint: MAINNET_USDC_MINT, usdcMint: MAINNET_USDC_MINT, amountBaseUnits: "101", sourceEvidenceReference: "pump-ui-receipt-reference", reliefUsdcVault: keys[2], buybackUsdcVault: keys[3], buildersUsdcVault: keys[4], stakingUsdcVault: keys[5] };
const manifest = buildPumpCreatorFeeManifest(input, {});
assert.deepStrictEqual(manifest.expectedSplit, { relief: "50", buyback: "20", builders: "20", staking: "11" });
assert.match(manifest.batchHash, /^[a-f0-9]{64}$/);
assert.throws(() => buildPumpCreatorFeeManifest({ ...input, settlementMint: keys[0] }, {}), /USDC settlement/);
assert.throws(() => buildPumpCreatorFeeManifest({ ...input, reliefUsdcVault: keys[3] }, {}), /four distinct/);
assert.throws(() => buildPumpCreatorFeeManifest(input, { VITE_REVENUE_WALLET: keys[1] }), /must not come from VITE/);
assert.throws(() => buildPumpCreatorFeeManifest({ ...input, sourceEvidenceReference: "jwt=not-allowed" }, {}), /non-secret/);
console.log("pump creator fee manifest tests passed");
