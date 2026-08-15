import { createClient } from '@supabase/supabase-js';
import {
  assertStagingSecretIsolation,
  isMainModule,
  loadOperationsStagingEnvironment,
  resolveOperationsStagingConfig,
  sanitizeStagingError,
  type OperationsStagingConfig,
  type OperationsStagingMode,
} from './common.ts';

export type OperationsIntakeGateAction = 'inspect' | 'activate' | 'disable';
export type OperationsIntakeGateMode = 'disabled' | 'wallet_staging';

export interface OperationsIntakeGateEvent {
  eventId: number;
  previousMode: OperationsIntakeGateMode;
  newMode: OperationsIntakeGateMode;
  changeReference: string;
  changedAt: string;
}

export interface OperationsIntakeGateSnapshot {
  mode: OperationsIntakeGateMode;
  activationReference: string | null;
  updatedAt: string;
  recentEvents: OperationsIntakeGateEvent[];
}

export interface OperationsIntakeGateResult {
  projectRef: string;
  action: OperationsIntakeGateAction;
  changed: boolean;
  snapshot: OperationsIntakeGateSnapshot;
}

export async function runOperationsIntakeGateAction(
  config: OperationsStagingConfig,
  action: OperationsIntakeGateAction,
): Promise<OperationsIntakeGateResult> {
  if (!config.serviceRoleKey || config.mode !== modeForAction(action)) {
    throw new Error('staging intake gate 工具配置与操作不一致');
  }

  const client = createOperationsGateClient(config.supabaseUrl, config.serviceRoleKey);
  let changed = false;
  let transitionEventId: number | null = null;

  if (action !== 'inspect') {
    if (!config.confirmedForGateChange || !config.gateChangeReference) {
      throw new Error('staging intake gate 变更缺少精确确认或审计引用');
    }

    const targetMode: OperationsIntakeGateMode = action === 'activate'
      ? 'wallet_staging'
      : 'disabled';
    const changeResult = await client.rpc('set_operations_wallet_intake_mode', {
      p_requested_mode: targetMode,
      p_change_reference: config.gateChangeReference,
    });
    if (changeResult.error) {
      throw new Error(`intake gate ${action} RPC failed: ${changeResult.error.message}`);
    }

    const changedRow = readSingleRow(changeResult.data, 'intake gate change');
    if (readGateMode(changedRow.mode, 'changed mode') !== targetMode) {
      throw new Error('intake gate change RPC returned an invalid transition receipt');
    }
    transitionEventId = readPositiveSafeInteger(
      changedRow.event_id,
      'intake gate transition event_id',
    );
    changed = true;
  }

  const snapshot = await inspectOperationsIntakeGate(client);
  const expectedMode = action === 'activate'
    ? 'wallet_staging'
    : action === 'disable'
      ? 'disabled'
      : null;
  if (expectedMode && snapshot.mode !== expectedMode) {
    throw new Error(`intake gate verification failed: expected ${expectedMode}`);
  }
  if (changed) {
    const latestEvent = snapshot.recentEvents[0];
    const expectedPreviousMode: OperationsIntakeGateMode = action === 'activate'
      ? 'disabled'
      : 'wallet_staging';
    const expectedActivationReference = action === 'activate'
      ? config.gateChangeReference
      : null;
    if (
      !latestEvent
      || latestEvent.eventId !== transitionEventId
      || latestEvent.previousMode !== expectedPreviousMode
      || latestEvent.newMode !== expectedMode
      || latestEvent.changeReference !== config.gateChangeReference
      || snapshot.activationReference !== expectedActivationReference
    ) {
      throw new Error('intake gate audit event does not match the requested transition receipt');
    }
  }

  return {
    projectRef: config.projectRef,
    action,
    changed,
    snapshot,
  };
}

function createOperationsGateClient(supabaseUrl: string, serviceRoleKey: string) {
  return createClient(supabaseUrl, serviceRoleKey, {
    auth: {
      autoRefreshToken: false,
      detectSessionInUrl: false,
      persistSession: false,
    },
    global: {
      fetch: (input, init) => fetch(input, { ...init, redirect: 'error' }),
    },
  });
}

