import {
  deriveGovernanceConfig,
  deriveProtocolAuthorityControl,
  loadDevnetContext,
  printAccount,
} from "../devnet/alpha-v1/common";

async function main(): Promise<void> {
  const ctx = await loadDevnetContext({
    scriptName: "protocol-authority-verify-legacy-fail-close",
    sendsTransactions: false,
  });
  const governanceConfig = deriveGovernanceConfig();
  const authorityControl = deriveProtocolAuthorityControl(governanceConfig);
  console.log("This script is read-only. It verifies the accounts needed for legacy fail-close review.");
  console.log("Legacy authority paths should remain Bootstrap-only after DaoControlled activation.");
  await printAccount(ctx.provider.connection, "governance_config_v1", governanceConfig);
  await printAccount(ctx.provider.connection, "protocol_authority_control_v1", authorityControl);
  console.log("Manual check: after DaoControlled activation, legacy authority-only unpause/pause paths must fail closed by on-chain mode checks.");
}

main().catch((error) => {
  console.error("protocol-authority verify-legacy-fail-close failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});