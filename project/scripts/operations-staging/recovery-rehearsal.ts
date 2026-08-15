import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { isMainModule } from './common.ts';

export const CURRENT_STAGING_PROJECT_REF = 'neevswvhndkalxkainxo';
const PROJECT_REF = /^[a-z0-9]{20}$/;

export interface RecoveryRehearsalEvidence {
  schemaVersion: 1;
  status: 'template' | 'verified';
  target: { classification: 'isolated_restore'; projectRef: string; environment: 'isolated_restore' };
  backupInventory: { method: 'dashboard_pitr' | 'pg_dump'; reviewedAt: string };
  migrationParity: { through: '202608110002'; verified: boolean };
  rls: { publicReadTables: 7; privateAnonTablesDenied: 7 };
  gateMode: 'disabled';
  noMutationAttestation: { users: false; roles: false; funds: false; chain: false };
}

export function validateRecoveryRehearsalEvidence(value: unknown): RecoveryRehearsalEvidence {
  if (!isRecord(value) || value.schemaVersion !== 1
    || (value.status !== 'template' && value.status !== 'verified')
    || !isRecord(value.target) || value.target.classification !== 'isolated_restore'
    || value.target.environment !== 'isolated_restore' || typeof value.target.projectRef !== 'string'
    || !PROJECT_REF.test(value.target.projectRef) || value.target.projectRef === CURRENT_STAGING_PROJECT_REF
    || !isRecord(value.backupInventory)
    || (value.backupInventory.method !== 'dashboard_pitr' && value.backupInventory.method !== 'pg_dump')
    || typeof value.backupInventory.reviewedAt !== 'string' || !Number.isFinite(Date.parse(value.backupInventory.reviewedAt))
    || !isRecord(value.migrationParity) || value.migrationParity.through !== '202608110002' || value.migrationParity.verified !== true
    || !isRecord(value.rls) || value.rls.publicReadTables !== 7 || value.rls.privateAnonTablesDenied !== 7
    || value.gateMode !== 'disabled' || !isRecord(value.noMutationAttestation)
    || value.noMutationAttestation.users !== false || value.noMutationAttestation.roles !== false
    || value.noMutationAttestation.funds !== false || value.noMutationAttestation.chain !== false) {
    throw new Error('recovery rehearsal evidence is invalid or not isolated');
  }
  return value as unknown as RecoveryRehearsalEvidence;
}

export function readRecoveryRehearsalEvidence(path: string): unknown {
  try { return JSON.parse(readFileSync(path, 'utf8')); } catch { throw new Error('recovery rehearsal evidence must be valid JSON'); }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

if (isMainModule(import.meta.url)) {
  try {
    const path = resolve(process.cwd(), process.argv[2] ?? 'operations/recovery-rehearsal-evidence.example.json');
    const evidence = validateRecoveryRehearsalEvidence(readRecoveryRehearsalEvidence(path));
    process.stdout.write(`${JSON.stringify({ status: evidence.status, target: evidence.target.classification, gateMode: evidence.gateMode })}\n`);
  } catch (error) {
    process.stderr.write(`Recovery rehearsal validation failed: ${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