async function inspectOperationsIntakeGate(
  client: ReturnType<typeof createOperationsGateClient>,
): Promise<OperationsIntakeGateSnapshot> {
  const [controlResult, eventsResult] = await Promise.all([
    client
      .from('operations_intake_control')
      .select('mode,activation_reference,updated_at')
      .eq('singleton', true)
      .single(),
    client
      .from('operations_intake_gate_events')
      .select('event_id,previous_mode,new_mode,change_reference,changed_at')
      .order('event_id', { ascending: false })
      .limit(10),
  ]);

  if (controlResult.error) {
    throw new Error(`intake gate control read failed: ${controlResult.error.message}`);
  }
  if (eventsResult.error) {
    throw new Error(`intake gate audit read failed: ${eventsResult.error.message}`);
  }

  const control = readObject(controlResult.data, 'intake gate control');
  const mode = readGateMode(control.mode, 'intake gate control mode');
  const activationReference = control.activation_reference === null
    ? null
    : readString(control.activation_reference, 'activation reference');
  const updatedAt = readTimestamp(control.updated_at, 'intake gate updated_at');
  if (
    (mode === 'disabled' && activationReference !== null)
    || (mode === 'wallet_staging' && activationReference === null)
  ) {
    throw new Error('intake gate control row violates its activation-reference invariant');
  }

  const recentEvents = readArray(eventsResult.data, 'intake gate audit events')
    .map((value): OperationsIntakeGateEvent => {
      const event = readObject(value, 'intake gate audit event');
      const eventId = readPositiveSafeInteger(
        event.event_id,
        'intake gate audit event_id',
      );
      const previousMode = readGateMode(event.previous_mode, 'previous gate mode');
      const newMode = readGateMode(event.new_mode, 'new gate mode');
      if (previousMode === newMode) {
        throw new Error('intake gate audit event does not describe a transition');
      }

      return {
        eventId,
        previousMode,
        newMode,
        changeReference: readGateReference(event.change_reference),
        changedAt: readTimestamp(event.changed_at, 'gate changed_at'),
      };
    });

  for (const [index, event] of recentEvents.entries()) {
    const olderEvent = recentEvents[index + 1];
    if (olderEvent && (
      event.eventId <= olderEvent.eventId
      || event.previousMode !== olderEvent.newMode
    )) {
      throw new Error('intake gate audit history is not a continuous descending chain');
    }
  }
  if (recentEvents[0] && recentEvents[0].newMode !== mode) {
    throw new Error('intake gate control mode does not match the latest audit event');
  }

  return { mode, activationReference, updatedAt, recentEvents };
}

function modeForAction(action: OperationsIntakeGateAction): OperationsStagingMode {
  return action === 'inspect'
    ? 'gate-inspect'
    : action === 'activate'
      ? 'gate-activate'
      : 'gate-disable';
}

function readSingleRow(value: unknown, label: string): Record<string, unknown> {
  const rows = readArray(value, label);
  if (rows.length !== 1) {
    throw new Error(`${label} must return exactly one row`);
  }
  return readObject(rows[0], label);
}

function readArray(value: unknown, label: string): unknown[] {
  if (!Array.isArray(value)) {
    throw new Error(`${label} did not return an array`);
  }
  return value;
}

function readObject(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw new Error(`${label} did not return an object`);
  }
  return value as Record<string, unknown>;
}

function readGateMode(value: unknown, label: string): OperationsIntakeGateMode {
  if (value !== 'disabled' && value !== 'wallet_staging') {
    throw new Error(`${label} is invalid`);
  }
  return value;
}

function readString(value: unknown, label: string): string {
  if (typeof value !== 'string' || value.length === 0) {
    throw new Error(`${label} is invalid`);
  }
  return value;
}

function readPositiveSafeInteger(value: unknown, label: string): number {
  if (!Number.isSafeInteger(value) || Number(value) <= 0) {
    throw new Error(`${label} is invalid`);
  }
  return Number(value);
}

function readGateReference(value: unknown): string {
  const reference = readString(value, 'gate change reference');
  if (
    reference !== reference.trim()
    || reference.length < 10
    || reference.length > 200
    || [...reference].some((character) => {
      const codePoint = character.codePointAt(0) ?? 0;
      return codePoint <= 31 || codePoint === 127;
    })
  ) {
    throw new Error('gate change reference is invalid');
  }
  return reference;
}

function readTimestamp(value: unknown, label: string): string {
  const timestamp = readString(value, label);
  if (Number.isNaN(Date.parse(timestamp))) {
    throw new Error(`${label} is not a valid timestamp`);
  }
  return timestamp;
}

function parseAction(value: string | undefined): OperationsIntakeGateAction {
  if (value === 'inspect' || value === 'activate' || value === 'disable') {
    return value;
  }
  throw new Error('intake gate action must be inspect, activate, or disable');
}

async function main(): Promise<void> {
  const action = parseAction(process.argv[2]);
  const loaded = loadOperationsStagingEnvironment();
  assertStagingSecretIsolation(loaded.projectDirectory, loaded.envFile);
  const config = resolveOperationsStagingConfig(loaded.env, modeForAction(action));

  try {
    const result = await runOperationsIntakeGateAction(config, action);
    console.log('Operations staging intake gate check passed.');
    console.log(`Project ref: ${result.projectRef}`);
    console.log(`Action: ${result.action}`);
    console.log(`Changed: ${result.changed ? 'yes' : 'no'}`);
    console.log(`Mode: ${result.snapshot.mode}`);
    console.log(`Updated at: ${result.snapshot.updatedAt}`);
    console.log(`Audit events returned: ${result.snapshot.recentEvents.length}`);
    if (!result.changed) {
      console.log('No rows were inserted, updated, or deleted.');
    }
    console.log('No Solana transaction or treasury action was performed.');
  } catch (error) {
    throw new Error(sanitizeStagingError(error, config));
  }
}

if (isMainModule(import.meta.url)) {
  main().catch((error: unknown) => {
    console.error(`Operations staging intake gate failed: ${sanitizeStagingError(error)}`);
    process.exitCode = 1;
  });
}
