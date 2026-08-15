import assert from "assert";
import * as crypto from "crypto";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import { Keypair } from "@solana/web3.js";
import { verifyProgramArtifacts } from "./verify-program-artifacts";

const fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), "alpha-artifact-test-"));
const keypair = Keypair.generate();
const keypairPath = path.join(fixtureDir, "program-keypair.json");
const programPath = path.join(fixtureDir, "program.so");
const idlPath = path.join(fixtureDir, "program.json");
fs.writeFileSync(keypairPath, JSON.stringify(Array.from(keypair.secretKey)));
fs.writeFileSync(programPath, "program-fixture");
fs.writeFileSync(idlPath, JSON.stringify({ address: keypair.publicKey.toBase58(), metadata: { name: "fixture", version: "0.1.0", spec: "0.1.0" } }));
const hash = (file: string) => crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");

const result = verifyProgramArtifacts({
  programId: keypair.publicKey.toBase58(), programKeypairPath: keypairPath, programSoPath: programPath, idlPath,
  expectedProgramSha256: hash(programPath), expectedIdlSha256: hash(idlPath),
});
assert.strictEqual(result.status, "MATCH");
assert.strictEqual(result.keypairPublicKey, keypair.publicKey.toBase58());
assert.throws(() => verifyProgramArtifacts({
  programId: keypair.publicKey.toBase58(), programKeypairPath: keypairPath, programSoPath: programPath, idlPath,
  expectedProgramSha256: "BAD", expectedIdlSha256: hash(idlPath),
}), /lowercase 64-character/);
fs.writeFileSync(idlPath, JSON.stringify({ address: keypair.publicKey.toBase58(), metadata: { name: "fixture", version: "0.1.0", spec: "0.1.0", address: Keypair.generate().publicKey.toBase58() } }));
const legacyAddressMismatch = verifyProgramArtifacts({
  programId: keypair.publicKey.toBase58(), programKeypairPath: keypairPath, programSoPath: programPath, idlPath,
  expectedProgramSha256: hash(programPath), expectedIdlSha256: hash(idlPath),
});
assert.strictEqual(legacyAddressMismatch.status, "UNRESOLVED_BLOCKER");
fs.writeFileSync(idlPath, JSON.stringify({ address: Keypair.generate().publicKey.toBase58(), metadata: { name: "fixture", version: "0.1.0", spec: "0.1.0" } }));
const topLevelAddressMismatch = verifyProgramArtifacts({
  programId: keypair.publicKey.toBase58(), programKeypairPath: keypairPath, programSoPath: programPath, idlPath,
  expectedProgramSha256: hash(programPath), expectedIdlSha256: hash(idlPath),
});
assert.strictEqual(topLevelAddressMismatch.status, "UNRESOLVED_BLOCKER");
console.log("program artifact verifier tests passed");
