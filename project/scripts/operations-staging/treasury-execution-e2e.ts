import type { SupabaseClient } from '@supabase/supabase-js';

export interface TreasuryExecutionManifestFixture {
  pool: 'relief' | 'buyback' | 'builders' | 'staking';
  reliefApplicationId: string | null;
  assetMint: string;
  destinationWallet: string;
  amountBaseUnits: string;
  recipientVerificationReference: string;
  purposeReference: string;
}

export interface TreasuryExecutionStagingE2EInput {
  preparerClient: SupabaseClient;
  authorizerClient: SupabaseClient;
  executorClient: SupabaseClient;
  reconcilerClient: SupabaseClient;
  serviceRoleClient: SupabaseClient;
  ownerId: string;
  runReference: `phase-2e-6d-staging-e2e:${string}`;
  approvedDecisionIds: readonly [string, string];
  rejectedDecisionId: string;
  manifests: readonly [TreasuryExecutionManifestFixture, TreasuryExecutionManifestFixture];
  externallyObservedSignature: string;
  externallyObservedConfirmedAt: string;
}

/**
 * Future Staging-only verifier for an already-created governance fixture set.
 * It never signs, sends, simulates, or confirms a Solana transaction. The caller
 * must supply an externally observed signature, and cleanup uses only the
 * isolated service-role client after exact owner/purpose checks in the database.
 */
export async function runTreasuryExecutionRegistryStagingE2E(
  input: TreasuryExecutionStagingE2EInput,
): Promise<void> {
  const parametersFor = (manifest: TreasuryExecutionManifestFixture) => ({
    p_pool: manifest.pool,
    p_relief_application_id: manifest.reliefApplicationId,
    p_network: 'devnet',
    p_asset_symbol: 'USDC',
    p_asset_decimals: 6,
    p_asset_mint: manifest.assetMint,
    p_destination_wallet: manifest.destinationWallet,
    p_amount_base_units: manifest.amountBaseUnits,
    p_recipient_verification_reference: manifest.recipientVerificationReference,
    p_purpose_reference: manifest.purposeReference,
    p_private_note: 'Phase 2E-6D controlled Staging private verification fixture.',
  });
  const base = parametersFor(input.manifests[0]);
  const rpc = async (
    client: SupabaseClient,
    name: string,
    parameters: Record<string, unknown>,
  ): Promise<Record<string, unknown>> => {
    const { data, error } = await client.rpc(name, parameters);
    if (error) throw new Error(`${name} failed: ${error.message}`);
    const row = Array.isArray(data) ? data[0] : data;
    if (!row || typeof row !== 'object') throw new Error(`${name} returned no row`);
    return row as Record<string, unknown>;
  };
  const expectRejected = async (operation: PromiseLike<unknown>, label: string): Promise<void> => {
    const result = await operation as { error?: unknown };
    if (!result?.error) throw new Error(`${label} unexpectedly succeeded`);
  };

  await expectRejected(
    input.preparerClient.rpc('prepare_treasury_execution_intent_v1', {
      ...base,
      p_governance_decision_id: input.rejectedDecisionId,
      p_audit_reference: `${input.runReference}:rejected-decision`,
    }),
    'rejected decision preparation',
  );
  await expectRejected(
    input.preparerClient.rpc('prepare_treasury_execution_intent_v1', {
      ...base,
      p_governance_decision_id: input.approvedDecisionIds[0],
      p_amount_base_units: (BigInt(input.manifests[0].amountBaseUnits) + 1n).toString(),
      p_audit_reference: `${input.runReference}:manifest-mismatch`,
    }),
    'manifest mismatch preparation',
  );

  const prepared = await rpc(input.preparerClient, 'prepare_treasury_execution_intent_v1', {
    ...base,
    p_governance_decision_id: input.approvedDecisionIds[0],
    p_audit_reference: `${input.runReference}:prepared`,
  });
  const firstIntentId = String(prepared.execution_intent_id);
  await rpc(input.authorizerClient, 'authorize_treasury_execution_intent_v1', {
    p_execution_intent_id: firstIntentId,
    p_authorization_reference: `${input.runReference}:authorization`,
    p_private_note: 'Phase 2E-6D controlled Staging private verification fixture.',
    p_audit_reference: `${input.runReference}:authorized`,
  });
  const beforeReport = await input.serviceRoleClient
    .from('treasury_execution_receipts').select('id', { count: 'exact', head: true })
    .eq('execution_intent_id', firstIntentId);
  if (beforeReport.error || beforeReport.count !== 0) {
    throw new Error('authorization created a receipt or receipt count could not be verified');
  }
  await expectRejected(
    input.executorClient.rpc('report_treasury_execution_receipt_v1', {
      p_execution_intent_id: firstIntentId,
      p_transaction_signature: 'invalid-signature',
      p_confirmed_at: input.externallyObservedConfirmedAt,
      p_private_note: base.p_private_note,
      p_audit_reference: `${input.runReference}:invalid-signature`,
    }),
    'invalid signature report',
  );
  await rpc(input.executorClient, 'report_treasury_execution_receipt_v1', {
    p_execution_intent_id: firstIntentId,
    p_transaction_signature: input.externallyObservedSignature,
    p_confirmed_at: input.externallyObservedConfirmedAt,
    p_private_note: base.p_private_note,
    p_audit_reference: `${input.runReference}:reported`,
  });
  await expectRejected(
    input.executorClient.rpc('report_treasury_execution_receipt_v1', {
      p_execution_intent_id: firstIntentId,
      p_transaction_signature: input.externallyObservedSignature,
      p_confirmed_at: input.externallyObservedConfirmedAt,
      p_private_note: base.p_private_note,
      p_audit_reference: `${input.runReference}:duplicate-report`,
    }),
    'duplicate receipt report',
  );
  await rpc(input.reconcilerClient, 'reconcile_treasury_execution_v1', {
    p_execution_intent_id: firstIntentId,
    p_outcome: 'reconciled',
    p_reconciliation_reference: `${input.runReference}:reconciliation`,
    p_private_note: base.p_private_note,
    p_audit_reference: `${input.runReference}:reconciled`,
  });

  const second = await rpc(input.preparerClient, 'prepare_treasury_execution_intent_v1', {
    ...parametersFor(input.manifests[1]),
    p_governance_decision_id: input.approvedDecisionIds[1],
    p_audit_reference: `${input.runReference}:cancel-fixture-prepared`,
  });
  const secondIntentId = String(second.execution_intent_id);
  await rpc(input.authorizerClient, 'cancel_treasury_execution_intent_v1', {
    p_execution_intent_id: secondIntentId,
    p_cancellation_reference: `${input.runReference}:cancellation`,
    p_private_note: base.p_private_note,
    p_audit_reference: `${input.runReference}:cancelled`,
  });

  const cleanup = await rpc(input.serviceRoleClient, 'cleanup_treasury_execution_staging_e2e_v1', {
    p_run_reference: input.runReference,
    p_fixture_owner_id: input.ownerId,
    p_execution_intent_ids: [firstIntentId, secondIntentId],
  });
  if (cleanup.intents_deleted !== 2) throw new Error('treasury execution cleanup was not exact');
}
