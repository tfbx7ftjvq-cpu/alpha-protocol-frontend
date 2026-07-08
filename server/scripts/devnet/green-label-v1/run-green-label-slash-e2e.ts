import { Transaction } from "@solana/web3.js";
import {
  ACTION_TYPES,
  DEVNET_USDC_MINT,
  DEFAULT_PUBLIC_KEY,
  PROGRAM_ID,
  PROPOSAL_DECISIONS,
  PROPOSAL_TYPES,
  RUG_REASON_CODES,
  assertTokenBalanceAtLeast,
  buildCreateProposalDecisionIx,
  buildExecuteGreenLabelSlashIx,
  buildInitializeGreenBondVaultIx,
  buildLinkGreenLabelSecurityDecisionIx,
  buildLockGreenLabelBondIx,
  buildMarkDisputeReadyForDecisionIx,
  buildOpenGreenLabelDisputeIx,
  buildQueueExecutionIx,
  buildSubmitGreenLabelApplicationIx,
  deriveExecutionQueueItem,
  deriveGovernanceConfig,
  deriveGreenBondVault,
  deriveGreenLabelDispute,
  deriveGreenLabelProject,
  fetchExecutionQueue,
  fetchGovernanceConfig,
  fetchGreenLabelConfig,
  fetchGreenLabelDispute,
  fetchGreenLabelProject,
  formatUsdc,
  getOrCreateAta,
  getTokenBalance,
  loadProvider,
  nextProposalId,
  printDevnetScriptHeader,
  printGreenLabelConfig,
  printGreenLabelDispute,
  printGreenLabelProject,
  printVaultBalance,
  readEnumEnv,
  readHashEnv,
  readPublicKeyEnv,
  readU64Env,
  requireShortEnoughResponseWindow,
  selectGreenLabelBondAmount,
  sendAndConfirmLabeled,
  waitForTimelock,
} from "./common";

function assertConfigCanFinishDisputeWindow(disputeWindow: bigint, responseWindow: bigint): void {
  const maxWaitSeconds = readU64Env("MAX_RESPONSE_WAIT_SECONDS", 120n);
  const expectedWait = disputeWindow + responseWindow;
  if (expectedWait > maxWaitSeconds) {
    throw new Error(
      `Current Green Label config requires waiting about ${expectedWait.toString()}s before mark_dispute_ready_for_decision. ` +
        `MAX_RESPONSE_WAIT_SECONDS=${maxWaitSeconds.toString()}. This script will not bypass contract time rules. ` +
        "Use a short-window Devnet config or re-run with a larger MAX_RESPONSE_WAIT_SECONDS.",
    );
  }
}

