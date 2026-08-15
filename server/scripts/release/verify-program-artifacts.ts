import * as crypto from "crypto";
import * as fs from "fs";
import * as path from "path";
import { Keypair, PublicKey } from "@solana/web3.js";

type ArtifactInput = {
  programId: string;
  programKeypairPath: string;
  programSoPath: string;
  idlPath: string;
  expectedProgramSha256: string;
  expectedIdlSha256: string;
  deployedDumpPath?: string;
  expectedDeployedDumpSha256?: string;
};

export type ArtifactVerification = {
  status: "MATCH" | "EXPLAINED_MISMATCH" | "UNRESOLVED_BLOCKER";
  programId: string;
  keypairPublicKey?: string;
  programSha256?: string;
  idlSha256?: string;
  deployedDumpSha256?: string;
  errors: string[];
};

const SHA256 = /^[a-f0-9]{64}$/;
const ANCHOR_IDL_SPEC = "0.1.0";

function requireFile(filePath: string, label: string): string {
  const resolved = path.resolve(filePath);
  if (!fs.statSync(resolved).isFile()) {
    throw new Error(`${label} must be a file`);
  }
  return resolved;
}

function sha256(filePath: string): string {
  return crypto.createHash("sha256").update(fs.readFileSync(filePath)).digest("hex");
}

function requireSha256(value: string, label: string): void {
  if (!SHA256.test(value)) {
    throw new Error(`${label} must be a lowercase 64-character SHA-256`);
  }
}

function readKeypairPublicKey(filePath: string): string {
  const parsed: unknown = JSON.parse(fs.readFileSync(filePath, "utf8"));
  if (!Array.isArray(parsed) || !parsed.every((item) => Number.isInteger(item) && item >= 0 && item <= 255)) {
    throw new Error("program keypair must be a JSON byte array");
  }
  return Keypair.fromSecretKey(Uint8Array.from(parsed as number[])).publicKey.toBase58();
}

function readIdlProgramAddresses(filePath: string): { address?: string; metadataAddress?: string; metadataSpec?: string } {
  const parsed: unknown = JSON.parse(fs.readFileSync(filePath, "utf8"));
  if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
    throw new Error("IDL must be a JSON object");
  }
  const idl = parsed as { address?: unknown; metadata?: { address?: unknown; spec?: unknown } };
  return {
    address: typeof idl.address === "string" ? idl.address : undefined,
    metadataAddress: typeof idl.metadata?.address === "string" ? idl.metadata.address : undefined,
    metadataSpec: typeof idl.metadata?.spec === "string" ? idl.metadata.spec : undefined,
  };
}

export function verifyProgramArtifacts(input: ArtifactInput): ArtifactVerification {
  requireSha256(input.expectedProgramSha256, "expected program SHA-256");
  requireSha256(input.expectedIdlSha256, "expected IDL SHA-256");
  if ((input.deployedDumpPath === undefined) !== (input.expectedDeployedDumpSha256 === undefined)) {
    throw new Error("deployed dump path and expected deployed dump SHA-256 must be provided together");
  }
  if (input.expectedDeployedDumpSha256 !== undefined) {
    requireSha256(input.expectedDeployedDumpSha256, "expected deployed dump SHA-256");
  }

  const programId = new PublicKey(input.programId).toBase58();
  const keypairPath = requireFile(input.programKeypairPath, "program keypair");
  const programSoPath = requireFile(input.programSoPath, "program artifact");
  const idlPath = requireFile(input.idlPath, "IDL");
  const result: ArtifactVerification = { status: "MATCH", programId, errors: [] };

  try {
    result.keypairPublicKey = readKeypairPublicKey(keypairPath);
    if (result.keypairPublicKey !== programId) result.errors.push("program keypair public key does not match program ID");
  } catch (error) {
    result.errors.push(error instanceof Error ? error.message : "program keypair could not be read");
  }

  result.programSha256 = sha256(programSoPath);
  if (result.programSha256 !== input.expectedProgramSha256) result.errors.push("program artifact SHA-256 mismatch");
  result.idlSha256 = sha256(idlPath);
  if (result.idlSha256 !== input.expectedIdlSha256) result.errors.push("IDL SHA-256 mismatch");

  try {
    const addresses = readIdlProgramAddresses(idlPath);
    if (addresses.metadataSpec !== ANCHOR_IDL_SPEC) result.errors.push("IDL metadata.spec is not the supported Anchor IDL spec");
    if (addresses.address !== programId) result.errors.push("IDL top-level address does not match program ID");
    // Anchor IDL spec 0.1.0 binds program identity at top level. metadata.address is legacy-only;
    // if present, it must still agree with the required top-level binding.
    if (addresses.metadataAddress !== undefined && addresses.metadataAddress !== programId) {
      result.errors.push("legacy IDL metadata.address does not match program ID");
    }
  } catch (error) {
    result.errors.push(error instanceof Error ? error.message : "IDL could not be read");
  }

  if (input.deployedDumpPath && input.expectedDeployedDumpSha256) {
    result.deployedDumpSha256 = sha256(requireFile(input.deployedDumpPath, "deployed dump"));
    if (result.deployedDumpSha256 !== input.expectedDeployedDumpSha256) result.errors.push("deployed dump SHA-256 mismatch");
  }
  if (result.errors.length > 0) result.status = "UNRESOLVED_BLOCKER";
  return result;
}

function readArgs(argv: string[]): ArtifactInput {
  const values = new Map<string, string>();
  for (let index = 0; index < argv.length; index += 2) {
    const flag = argv[index];
    const value = argv[index + 1];
    if (!flag?.startsWith("--") || value === undefined || value.startsWith("--")) throw new Error("arguments must use --name value pairs");
    values.set(flag.slice(2), value);
  }
  const required = ["program-id", "program-keypair", "program-so", "idl", "expected-program-sha256", "expected-idl-sha256"];
  for (const key of required) if (!values.has(key)) throw new Error(`missing --${key}`);
  return {
    programId: values.get("program-id")!,
    programKeypairPath: values.get("program-keypair")!,
    programSoPath: values.get("program-so")!,
    idlPath: values.get("idl")!,
    expectedProgramSha256: values.get("expected-program-sha256")!,
    expectedIdlSha256: values.get("expected-idl-sha256")!,
    deployedDumpPath: values.get("deployed-dump"),
    expectedDeployedDumpSha256: values.get("expected-deployed-dump-sha256"),
  };
}

if (require.main === module) {
  try {
    const result = verifyProgramArtifacts(readArgs(process.argv.slice(2)));
    console.log(JSON.stringify(result));
    console.log(`artifact verification: ${result.status}`);
    process.exitCode = result.status === "MATCH" ? 0 : 1;
  } catch (error) {
    console.log(JSON.stringify({ status: "UNRESOLVED_BLOCKER", errors: [error instanceof Error ? error.message : "verification failed"] }));
    console.log("artifact verification: UNRESOLVED_BLOCKER");
    process.exitCode = 1;
  }
}
