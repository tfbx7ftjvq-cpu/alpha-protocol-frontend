import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import {
  isMainModule,
} from './common.ts';
import {
  OPERATIONS_ROLE_NAMES,
  type OperationsRoleName,
} from './operations-roles.ts';

const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;
const TREASURY_ROLES = new Set<OperationsRoleName>([
  'treasury_preparer',
  'treasury_authorizer',
  'executor',
  'treasury_reconciler',
]);

export interface RoleBootstrapPlanEntry {
  userId: string;
  role: OperationsRoleName;
  responsibility: string;
  expiresAt: string;
  changeReference: string;
  staffIdentityAttestation: 'separate_staff_auth_user';
  existingWeb3User: false;
}

export interface RoleBootstrapPlan {
  schemaVersion: 1;
  environment: 'staging';
  purpose: 'public_pilot_candidate';
  treasuryRolesMustRemainUnassigned: boolean;
  entries: RoleBootstrapPlanEntry[];
}

export interface RoleBootstrapPlanResult {
  entries: number;
  treasuryRolesAssigned: number;
}

export function validateRoleBootstrapPlan(value: unknown): RoleBootstrapPlanResult {
  const plan = readPlan(value);
  const userIds = new Set<string>();
  const roles = new Set<OperationsRoleName>();
  const responsibilities = new Set<string>();
  let treasuryRolesAssigned = 0;

  for (const entry of plan.entries) {
    if (!UUID_PATTERN.test(entry.userId)) {
      throw new Error('role bootstrap plan userId must be a UUID');
    }
    if (userIds.has(entry.userId)) {
      throw new Error('role bootstrap plan requires one unique user per responsibility');
    }
    userIds.add(entry.userId);
    if (roles.has(entry.role)) {
      throw new Error('role bootstrap plan requires one unique responsibility per role');
    }
    roles.add(entry.role);
    if (!entry.responsibility || responsibilities.has(entry.responsibility)) {
      throw new Error('role bootstrap plan responsibility must be unique and non-empty');
    }
    responsibilities.add(entry.responsibility);
    if (!Number.isFinite(Date.parse(entry.expiresAt)) || Date.parse(entry.expiresAt) <= Date.now()) {
      throw new Error('role bootstrap plan expiresAt must be a future ISO timestamp');
    }
    if (!isChangeReference(entry.changeReference)) {
      throw new Error('role bootstrap plan changeReference must be 10 to 200 printable characters');
    }
    if (entry.staffIdentityAttestation !== 'separate_staff_auth_user' || entry.existingWeb3User !== false) {
      throw new Error('ordinary Web3 users cannot be silently upgraded to staff');
    }
    if (TREASURY_ROLES.has(entry.role)) {
      treasuryRolesAssigned += 1;
      if (plan.treasuryRolesMustRemainUnassigned) {
        throw new Error('treasury roles must remain unassigned for the public pilot candidate');
      }
    }
  }

  return { entries: plan.entries.length, treasuryRolesAssigned };
}

export function readRoleBootstrapPlan(path: string): unknown {
  try {
    return JSON.parse(readFileSync(path, 'utf8'));
  } catch {
    throw new Error('role bootstrap plan must be readable valid JSON');
  }
}

function readPlan(value: unknown): RoleBootstrapPlan {
  if (!isRecord(value) || value.schemaVersion !== 1 || value.environment !== 'staging'
    || value.purpose !== 'public_pilot_candidate'
    || typeof value.treasuryRolesMustRemainUnassigned !== 'boolean'
    || !Array.isArray(value.entries)) {
    throw new Error('role bootstrap plan schema is invalid');
  }
  return {
    schemaVersion: 1,
    environment: 'staging',
    purpose: 'public_pilot_candidate',
    treasuryRolesMustRemainUnassigned: value.treasuryRolesMustRemainUnassigned,
    entries: value.entries.map(readEntry),
  };
}

function readEntry(value: unknown): RoleBootstrapPlanEntry {
  if (!isRecord(value) || typeof value.userId !== 'string' || typeof value.role !== 'string'
    || !OPERATIONS_ROLE_NAMES.includes(value.role as OperationsRoleName)
    || typeof value.responsibility !== 'string' || typeof value.expiresAt !== 'string'
    || typeof value.changeReference !== 'string'
    || typeof value.staffIdentityAttestation !== 'string'
    || typeof value.existingWeb3User !== 'boolean') {
    throw new Error('role bootstrap plan entry is invalid');
  }
  return value as unknown as RoleBootstrapPlanEntry;
}

function isChangeReference(value: string): boolean {
  return value.trim().length >= 10 && value.trim().length <= 200
    && [...value].every((character) => {
      const codePoint = character.codePointAt(0) ?? 0;
      return codePoint > 31 && codePoint !== 127;
    });
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function main(): void {
  const path = resolve(process.cwd(), process.argv[2] ?? 'operations/role-bootstrap-plan.example.json');
  const result = validateRoleBootstrapPlan(readRoleBootstrapPlan(path));
  process.stdout.write(`${JSON.stringify({ status: 'VALID', ...result })}\n`);
}

if (isMainModule(import.meta.url)) {
  try {
    main();
  } catch (error) {
    process.stderr.write(`Role bootstrap plan invalid: ${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
