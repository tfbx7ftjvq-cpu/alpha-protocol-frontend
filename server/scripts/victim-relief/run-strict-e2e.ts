import { PublicKey, Transaction } from "@solana/web3.js";
import {
  DEVNET_USDC_MINT,
  MODULE_CODES,
  SYS,
  TOKEN,
  buildIx,
  deriveAppeal,
  deriveAppealRecord,
  deriveDecisionRecord,
  deriveEvidenceSnapshot,
  deriveExecutionQueueItem,
  deriveGovernanceAdapter,
  deriveGovernanceConfig,
  deriveGovernanceProposal,
  deriveGovernanceProposalAction,
  derivePauseRecord,
  derivePayoutCancelRecord,
  derivePayoutRecord,
  deriveProposalDecision,
  deriveProtocolModuleRegistry,
  deriveReliefPayoutRequest,
  deriveTreasuryPdas,
  deriveVictimReliefCase,
  deriveVictimReliefClaimantState,
  deriveVictimReliefConfig,
  deriveVictimReliefPolicy,
  loadDevnetContext,
  meta,
  pk,
  printRequiredEnv,
  readU64,
  requiredScenarioBasics,
  sendOrPlan,
} from "../devnet/alpha-v1/common";

type GovAccounts = {
  proposal: PublicKey;
  action: PublicKey;
  adapter: PublicKey;
  decision: PublicKey;
  queue: PublicKey;
};

function gov(proposalId: bigint): GovAccounts {
  const proposal = deriveGovernanceProposal(proposalId);
  return {
    proposal,
    action: deriveGovernanceProposalAction(proposal),
    adapter: deriveGovernanceAdapter(proposal),
    decision: deriveProposalDecision(proposalId),
    queue: deriveExecutionQueueItem(proposalId),
  };
}

function baseCaseAccounts(): {
  caseId: bigint;
  claimant: PublicKey;
  recipient: PublicKey | null;
  config: PublicKey;
  policy: PublicKey;
  claimantState: PublicKey;
  caseKey: PublicKey;
  snapshot: PublicKey;
  request: PublicKey;
  appeal: PublicKey;
} {
  const basics = requiredScenarioBasics();
  const config = deriveVictimReliefConfig();
  const policy = deriveVictimReliefPolicy(config);
  const caseKey = deriveVictimReliefCase(config, basics.caseId);
  return {
    ...basics,
    config,
    policy,
    claimantState: deriveVictimReliefClaimantState(config, basics.claimant),
    caseKey,
    snapshot: deriveEvidenceSnapshot(caseKey),
    request: deriveReliefPayoutRequest(caseKey),
    appeal: deriveAppeal(caseKey),
  };
}

function originalPayoutTx(ctxIdl: any): Transaction {
  printRequiredEnv(["PROPOSAL_ID", "CASE_ID", "CLAIMANT", "RECIPIENT_USDC_TOKEN_ACCOUNT"]);
  const proposalId = readU64("PROPOSAL_ID");
  const g = gov(proposalId);
  const c = baseCaseAccounts();
  if (!c.recipient) {
    throw new Error("RECIPIENT_USDC_TOKEN_ACCOUNT is required for payout scenarios.");
  }
  const treasury = deriveTreasuryPdas();
  const ix = buildIx(ctxIdl, "execute_victim_relief_approved_payout_v1", [
    meta("security_governance_config", deriveGovernanceConfig()),
    meta("governance_proposal", g.proposal),
    meta("governance_proposal_action", g.action),
    meta("proposal_decision", g.decision),
    meta("execution_queue_item", g.queue),
    meta("victim_relief_config", c.config),
    meta("victim_relief_policy", c.policy),
    meta("claimant_state", c.claimantState, false, true),
    meta("victim_relief_case", c.caseKey, false, true),
    meta("evidence_snapshot", c.snapshot),
    meta("relief_payout_request", c.request, false, true),
    meta("decision_execution_record", deriveDecisionRecord(g.queue)),
    meta("treasury_config", treasury.treasuryConfigV2),
    meta("vault_authority", treasury.vaultAuthorityV2),
    meta("relief_usdc_vault", treasury.reliefUsdcVault, false, true),
    meta("recipient_usdc_token_account", c.recipient, false, true),
    meta("usdc_mint", DEVNET_USDC_MINT),
    meta("relief_payout_execution_record", derivePayoutRecord(c.request), false, true),
    meta("executor", pk(process.env.EXECUTOR || process.env.ANCHOR_WALLET_PUBKEY, "EXECUTOR or ANCHOR_WALLET_PUBKEY"), true, true),
    meta("token_program", TOKEN),
    meta("system_program", SYS),
  ]);
  return new Transaction().add(ix);
}

