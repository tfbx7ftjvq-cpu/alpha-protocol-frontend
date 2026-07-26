import { Transaction } from "@solana/web3.js";
import {
  MODULE_CODES,
  SYS,
  buildIx,
  deriveExecutionQueueItem,
  deriveGovernanceAdapter,
  deriveGovernanceConfig,
  deriveGovernanceProposal,
  deriveGovernanceProposalAction,
  deriveProposalDecision,
  deriveProtocolAuthorityControl,
  deriveProtocolModuleRegistry,
  deriveProtocolUnpauseRecord,
  loadDevnetContext,
  meta,
  readU64,
  sendOrPlan,
} from "../devnet/alpha-v1/common";

async function main(): Promise<void> {
  const ctx = await loadDevnetContext({
    scriptName: "protocol-authority-recovery-unpause-e2e",
    sendsTransactions: true,
    requiredInstructions: ["execute_protocol_unpause_security_v1"],
  });
  const proposalId = readU64("PROPOSAL_ID");
  const governanceConfig = deriveGovernanceConfig();
  const authorityControl = deriveProtocolAuthorityControl(governanceConfig);
  const proposal = deriveGovernanceProposal(proposalId);
  const queue = deriveExecutionQueueItem(proposalId);
  console.log("proposal_id:", proposalId.toString());
  console.log("authority_control:", authorityControl.toBase58());

  const tx = new Transaction().add(
    buildIx(ctx.idl, "execute_protocol_unpause_security_v1", [
      meta("executor", ctx.wallet, true, true),
      meta("governance_config", governanceConfig, false, true),
      meta("authority_control", authorityControl),
      meta("protocol_module_registry", deriveProtocolModuleRegistry(MODULE_CODES.Protocol)),
      meta("governance_proposal_action", deriveGovernanceProposalAction(proposal)),
      meta("governance_proposal", proposal),
      meta("governance_decision_adapter", deriveGovernanceAdapter(proposal)),
      meta("proposal_decision", deriveProposalDecision(proposalId)),
      meta("execution_queue_item", queue, false, true),
      meta("unpause_record", deriveProtocolUnpauseRecord(queue), false, true),
      meta("system_program", SYS),
    ]),
  );
  await sendOrPlan(ctx, "execute_protocol_unpause_security_v1", tx);
}

main().catch((error) => {
  console.error("protocol-authority recovery-unpause-e2e failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
