import { Transaction } from "@solana/web3.js";
import {
  DEVNET_USDC_MINT,
  PROGRAM_ID,
  buildInitializeGreenLabelConfigIx,
  deriveGovernanceConfig,
  deriveGreenLabelConfig,
  deriveTreasuryPdas,
  fetchGreenLabelConfig,
  loadProvider,
  printDevnetRiskBanner,
  printGreenLabelConfig,
  readPublicKeyEnv,
  requireAccountExists,
  sendAndConfirmLabeled,
} from "./common";

async function main(): Promise<void> {
  printDevnetRiskBanner("setup-green-label-config");

  const provider = loadProvider();
  const authority = provider.wallet.publicKey;
  const greenLabelConfig = deriveGreenLabelConfig();
  const treasuryPdas = deriveTreasuryPdas();
  const governanceConfig = deriveGovernanceConfig();
  const usdcMint = readPublicKeyEnv("USDC_MINT", DEVNET_USDC_MINT);
  const baseBondTreasuryVault = readPublicKeyEnv(
    "BASE_BOND_TREASURY_VAULT",
    treasuryPdas.buildersUsdcVault,
  );
  const reliefOrRiskVault = readPublicKeyEnv("RELIEF_OR_RISK_VAULT", treasuryPdas.reliefUsdcVault);

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("authority:", authority.toBase58());
  console.log("green_label_config:", greenLabelConfig.toBase58());
  console.log("usdc_mint:", usdcMint.toBase58());
  console.log("treasury_usdc_state_v2:", treasuryPdas.treasuryUsdcStateV2.toBase58());
  console.log("base_bond_treasury_vault:", baseBondTreasuryVault.toBase58());
  console.log("relief_or_risk_vault:", reliefOrRiskVault.toBase58());
  console.log("vault_authority_v2:", treasuryPdas.vaultAuthorityV2.toBase58());
  console.log("security_governance_config:", governanceConfig.toBase58());

  const existingConfig = await fetchGreenLabelConfig(provider);
  if (existingConfig) {
    console.log("GreenLabelConfigV1 already exists. No initialization transaction will be sent.");
    printGreenLabelConfig(existingConfig);
    if (!existingConfig.usdcMint.equals(usdcMint)) {
      throw new Error(
        `Existing config usdc_mint mismatch. Expected ${usdcMint.toBase58()}, got ${existingConfig.usdcMint.toBase58()}`,
      );
    }
    if (!existingConfig.baseBondTreasuryVault.equals(baseBondTreasuryVault)) {
      throw new Error(
        `Existing config base_bond_treasury_vault mismatch. Expected ${baseBondTreasuryVault.toBase58()}, got ${existingConfig.baseBondTreasuryVault.toBase58()}`,
      );
    }
    if (!existingConfig.reliefOrRiskVault.equals(reliefOrRiskVault)) {
      throw new Error(
        `Existing config relief_or_risk_vault mismatch. Expected ${reliefOrRiskVault.toBase58()}, got ${existingConfig.reliefOrRiskVault.toBase58()}`,
      );
    }
    if (!existingConfig.securityGovernanceConfig.equals(governanceConfig)) {
      throw new Error(
        `Existing config security_governance_config mismatch. Expected ${governanceConfig.toBase58()}, got ${existingConfig.securityGovernanceConfig.toBase58()}`,
      );
    }
    return;
  }

  await requireAccountExists(provider, treasuryPdas.treasuryUsdcStateV2, "TreasuryUsdcStateV2");
  await requireAccountExists(provider, baseBondTreasuryVault, "base_bond_treasury_vault");
  await requireAccountExists(provider, reliefOrRiskVault, "relief_or_risk_vault");
  await requireAccountExists(provider, governanceConfig, "Security Layer GovernanceConfigV1");

  if (!process.env.BASE_BOND_TREASURY_VAULT) {
    console.log(
      "BASE_BOND_TREASURY_VAULT not set; defaulting to Treasury V2 builders_usdc_vault. Override the env var if governance selected a different treasury vault.",
    );
  }
  if (!process.env.RELIEF_OR_RISK_VAULT) {
    console.log(
      "RELIEF_OR_RISK_VAULT not set; defaulting to Treasury V2 relief_usdc_vault. Override the env var if governance selected a dedicated risk reserve vault.",
    );
  }

  const ix = buildInitializeGreenLabelConfigIx({
    authority,
    usdcMint,
    treasuryUsdcStateV2: treasuryPdas.treasuryUsdcStateV2,
    baseBondTreasuryVault,
    reliefOrRiskVault,
    vaultAuthorityV2: treasuryPdas.vaultAuthorityV2,
    securityGovernanceConfig: governanceConfig,
  });

  await sendAndConfirmLabeled(provider, "initialize_green_label_config", new Transaction().add(ix));
  const createdConfig = await fetchGreenLabelConfig(provider);
  if (!createdConfig) {
    throw new Error("GreenLabelConfigV1 was not found after initialization.");
  }
  printGreenLabelConfig(createdConfig);
}

main().catch((error) => {
  console.error("setup-green-label-config failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