function rejectUpholdTx(ctxIdl: any): Transaction {
  printRequiredEnv(["PROPOSAL_ID", "ORIGINAL_PROPOSAL_ID", "CASE_ID", "CLAIMANT"]);
  const proposalId = readU64("PROPOSAL_ID");
  const originalProposalId = readU64("ORIGINAL_PROPOSAL_ID");
  const g = gov(proposalId);
  const original = gov(originalProposalId);
  const c = baseCaseAccounts();
  const treasury = deriveTreasuryPdas();
  const ix = buildIx(ctxIdl, "execute_uphold_victim_relief_appeal_v1", [
    meta("security_governance_config", deriveGovernanceConfig()),
    meta("protocol_module_registry", deriveProtocolModuleRegistry(MODULE_CODES.VictimRelief)),
    meta("governance_proposal", g.proposal),
    meta("governance_proposal_action", g.action),
    meta("governance_decision_adapter", g.adapter),
    meta("proposal_decision", g.decision),
    meta("execution_queue_item", g.queue),
    meta("victim_relief_config", c.config),
    meta("victim_relief_case", c.caseKey, false, true),
    meta("victim_relief_appeal", c.appeal, false, true),
    meta("victim_relief_policy", c.policy),
    meta("original_evidence_snapshot", c.snapshot),
    meta("original_decision_record", deriveDecisionRecord(original.queue)),
    meta("treasury_config", treasury.treasuryConfigV2),
    meta("vault_authority", treasury.vaultAuthorityV2),
    meta("relief_usdc_vault", treasury.reliefUsdcVault),
    meta("usdc_mint", DEVNET_USDC_MINT),
    meta("appeal_decision_execution_record", deriveAppealRecord(g.queue), false, true),
    meta("executor", pk(process.env.EXECUTOR || process.env.ANCHOR_WALLET_PUBKEY, "EXECUTOR or ANCHOR_WALLET_PUBKEY"), true, true),
    meta("system_program", SYS),
  ]);
  return new Transaction().add(ix);
}

function overturnPayoutTx(ctxIdl: any): Transaction {
  printRequiredEnv(["PROPOSAL_ID", "ORIGINAL_PROPOSAL_ID", "CASE_ID", "CLAIMANT", "RECIPIENT_USDC_TOKEN_ACCOUNT"]);
  const proposalId = readU64("PROPOSAL_ID");
  const originalProposalId = readU64("ORIGINAL_PROPOSAL_ID");
  const g = gov(proposalId);
  const original = gov(originalProposalId);
  const c = baseCaseAccounts();
  if (!c.recipient) {
    throw new Error("RECIPIENT_USDC_TOKEN_ACCOUNT is required for payout scenarios.");
  }
  const treasury = deriveTreasuryPdas();
  const ix = buildIx(ctxIdl, "execute_victim_relief_overturn_payout_v1", [
    meta("security_governance_config", deriveGovernanceConfig()),
    meta("governance_proposal", g.proposal),
    meta("governance_proposal_action", g.action),
    meta("proposal_decision", g.decision),
    meta("execution_queue_item", g.queue),
    meta("victim_relief_config", c.config),
    meta("victim_relief_policy", c.policy),
    meta("claimant_state", c.claimantState, false, true),
    meta("victim_relief_case", c.caseKey, false, true),
    meta("victim_relief_appeal", c.appeal),
    meta("original_evidence_snapshot", c.snapshot),
    meta("original_decision_record", deriveDecisionRecord(original.queue)),
    meta("appeal_decision_execution_record", deriveAppealRecord(g.queue)),
    meta("relief_payout_request", c.request, false, true),
    meta("treasury_config", treasury.treasuryConfigV2),
    meta("vault_authority", treasury.vaultAuthorityV2),
    meta("relief_usdc_vault", treasury.reliefUsdcVault, false, true),
    meta("recipient_usdc_token_account", c.recipient, false, true),
    meta("usdc_mint", DEVNET_USDC_MINT),
    meta("relief_payout_execution_record", derivePayoutRecord(c.request), false, true),
    meta("executor", pk(process.env.EXECUTOR || process.env.ANCHOR_WALLET_PUBKEY, "EXECUTOR or ANCHOR_WALLET_PUBKEY"), true, true),
    meta("token_program", TOKEN),
    meta("system_program", SYS),
  ]);
  return new Transaction().add(ix);
}

