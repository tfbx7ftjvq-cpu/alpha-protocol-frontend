import { existsSync, readFileSync, realpathSync, statSync } from 'node:fs';
import { resolve, relative } from 'node:path';
import { verifyDependencyRiskRegister } from './dependency-risk-verify.ts';
import { verifyReleaseArtifacts } from './release-artifact-verify.ts';
import { assertReleaseManifest, type ReleaseManifest } from './release-manifest.ts';
import { readRoleBootstrapPlan, validateRoleBootstrapPlan } from './operations-staging/role-bootstrap-plan.ts';
import { readRecoveryRehearsalEvidence, validateRecoveryRehearsalEvidence } from './operations-staging/recovery-rehearsal.ts';

interface StagingEvidence {
  schemaVersion: 1;
  status: 'template' | 'verified';
  environment: 'staging';
  migrationThrough: '202608110002';
  walletE2EPassed: true;
  executionGateMode: 'wallet_staging';
  postE2EGateMode: 'wallet_staging';
  cleanupPassed: true;
  noChainTransaction: true;
}

export interface PilotEvidencePaths {
  release?: string;
  roles?: string;
  recovery?: string;
  stagingE2E?: string;
}

export interface PilotVerificationResult {
  status: 'GO' | 'NO-GO';
  checks: Record<'release' | 'dependency' | 'roles' | 'recovery' | 'stagingE2E', 'PASS' | 'MISSING' | 'FAIL'>;
  evidenceSources: Record<'release' | 'roles' | 'recovery' | 'stagingE2E', 'template' | 'external'>;
  reasons: string[];
}

