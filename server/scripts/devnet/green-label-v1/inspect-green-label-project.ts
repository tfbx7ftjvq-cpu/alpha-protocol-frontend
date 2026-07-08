import {
  DEFAULT_PUBLIC_KEY,
  deriveGreenBondVault,
  deriveGreenLabelDispute,
  deriveGreenLabelProject,
  fetchGreenLabelDispute,
  fetchGreenLabelProject,
  getTokenBalance,
  loadProvider,
  printDevnetScriptHeader,
  printGreenLabelDispute,
  printGreenLabelProject,
  readU64Env,
  formatUsdc,
} from "./common";

async function main(): Promise<void> {
  const provider = loadProvider();
  printDevnetScriptHeader({
    scriptName: "inspect-green-label-project",
    provider,
    sendsTransactions: false,
  });

  const projectId = readU64Env("PROJECT_ID", 0n);
  if (projectId === 0n) {
    throw new Error("PROJECT_ID is required and must be greater than 0.");
  }

  const projectKey = deriveGreenLabelProject(projectId);
  const project = await fetchGreenLabelProject(provider, projectId);
  if (!project) {
    throw new Error(`GreenLabelProjectV1 not found: ${projectKey.toBase58()}`);
  }

  console.log("project_pda:", projectKey.toBase58());
  printGreenLabelProject(project);

  if (!project.bondVault.equals(DEFAULT_PUBLIC_KEY)) {
    const bondVaultBalance = await getTokenBalance(provider, project.bondVault);
    console.log(
      "bond_vault_balance:",
      `${formatUsdc(bondVaultBalance)} USDC (${bondVaultBalance.toString()} base units)`,
    );
  } else {
    console.log("bond_vault_balance: no bond vault recorded");
  }

  const explicitDisputeId = process.env.DISPUTE_ID ? readU64Env("DISPUTE_ID", 0n) : null;
  if (explicitDisputeId !== null && explicitDisputeId > 0n) {
    const disputeKey = deriveGreenLabelDispute(projectKey, explicitDisputeId);
    const dispute = await fetchGreenLabelDispute(provider, projectKey, explicitDisputeId);
    if (!dispute) {
      throw new Error(`GreenLabelDisputeV1 not found: ${disputeKey.toBase58()}`);
    }
    console.log("dispute_pda:", disputeKey.toBase58());
    printGreenLabelDispute(dispute);
    return;
  }

  if (!project.activeDispute.equals(DEFAULT_PUBLIC_KEY)) {
    console.log("active_dispute:", project.activeDispute.toBase58());
    console.log("Set DISPUTE_ID to print the decoded dispute account.");
  } else {
    console.log("active_dispute: none");
  }
}

main().catch((error) => {
  console.error("inspect-green-label-project failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
