import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { verifyDependencyRiskRegister } from './dependency-risk-verify.ts';
import { verifyReleaseArtifacts } from './release-artifact-verify.ts';
import { readRoleBootstrapPlan, validateRoleBootstrapPlan } from './operations-staging/role-bootstrap-plan.ts';
import { readRecoveryRehearsalEvidence, validateRecoveryRehearsalEvidence } from './operations-staging/recovery-rehearsal.ts';

interface StagingEvidence { schemaVersion: 1; status: 'template' | 'verified'; environment: 'staging'; migrationThrough: '202608110002'; walletE2EPassed: true; gateMode: 'disabled'; noChainTransaction: true }

export interface PilotVerificationResult { status: 'GO' | 'NO-GO'; checks: Record<string, 'PASS' | 'MISSING' | 'FAIL'>; reasons: string[] }

export function verifyPublicPilot(
  root = process.cwd(),
): PilotVerificationResult {
  const checks: Record<string, 'PASS' | 'MISSING' | 'FAIL'> = {};
  const reasons: string[] = [];
  const run = (name: string, callback: () => void): void => {
    try { callback(); checks[name] = 'PASS'; } catch (error) { checks[name] = 'FAIL'; reasons.push(`${name}: ${error instanceof Error ? error.message : String(error)}`); }
  };
  run('releaseArtifact', () => {
    const manifest = verifyReleaseArtifacts(resolve(root, 'dist')).manifest;
    if (!/^[0-9a-f]{40}$/.test(manifest.commitSha) || manifest.buildContext === 'local') throw new Error('deployed CI/release commit evidence is required');
  });
  run('dependencyRiskRegister', () => { verifyDependencyRiskRegister(resolve(root, 'operations/dependency-risk-register.json'), resolve(root, 'package.json')); });
  run('roleBootstrapPlan', () => { validateRoleBootstrapPlan(readRoleBootstrapPlan(resolve(root, 'operations/role-bootstrap-plan.example.json'))); });
  run('recoveryEvidence', () => {
    const evidence = validateRecoveryRehearsalEvidence(readRecoveryRehearsalEvidence(resolve(root, 'operations/recovery-rehearsal-evidence.example.json')));
    if (evidence.status !== 'verified') throw new Error('verified isolated recovery rehearsal evidence is required');
  });
  run('stagingE2EEvidence', () => {
    const value = readJson(resolve(root, 'operations/staging-e2e-evidence.example.json')) as StagingEvidence;
    if (value.schemaVersion !== 1 || value.environment !== 'staging' || value.migrationThrough !== '202608110002' || value.walletE2EPassed !== true || value.gateMode !== 'disabled' || value.noChainTransaction !== true || value.status !== 'verified') throw new Error('verified Staging E2E evidence with disabled gate is required');
  });
  const status = reasons.length === 0 ? 'GO' : 'NO-GO';
  return { status, checks, reasons };
}

function readJson(path: string): unknown { try { return JSON.parse(readFileSync(path, 'utf8')); } catch { throw new Error('evidence must be readable valid JSON'); } }

if (process.argv[1]?.endsWith('operations-pilot-verify.ts')) {
  const result = verifyPublicPilot();
  process.stdout.write(`${JSON.stringify(result)}\n`);
  process.stdout.write(`Public Pilot ${result.status}: ${result.reasons.length === 0 ? 'all mandatory local evidence checks passed' : result.reasons.join('; ')}\n`);
}
