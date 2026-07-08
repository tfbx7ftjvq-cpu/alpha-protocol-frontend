import { Connection } from "@solana/web3.js";
import {
  addFail,
  checkAuthorityPolicy,
  checkClusterEndpoint,
  checkGreenLabelConfigAccount,
  checkLocalIdl,
  checkParameterPolicy,
  checkProgramAccount,
  checkSecurityGovernanceConfig,
  checkTokenVault,
  createSummary,
  printEnvironment,
  printSummary,
  resolveRuntimeConfig,
} from "./common";

async function main(): Promise<void> {
  const summary = createSummary();
  const runtime = resolveRuntimeConfig(summary);

  printEnvironment(runtime);
  checkClusterEndpoint(runtime, summary);
  checkLocalIdl(runtime.programId, summary);

  if (!runtime.rpcUrl) {
    printSummary(summary);
    process.exitCode = summary.fail.length > 0 ? 1 : 0;
    return;
  }

  const connection = new Connection(runtime.rpcUrl, "confirmed");

  await checkProgramAccount(connection, runtime.programId, summary);

  const greenLabelConfig = await checkGreenLabelConfigAccount(
    connection,
    runtime.greenLabelConfig,
    runtime.programId,
    summary,
  );

  if (!greenLabelConfig) {
    addFail(summary, "GreenLabelConfigV1 decode unavailable; dependent checks skipped");
    printSummary(summary);
    process.exitCode = 1;
    return;
  }

  checkParameterPolicy(greenLabelConfig, runtime.expectedMode, summary);
  await checkTokenVault(
    connection,
    "base_bond_treasury_vault",
    greenLabelConfig.baseBondTreasuryVault,
    greenLabelConfig.usdcMint,
    summary,
  );
  await checkTokenVault(
    connection,
    "relief_or_risk_vault",
    greenLabelConfig.reliefOrRiskVault,
    greenLabelConfig.usdcMint,
    summary,
  );
  await checkSecurityGovernanceConfig(
    connection,
    greenLabelConfig.securityGovernanceConfig,
    runtime.programId,
    runtime.expectedMode,
    summary,
  );
  checkAuthorityPolicy(greenLabelConfig, runtime.expectedMode, summary);

  printSummary(summary);
  process.exitCode = summary.fail.length > 0 ? 1 : 0;
}

main().catch((error: unknown) => {
  console.error("Read-only prelaunch sanity check failed unexpectedly:");
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
});
