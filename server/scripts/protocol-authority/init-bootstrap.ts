import { Transaction } from "@solana/web3.js";
import {
  SYS,
  accountExists,
  buildIx,
  deriveGovernanceConfig,
  deriveProtocolAuthorityControl,
  loadDevnetContext,
  meta,
  sendOrPlan,
} from "../devnet/alpha-v1/common";

async function main(): Promise<void> {
  const ctx = await loadDevnetContext({
    scriptName: "protocol-authority-init-bootstrap",
    sendsTransactions: true,
    requiredInstructions: ["initialize_protocol_authority_control_v1"],
  });
  const governanceConfig = deriveGovernanceConfig();
  const authorityControl = deriveProtocolAuthorityControl(governanceConfig);
  console.log("governance_config_v1:", governanceConfig.toBase58());
  console.log("protocol_authority_control_v1:", authorityControl.toBase58());

  if (!(await accountExists(ctx.provider.connection, governanceConfig))) {
    throw new Error(`GovernanceConfigV1 is missing: ${governanceConfig.toBase58()}`);
  }
  if (await accountExists(ctx.provider.connection, authorityControl)) {
    console.log("ProtocolAuthorityControlV1 already exists. No transaction needed.");
    return;
  }

  const tx = new Transaction().add(
    buildIx(ctx.idl, "initialize_protocol_authority_control_v1", [
      meta(governanceConfig),
      meta(authorityControl, false, true),
      meta(ctx.wallet, true, true),
      meta(SYS),
    ]),
  );
  await sendOrPlan(ctx, "initialize_protocol_authority_control_v1", tx);
}

main().catch((error) => {
  console.error("protocol-authority init-bootstrap failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});