function cancelOriginalTx(ctxIdl: any): Transaction {
  printRequiredEnv(["PROPOSAL_ID", "ORIGINAL_PROPOSAL_ID", "CASE_ID", "CLAIMANT"]);
  const cancellationProposalId = readU64("PROPOSAL_ID");
  const originalProposalId = readU64("ORIGINAL_PROPOSAL_ID");
  const cancel = gov(cancellationProposalId);
  const original = gov(originalProposalId);
  const c = baseCaseAccounts();
  const treasury = deriveTreasuryPdas();
  const ix = buildIx(ctxIdl, "execute_cancel_original_victim_relief_payout_v1", [
    meta("security_governance_config", deriveGovernanceConfig()),
    meta("protocol_module_registry", deriveProtocolModuleRegistry(MODULE_CODES.VictimRelief)),
    meta("governance_proposal", original.proposal),
    meta("governance_proposal_action", original.action),
    meta("proposal_decision", original.decision),
    meta("execution_queue_item", original.queue),
    meta("decision_execution_record", deriveDecisionRecord(original.queue)),
    meta("cancellation_governance_proposal", cancel.proposal),
    meta("cancellation_governance_proposal_action", cancel.action),
    meta("cancellation_governance_decision_adapter", cancel.adapter),
    meta("cancellation_proposal_decision", cancel.decision),
    meta("cancellation_execution_queue_item", cancel.queue),
    meta("victim_relief_config", c.config),
    meta("victim_relief_policy", c.policy),
    meta("claimant_state", c.claimantState, false, true),
    meta("victim_relief_case", c.caseKey, false, true),
    meta("evidence_snapshot", c.snapshot),
    meta("relief_payout_request", c.request, false, true),
    meta("treasury_config", treasury.treasuryConfigV2),
    meta("payout_cancellation_record", derivePayoutCancelRecord(c.request), false, true),
    meta("executor", pk(process.env.EXECUTOR || process.env.ANCHOR_WALLET_PUBKEY, "EXECUTOR or ANCHOR_WALLET_PUBKEY"), true, true),
    meta("system_program", SYS),
  ]);
  return new Transaction().add(ix);
}

function cancelOverturnTx(ctxIdl: any): Transaction {
  printRequiredEnv(["PROPOSAL_ID", "OVERTURN_PROPOSAL_ID", "ORIGINAL_PROPOSAL_ID", "CASE_ID", "CLAIMANT"]);
  const cancellationProposalId = readU64("PROPOSAL_ID");
  const overturnProposalId = readU64("OVERTURN_PROPOSAL_ID");
  const originalProposalId = readU64("ORIGINAL_PROPOSAL_ID");
  const cancel = gov(cancellationProposalId);
  const overturn = gov(overturnProposalId);
  const original = gov(originalProposalId);
  const c = baseCaseAccounts();
  const treasury = deriveTreasuryPdas();
  const ix = buildIx(ctxIdl, "execute_cancel_overturn_victim_relief_payout_v1", [
    meta("security_governance_config", deriveGovernanceConfig()),
    meta("protocol_module_registry", deriveProtocolModuleRegistry(MODULE_CODES.VictimRelief)),
    meta("governance_proposal", overturn.proposal),
    meta("governance_proposal_action", overturn.action),
    meta("proposal_decision", overturn.decision),
    meta("execution_queue_item", overturn.queue),
    meta("victim_relief_appeal", c.appeal),
    meta("original_evidence_snapshot", c.snapshot),
    meta("original_decision_record", deriveDecisionRecord(original.queue)),
    meta("appeal_decision_execution_record", deriveAppealRecord(overturn.queue)),
    meta("cancellation_governance_proposal", cancel.proposal),
    meta("cancellation_governance_proposal_action", cancel.action),
    meta("cancellation_governance_decision_adapter", cancel.adapter),
    meta("cancellation_proposal_decision", cancel.decision),
    meta("cancellation_execution_queue_item", cancel.queue),
    meta("victim_relief_config", c.config),
    meta("victim_relief_policy", c.policy),
    meta("claimant_state", c.claimantState, false, true),
    meta("victim_relief_case", c.caseKey, false, true),
    meta("relief_payout_request", c.request, false, true),
    meta("treasury_config", treasury.treasuryConfigV2),
    meta("payout_cancellation_record", derivePayoutCancelRecord(c.request), false, true),
    meta("executor", pk(process.env.EXECUTOR || process.env.ANCHOR_WALLET_PUBKEY, "EXECUTOR or ANCHOR_WALLET_PUBKEY"), true, true),
    meta("system_program", SYS),
  ]);
  return new Transaction().add(ix);
}

