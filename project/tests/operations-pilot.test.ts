import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { verifyDependencyRiskRegister } from '../scripts/dependency-risk-verify.ts';
import { verifyPublicPilot } from '../scripts/operations-pilot-verify.ts';
import { readRoleBootstrapPlan, validateRoleBootstrapPlan } from '../scripts/operations-staging/role-bootstrap-plan.ts';
import { validateRecoveryRehearsalEvidence } from '../scripts/operations-staging/recovery-rehearsal.ts';

test('role bootstrap plan requires separate staff, unique roles, and future expiry without RPC calls', () => {
  const plan = readRoleBootstrapPlan(fileURLToPath(
    new URL('../operations/role-bootstrap-plan.example.json', import.meta.url),
  ));
  assert.equal(validateRoleBootstrapPlan(plan).entries, 4);
  const duplicate = structuredClone(plan) as { entries: Array<{ userId: string; existingWeb3User: boolean }> };
  duplicate.entries[1]!.userId = duplicate.entries[0]!.userId;
  assert.throws(() => validateRoleBootstrapPlan(duplicate), /unique user/);
  const web3User = structuredClone(plan) as { entries: Array<{ existingWeb3User: boolean }> };
  web3User.entries[0]!.existingWeb3User = true;
  assert.throws(() => validateRoleBootstrapPlan(web3User), /Web3 users/);
});

test('recovery rehearsal evidence rejects current staging and requires isolated disabled target', () => {
  const evidence = JSON.parse(readFileSync(new URL('../operations/recovery-rehearsal-evidence.example.json', import.meta.url), 'utf8'));
  assert.equal(validateRecoveryRehearsalEvidence(evidence).target.classification, 'isolated_restore');
  const unsafe = structuredClone(evidence);
  unsafe.target.projectRef = 'neevswvhndkalxkainxo';
  assert.throws(() => validateRecoveryRehearsalEvidence(unsafe), /invalid or not isolated/);
});

test('dependency risk register covers the npm 10.9.8 baseline without direct production high findings', () => {
  assert.deepEqual(verifyDependencyRiskRegister(), { findings: 26, directProductionHigh: 0 });
});

test('pilot verifier is local-only and reports NO-GO until real release and rehearsal evidence exists', async () => {
  const result = verifyPublicPilot(fileURLToPath(new URL('../', import.meta.url)));
  assert.equal(result.status, 'NO-GO');
  assert.equal(result.checks.dependencyRiskRegister, 'PASS');
  assert.equal(result.checks.roleBootstrapPlan, 'PASS');
  assert.equal(result.checks.recoveryEvidence, 'FAIL');
  const source = readFileSync(new URL('../scripts/operations-pilot-verify.ts', import.meta.url), 'utf8');
  assert.doesNotMatch(source, /fetch\(|createClient|sendTransaction|grant_operations_role|revoke_operations_role|\.insert\(/);
});
