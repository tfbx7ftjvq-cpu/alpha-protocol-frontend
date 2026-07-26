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
  deriveProtocolActivationRecord,
  deriveProtocolAuthorityControl,
  deriveProtocolModuleRegistry,
  loadDevnetContext,
  meta,
  requireDaoControlActivationConfirmation,
  readU64,
  sendOrPlan,
} from "../devnet/alpha-v1/common";

async function main(): Promise<void> {
  requireDaoControlActivationConfirmation();
  const ctx = await loadDevnetContext({
    scriptName: "protocol-authority-activate-dao-control",
    sendsTransactions: true,
    requiredInstructions: ["execute_activate_protocol_dao_control_v1"],
  });
  const proposalId = readU64("PROPOSAL_ID");
  const governanceConfig = deriveGovernanceConfig();
  const authorityControl = deriveProtocolAuthorityControl(governanceConfig);
  const proposal = deriveGovernanceProposal(proposalId);
  const queue = deriveExecutionQueueItem(proposalId);
  console.log("proposal_id:", proposalId.toString());
  console.log("authority_control:", authorityControl.toBase58());

  const tx = new Transaction().add(
    buildIx(ctx.idl, "execute_activate_protocol_dao_control_v1", [
      meta("executor", ctx.wallet, true, true),
      meta("current_authority", ctx.wallet, true),
      meta("governance_config", governanceConfig),
      meta("authority_control", authorityControl, false, true),
      meta("protocol_module_registry", deriveProtocolModuleRegistry(MODULE_CODES.Protocol)),
      meta("governance_proposal_action", deriveGovernanceProposalAction(proposal)),
      meta("governance_proposal", proposal),
      meta("governance_decision_adapter", deriveGovernanceAdapter(proposal)),
      meta("proposal_decision", deriveProposalDecision(proposalId)),
      meta("execution_queue_item", queue),
      meta("activation_record", deriveProtocolActivationRecord(authorityControl), false, true),
      meta("system_program", SYS),
    ]),
  );
  await sendOrPlan(ctx, "execute_activate_protocol_dao_control_v1", tx);
}

main().catch((error) => {
  console.error("protocol-authority activate-dao-control failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
