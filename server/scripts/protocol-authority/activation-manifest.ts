import * as fs from "fs";
import { execFileSync } from "child_process";
import {
  EXPECTED_UPGRADE_AUTHORITY,
  PROGRAM_ID,
  deriveGovernanceConfig,
  deriveProtocolAuthorityControl,
  loadDevnetContext,
  readProgramDeployment,
} from "../devnet/alpha-v1/common";

function tryExec(command: string, args: string[], cwd = process.cwd()): string {
  try {
    return execFileSync(command, args, { cwd, encoding: "utf8" }).trim();
  } catch (error) {
    return `<unavailable: ${command} ${args.join(" ")}>`;
  }
}

async function main(): Promise<void> {
  const ctx = await loadDevnetContext({ scriptName: "protocol-authority-activation-manifest", sendsTransactions: false });
  const deployment = await readProgramDeployment(ctx.provider.connection);
  const balance = await ctx.provider.connection.getBalance(ctx.wallet);
  const soPath = "target/deploy/my_first_solana_program.so";
  const localBinaryBytes = fs.existsSync(soPath) ? fs.statSync(soPath).size : 0;
  const gitCommit = tryExec("git", ["rev-parse", "--short", "HEAD"], "..");
  const gitStatus = tryExec("git", ["status", "--short"], "..");

  console.log("# Devnet Program Upgrade Manifest");
  console.log("cluster: Devnet");
  console.log("RPC URL:", ctx.rpcUrl);
  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("wallet:", ctx.wallet.toBase58());
  console.log("wallet SOL balance:", balance / 1_000_000_000);
  console.log("expected upgrade authority:", EXPECTED_UPGRADE_AUTHORITY.toBase58());
  console.log("current upgrade authority:", deployment.upgradeAuthority?.toBase58() ?? "<none>");
  console.log("wallet matches upgrade authority:", deployment.upgradeAuthority?.equals(ctx.wallet) ?? false);
  console.log("ProgramData:", deployment.programData?.toBase58() ?? "<missing>");
  console.log("current ProgramData bytes:", deployment.dataLength);
  console.log("new local binary bytes:", localBinaryBytes || "<missing build output>");
  console.log("git commit:", gitCommit);
  console.log("working tree clean:", gitStatus.length === 0);
  console.log("governance_config_v1:", deriveGovernanceConfig().toBase58());
  console.log("protocol_authority_control_v1:", deriveProtocolAuthorityControl().toBase58());
  console.log("deployment command: anchor upgrade target/deploy/my_first_solana_program.so --program-id", PROGRAM_ID.toBase58());
  console.log("STOP: do not deploy until user says: È·ÈÏÖ´ÐÐDevnet program upgrade");
}

main().catch((error) => {
  console.error("protocol-authority activation-manifest failed:");
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});