const SENSITIVE_EVIDENCE = /(?:\bsb_secret_[A-Za-z0-9_-]+\b|\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b|-----BEGIN(?: [A-Z]+)? PRIVATE KEY-----|wallet\s+private\s+key|["']?(?:service[_-]?role|servicerole|wallet[_-]?private[_-]?key|walletprivatekey|private[_-]?key|privatekey|access[_-]?token|refresh[_-]?token|captcha[_-]?token|turnstile[_-]?token|token|secret)["']?\s*[:=]|[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,})/i;

export function verifyPublicPilot(
  root = process.cwd(),
  paths: PilotEvidencePaths = pathsFromEnvironment(process.env),
): PilotVerificationResult {
  const checks: PilotVerificationResult['checks'] = {
    release: 'MISSING', dependency: 'MISSING', roles: 'MISSING', recovery: 'MISSING', stagingE2E: 'MISSING',
  };
  const evidenceSources: PilotVerificationResult['evidenceSources'] = {
    release: paths.release ? 'external' : 'template', roles: paths.roles ? 'external' : 'template', recovery: paths.recovery ? 'external' : 'template', stagingE2E: paths.stagingE2E ? 'external' : 'template',
  };
  const reasons: string[] = [];
  const run = (name: keyof PilotVerificationResult['checks'], callback: () => void): void => {
    try { callback(); checks[name] = 'PASS'; } catch (error) { checks[name] = 'FAIL'; reasons.push(`${name}: ${message(error)}`); }
  };

  run('release', () => {
    const manifest = readReleaseEvidence(root, paths.release);
    if (!paths.release) throw new Error('external deployed release.json evidence is required');
    assertDeployedRelease(manifest);
  });
  run('dependency', () => { verifyDependencyRiskRegister(resolve(root, 'operations/dependency-risk-register.json'), resolve(root, 'package.json')); });
  run('roles', () => {
    validateRoleBootstrapPlan(readRoleEvidence(root, paths.roles));
    if (!paths.roles) throw new Error('external approved role plan is required');
  });
  run('recovery', () => {
    const evidence = validateRecoveryRehearsalEvidence(readRecoveryEvidence(root, paths.recovery));
    if (!paths.recovery || evidence.status !== 'verified') throw new Error('external verified isolated recovery rehearsal evidence is required');
  });
  run('stagingE2E', () => {
    const evidence = validateStagingEvidence(readStagingEvidence(root, paths.stagingE2E));
    if (!paths.stagingE2E || evidence.status !== 'verified') throw new Error('external verified wallet Staging E2E evidence is required');
  });
  return { status: reasons.length === 0 ? 'GO' : 'NO-GO', checks, evidenceSources, reasons };
}

export function assertExternalEvidencePath(path: string, root: string): string {
  const resolved = resolve(path);
  if (!existsSync(resolved) || !statSync(resolved).isFile()) throw new Error('external evidence path must name a readable file');
  const gitRoot = findGitRoot(root);
  const actual = realpathSync.native(resolved);
  if (isInside(actual, gitRoot)) throw new Error('external evidence must be outside the Git working tree');
  const contents = readFileSync(actual, 'utf8');
  if (SENSITIVE_EVIDENCE.test(contents)) throw new Error('external evidence contains prohibited secret, token, email, or private-key data');
  return actual;
}

export function validateStagingEvidence(value: unknown): StagingEvidence {
  if (!isRecord(value) || value.schemaVersion !== 1 || (value.status !== 'template' && value.status !== 'verified')
    || value.environment !== 'staging' || value.migrationThrough !== '202608110002'
    || value.walletE2EPassed !== true || value.executionGateMode !== 'wallet_staging'
    || value.postE2EGateMode !== 'wallet_staging' || value.cleanupPassed !== true
    || value.noChainTransaction !== true) throw new Error('wallet Staging E2E evidence requires wallet_staging execution/post-E2E gates and successful cleanup');
  return value as unknown as StagingEvidence;
}

function readReleaseEvidence(root: string, path?: string): ReleaseManifest {
  if (!path) return verifyReleaseArtifacts(resolve(root, 'dist')).manifest;
  const value = readExternalJson(path, root);
  assertReleaseManifest(value);
  return value;
}

function assertDeployedRelease(manifest: ReleaseManifest): void {
  if (!/^[0-9a-f]{40}$/.test(manifest.commitSha) || manifest.branch !== 'main' || manifest.buildContext !== 'cloudflare-pages') {
    throw new Error('release evidence must contain a 40-character commit, main branch, and cloudflare-pages context');
  }
}

function readRoleEvidence(root: string, path?: string): unknown {
  return path ? readExternalJson(path, root) : readRoleBootstrapPlan(resolve(root, 'operations/role-bootstrap-plan.example.json'));
}

function readRecoveryEvidence(root: string, path?: string): unknown {
  return path ? readExternalJson(path, root) : readRecoveryRehearsalEvidence(resolve(root, 'operations/recovery-rehearsal-evidence.example.json'));
}

function readStagingEvidence(root: string, path?: string): unknown {
  return path ? readExternalJson(path, root) : readJson(resolve(root, 'operations/staging-e2e-evidence.example.json'));
}

function readExternalJson(path: string, root: string): unknown { return JSON.parse(readFileSync(assertExternalEvidencePath(path, root), 'utf8')); }
function readJson(path: string): unknown { try { return JSON.parse(readFileSync(path, 'utf8')); } catch { throw new Error('evidence must be readable valid JSON'); } }
function isRecord(value: unknown): value is Record<string, unknown> { return typeof value === 'object' && value !== null && !Array.isArray(value); }
function message(error: unknown): string { return error instanceof Error ? error.message : String(error); }
function isInside(path: string, parent: string): boolean { const value = relative(parent, path); return value === '' || (!value.startsWith('..') && !value.includes(':')); }
function findGitRoot(root: string): string { let current = realpathSync.native(resolve(root)); while (!existsSync(resolve(current, '.git'))) { const parent = resolve(current, '..'); if (parent === current) throw new Error('Git working tree could not be resolved'); current = parent; } return current; }

function pathsFromEnvironment(environment: NodeJS.ProcessEnv): PilotEvidencePaths {
  return {
    release: environment.OPERATIONS_PILOT_RELEASE_EVIDENCE_PATH,
    roles: environment.OPERATIONS_PILOT_ROLE_PLAN_PATH,
    recovery: environment.OPERATIONS_PILOT_RECOVERY_EVIDENCE_PATH,
    stagingE2E: environment.OPERATIONS_PILOT_STAGING_E2E_EVIDENCE_PATH,
  };
}

function pathsFromArguments(args: string[]): PilotEvidencePaths {
  const paths: PilotEvidencePaths = {};
  for (let index = 0; index < args.length; index += 2) {
    const value = args[index + 1];
    if (!value) throw new Error('evidence option requires a path');
    if (args[index] === '--release-evidence') paths.release = value;
    else if (args[index] === '--role-plan') paths.roles = value;
    else if (args[index] === '--recovery-evidence') paths.recovery = value;
    else if (args[index] === '--staging-e2e-evidence') paths.stagingE2E = value;
    else throw new Error('unsupported pilot evidence option');
  }
  return paths;
}

if (process.argv[1]?.endsWith('operations-pilot-verify.ts')) {
  try {
    const paths = { ...pathsFromEnvironment(process.env), ...pathsFromArguments(process.argv.slice(2)) };
    const result = verifyPublicPilot(process.cwd(), paths);
    process.stdout.write(`${JSON.stringify(result)}\n`);
    process.stdout.write(`Public Pilot ${result.status}: ${result.reasons.length === 0 ? 'all five evidence classes passed' : result.reasons.join('; ')}\n`);
  } catch (error) {
    process.stderr.write(`Public Pilot verification failed: ${message(error)}\n`);
    process.exitCode = 1;
  }
}
