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
  readU64,
  sendOrPlan,
} from "../devnet/alpha-v1/common";

async function main(): Promise<void> {
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
      meta(ctx.wallet, true, true),
      meta(ctx.wallet, true),
      meta(governanceConfig),
      meta(authorityControl, false, true),
      meta(deriveProtocolModuleRegistry(MODULE_CODES.Protocol)),
      meta(deriveGovernanceProposalAction(proposal)),
      meta(proposal),
      meta(deriveGovernanceAdapter(proposal)),
      meta(deriveProposalDecision(proposalId)),
      meta(queue),
      meta(deriveProtocolActivationRecord(authorityControl), false, true),
      meta(SYS),
    ]),
  );
  await sendOrPlan(ctx, "execute_activate_protocol_dao_control_v1", tx);
}

main().catch((error) => {
  console.error("protocol-authority activate-dao-control failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});