function pauseTx(ctxIdl: any): { label: string; tx: Transaction } {
  const mode = (process.env.PAUSE_MODE || "guardian").toLowerCase();
  const config = deriveVictimReliefConfig();
  if (mode === "guardian") {
    printRequiredEnv(["PAUSE_MODE=guardian", "EMERGENCY_GUARDIAN"]);
    const ix = buildIx(ctxIdl, "guardian_pause_victim_relief_v1", [
      meta("security_governance_config", deriveGovernanceConfig()),
      meta("victim_relief_config", config, false, true),
      meta("emergency_guardian", pk(process.env.EMERGENCY_GUARDIAN, "EMERGENCY_GUARDIAN"), true),
    ]);
    return { label: "guardian_pause_victim_relief_v1", tx: new Transaction().add(ix) };
  }
  printRequiredEnv(["PAUSE_MODE", "PROPOSAL_ID"]);
  const proposalId = readU64("PROPOSAL_ID");
  const g = gov(proposalId);
  const instructionName = mode === "dao-unpause" ? "execute_unpause_victim_relief_v1" : "execute_pause_victim_relief_v1";
  const ix = buildIx(ctxIdl, instructionName, [
    meta("security_governance_config", deriveGovernanceConfig()),
    meta("protocol_module_registry", deriveProtocolModuleRegistry(MODULE_CODES.VictimRelief)),
    meta("governance_proposal", g.proposal),
    meta("governance_proposal_action", g.action),
    meta("governance_decision_adapter", g.adapter),
    meta("proposal_decision", g.decision),
    meta("execution_queue_item", g.queue),
    meta("victim_relief_config", config, false, true),
    meta("pause_execution_record", derivePauseRecord(g.queue), false, true),
    meta("executor", pk(process.env.EXECUTOR || process.env.ANCHOR_WALLET_PUBKEY, "EXECUTOR or ANCHOR_WALLET_PUBKEY"), true, true),
    meta("system_program", SYS),
  ]);
  return { label: instructionName, tx: new Transaction().add(ix) };
}

async function main(): Promise<void> {
  const scenario = process.argv[2];
  const requiredInstructions = [
    "execute_victim_relief_approved_payout_v1",
    "execute_uphold_victim_relief_appeal_v1",
    "execute_victim_relief_overturn_payout_v1",
    "execute_cancel_original_victim_relief_payout_v1",
    "execute_cancel_overturn_victim_relief_payout_v1",
    "guardian_pause_victim_relief_v1",
    "execute_pause_victim_relief_v1",
    "execute_unpause_victim_relief_v1",
  ];
  const ctx = await loadDevnetContext({
    scriptName: `victim-relief-${scenario}`,
    sendsTransactions: true,
    requiredInstructions,
  });
  process.env.ANCHOR_WALLET_PUBKEY = ctx.wallet.toBase58();

  const scenarioBuilders: Record<string, () => { label: string; tx: Transaction } | Transaction> = {
    "original-payout": () => originalPayoutTx(ctx.idl),
    "reject-uphold": () => rejectUpholdTx(ctx.idl),
    "overturn-payout": () => overturnPayoutTx(ctx.idl),
    "cancel-original": () => cancelOriginalTx(ctx.idl),
    "cancel-overturn": () => cancelOverturnTx(ctx.idl),
    pause: () => pauseTx(ctx.idl),
  };

  if (!scenario || !scenarioBuilders[scenario]) {
    throw new Error(`Unknown scenario: ${scenario}. Expected one of ${Object.keys(scenarioBuilders).join(", ")}`);
  }

  const built = scenarioBuilders[scenario]();
  const tx = built instanceof Transaction ? built : built.tx;
  const label = built instanceof Transaction ? `victim_relief_${scenario}` : built.label;
  await sendOrPlan(ctx, label, tx);
}

main().catch((error) => {
  console.error("victim-relief strict e2e failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