async function main(): Promise<void> {
  const provider = loadProvider();
  printDevnetScriptHeader({
    scriptName: "run-green-label-slash-e2e",
    provider,
    sendsTransactions: true,
  });

  const wallet = provider.wallet.publicKey;
  const config = await fetchGreenLabelConfig(provider);
  if (!config) {
    throw new Error("GreenLabelConfigV1 is not initialized. Run devnet:green-label:setup first.");
  }
  printGreenLabelConfig(config);
  if (!config.usdcMint.equals(DEVNET_USDC_MINT)) {
    throw new Error(
      `Green Label config is not using Devnet USDC mint. Expected ${DEVNET_USDC_MINT.toBase58()}, got ${config.usdcMint.toBase58()}`,
    );
  }
  if (config.isPaused) {
    throw new Error("Green Label config is paused.");
  }
  assertConfigCanFinishDisputeWindow(config.disputeWindowSeconds, config.responseWindowSeconds);

  const governance = await fetchGovernanceConfig(provider);
  if (!governance) {
    throw new Error("Security Layer GovernanceConfigV1 is not initialized.");
  }
  if (!config.securityGovernanceConfig.equals(deriveGovernanceConfig())) {
    throw new Error(
      `Green Label config security_governance_config mismatch. Expected ${deriveGovernanceConfig().toBase58()}, got ${config.securityGovernanceConfig.toBase58()}`,
    );
  }

  const projectId = config.projectCount + 1n;
  const disputeId = readU64Env("DISPUTE_ID", 1n);
  const project = deriveGreenLabelProject(projectId);
  const dispute = deriveGreenLabelDispute(project, disputeId);
  const greenBondVault = deriveGreenBondVault(project);
  const { bondAmount, bondAmountSource } = selectGreenLabelBondAmount(config.minBaseBondUsdc);
  const projectNameHash = readHashEnv(
    "PROJECT_NAME_HASH",
    "alpha-green-label-devnet-project",
  );
  const projectUrlHash = readHashEnv(
    "PROJECT_URL_HASH",
    "https://example.dev/alpha-green-label",
  );
  const tokenMint = readPublicKeyEnv("TOKEN_MINT", DEFAULT_PUBLIC_KEY);
  const projectTreasuryWallet = readPublicKeyEnv("PROJECT_TREASURY_WALLET", wallet);
  const evidenceHash = readHashEnv("DISPUTE_EVIDENCE_HASH", "green-label-devnet-evidence");
  const reasonCode = readEnumEnv("REASON_CODE", "LiquidityRemoved", RUG_REASON_CODES);
  const actionType = readEnumEnv("ACTION_TYPE", "GreenLabelSlash", ACTION_TYPES);
  const proposalType = readEnumEnv("PROPOSAL_TYPE", "GreenLabelSlash", PROPOSAL_TYPES);
  const decision = readEnumEnv("DECISION", "Approved", PROPOSAL_DECISIONS);
  if (actionType.name !== "GreenLabelSlash" || proposalType.name !== "GreenLabelSlash") {
    throw new Error("Slash E2E requires ACTION_TYPE=GreenLabelSlash and PROPOSAL_TYPE=GreenLabelSlash.");
  }
  const payloadHash = readHashEnv(
    "PAYLOAD_HASH",
    `green-label-slash:${projectId.toString()}:${disputeId.toString()}`,
  );
  const proposalDuration = readU64Env(
    "DELAY_SECONDS",
    governance.minExecutionDelaySeconds > 0n ? governance.minExecutionDelaySeconds : 60n,
  );
  const proposalId = await nextProposalId(provider, governance);
  const projectOwnerUsdcAta = await getOrCreateAta(provider, config.usdcMint, wallet);

  console.log("project_id:", projectId.toString());
  console.log("proposal_id:", proposalId.toString());
  console.log("project:", project.toBase58());
  console.log("dispute:", dispute.toBase58());
  console.log("green_bond_vault:", greenBondVault.toBase58());
  console.log("project_owner_usdc_ata:", projectOwnerUsdcAta.toBase58());
  console.log("relief_or_risk_vault:", config.reliefOrRiskVault.toBase58());
  console.log("config min_base_bond_usdc:", formatUsdc(config.minBaseBondUsdc));
  console.log("selected bond_amount:", formatUsdc(bondAmount));
  console.log("bond amount source:", bondAmountSource);
  console.log("payload_hash:", payloadHash.toString("hex"));

  await assertTokenBalanceAtLeast(provider, projectOwnerUsdcAta, bondAmount, "project_owner_usdc_ata");
  const ownerBefore = await getTokenBalance(provider, projectOwnerUsdcAta);
  const reliefBefore = await getTokenBalance(provider, config.reliefOrRiskVault);
  const bondVaultBefore = await getTokenBalance(provider, greenBondVault);

  await sendAndConfirmLabeled(
    provider,
    "submit_green_label_application",
    new Transaction().add(
      buildSubmitGreenLabelApplicationIx({
        projectId,
        projectNameHash,
        projectUrlHash,
        projectTreasuryWallet,
        totalBondAmount: bondAmount,
        projectOwner: wallet,
        tokenMint,
      }),
    ),
  );

  await sendAndConfirmLabeled(
    provider,
    "initialize_green_bond_vault",
    new Transaction().add(
      buildInitializeGreenBondVaultIx({
        projectId,
        projectOwner: wallet,
        usdcMint: config.usdcMint,
      }),
    ),
  );

  await sendAndConfirmLabeled(
    provider,
    "lock_green_label_bond",
    new Transaction().add(
      buildLockGreenLabelBondIx({
        projectId,
        projectOwner: wallet,
        projectOwnerUsdcAta,
        usdcMint: config.usdcMint,
      }),
    ),
  );

  await sendAndConfirmLabeled(
    provider,
    "open_green_label_dispute",
    new Transaction().add(
      buildOpenGreenLabelDisputeIx({
        projectId,
        disputeId,
        reasonCode,
        evidenceHash,
        disputer: wallet,
      }),
    ),
  );

  const openedDispute = await fetchGreenLabelDispute(provider, project, disputeId);
  if (!openedDispute) {
    throw new Error("Dispute was not found after open_green_label_dispute.");
  }
  await requireShortEnoughResponseWindow(openedDispute);

  await sendAndConfirmLabeled(
    provider,
    "mark_dispute_ready_for_decision",
    new Transaction().add(
      buildMarkDisputeReadyForDecisionIx({
        projectId,
        disputeId,
        caller: wallet,
      }),
    ),
  );

  const now = BigInt(Math.floor(Date.now() / 1000));
  await sendAndConfirmLabeled(
    provider,
    "create_security_slash_decision",
    new Transaction().add(
      buildCreateProposalDecisionIx({
        proposalId,
        proposalType,
        decision,
        yesWeight: readU64Env("YES_WEIGHT", 100n),
        noWeight: readU64Env("NO_WEIGHT", 0n),
        startTs: now,
        endTs: now + proposalDuration,
        authority: wallet,
      }),
    ),
  );

  await sendAndConfirmLabeled(
    provider,
    "queue_green_label_slash",
    new Transaction().add(
      buildQueueExecutionIx({
        proposalId,
        actionType,
        targetProgram: PROGRAM_ID,
        targetAccount: dispute,
        payloadHash,
        authority: wallet,
      }),
    ),
  );

  const queue = await fetchExecutionQueue(provider, proposalId);
  if (!queue) {
    throw new Error(`Execution queue item not found: ${deriveExecutionQueueItem(proposalId).toBase58()}`);
  }
  await waitForTimelock(queue);

  await sendAndConfirmLabeled(
    provider,
    "link_green_label_security_decision",
    new Transaction().add(
      buildLinkGreenLabelSecurityDecisionIx({
        projectId,
        disputeId,
        proposalId,
        actionType,
        payloadHash,
        linker: wallet,
      }),
    ),
  );

  await sendAndConfirmLabeled(
    provider,
    "execute_green_label_slash",
    new Transaction().add(
      buildExecuteGreenLabelSlashIx({
        projectId,
        disputeId,
        executor: wallet,
        reliefOrRiskVault: config.reliefOrRiskVault,
        usdcMint: config.usdcMint,
        proposalId,
      }),
    ),
  );

  const finalProject = await fetchGreenLabelProject(provider, projectId);
  const finalDispute = await fetchGreenLabelDispute(provider, project, disputeId);
  if (finalProject) {
    printGreenLabelProject(finalProject);
  }
  if (finalDispute) {
    printGreenLabelDispute(finalDispute);
  }

  const ownerAfter = await getTokenBalance(provider, projectOwnerUsdcAta);
  const reliefAfter = await getTokenBalance(provider, config.reliefOrRiskVault);
  const bondVaultAfter = await getTokenBalance(provider, greenBondVault);
  console.log("Balance deltas:");
  console.log("  project_owner_usdc_ata:", formatUsdc(ownerAfter - ownerBefore));
  console.log("  relief_or_risk_vault:", formatUsdc(reliefAfter - reliefBefore));
  console.log("  green_bond_vault:", formatUsdc(bondVaultAfter - bondVaultBefore));
  await printVaultBalance(provider, "final green_bond_vault", greenBondVault);
  await printVaultBalance(provider, "final project_owner_usdc_ata", projectOwnerUsdcAta);
  await printVaultBalance(provider, "final relief_or_risk_vault", config.reliefOrRiskVault);
}

main().catch((error) => {
  console.error("run-green-label-slash-e2e failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
