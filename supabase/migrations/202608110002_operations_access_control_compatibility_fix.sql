begin;

-- Phase 2E-6E compatibility fix only.
-- 202608110001 is already applied on Staging; keep this as a forward-only
-- correction for schema lint/runtime compatibility.

create or replace function public.finalize_governance_decision_v1(
  p_proposal_id uuid,
  p_decision text,
  p_rationale text,
  p_execution_manifest_sha256 text,
  p_finalization_reference text
)
returns table (
  governance_decision_id uuid,
  decision_hash text,
  execution_required boolean,
  execution_intent_created boolean,
  execution_receipt_created boolean
)
language plpgsql
security definer
set search_path = public, pg_temp
as $$
declare
  v_actor_id uuid := auth.uid();
  v_actor_role text := public.current_operations_role_v1();
  v_proposal public.governance_proposals%rowtype;
  v_submission public.governance_proposal_submissions%rowtype;
  v_hash text := lower(nullif(btrim(p_execution_manifest_sha256), ''));
  v_decision_hash text;
  v_decision_id uuid;
begin
  if v_actor_id is null or v_actor_role is null or v_actor_role <> 'governance_admin' then
    raise exception 'operations role is not authorized to finalize governance decisions';
  end if;
  if p_decision is null or p_decision not in ('approved', 'rejected', 'cancelled') then
    raise exception 'invalid governance decision';
  end if;
  if p_finalization_reference is null or char_length(btrim(p_finalization_reference)) not between 10 and 200
    or p_finalization_reference ~ '[[:cntrl:]]' then raise exception 'invalid finalization reference'; end if;
  select * into v_proposal from public.governance_proposals where id = p_proposal_id for update;
  if not found or v_proposal.publication_status <> 'published' or v_proposal.status not in ('discussion', 'voting') then
    raise exception 'governance proposal is not finalizable';
  end if;
  select * into v_submission from public.governance_proposal_submissions
  where published_proposal_id = v_proposal.id;
  if not found then raise exception 'governance proposal has no audited private source binding'; end if;
  if v_submission.submitted_by = v_actor_id or v_submission.reviewed_by = v_actor_id then
    raise exception 'decision finalizer must be independent from proposal submission and publication';
  end if;
  if v_proposal.execution_required then
    if v_hash is distinct from v_submission.execution_manifest_sha256 then
      raise exception 'decision execution manifest sha256 does not match the audited proposal';
    end if;
  elsif v_hash is not null then
    raise exception 'non-execution decision cannot bind an execution manifest';
  end if;

  v_decision_hash := encode(sha256(convert_to(concat_ws(E'\n',
    'alpha-governance-decision-v1', v_proposal.id::text, p_decision,
    btrim(p_rationale), v_proposal.execution_required::text, coalesce(v_hash, '')
  ), 'utf8')), 'hex');

  if v_proposal.status = 'discussion' then
    update public.governance_proposals set status = 'voting' where id = v_proposal.id;
  end if;
  update public.governance_proposals set status = 'decided' where id = v_proposal.id;
  insert into public.governance_decisions (
    proposal_id, decision, rationale, decision_hash, execution_required,
    execution_reference, execution_manifest_sha256, finalization_reference,
    publication_status, decided_at
  ) values (
    v_proposal.id, p_decision, btrim(p_rationale), v_decision_hash, v_proposal.execution_required,
    case when v_proposal.execution_required then v_hash else null end,
    case when v_proposal.execution_required then v_hash else null end, btrim(p_finalization_reference),
    'published', timezone('utc', now())
  ) returning id into v_decision_id;

  insert into public.operations_governance_workflow_events (
    entity_type, private_entity_id, public_entity_id, action, actor_id,
    actor_role, event_reference, event_payload
  ) values (
    'decision', v_submission.id, v_decision_id, 'decision_finalized',
    v_actor_id, v_actor_role, btrim(p_finalization_reference),
    jsonb_build_object(
      'decision', p_decision,
      'decision_hash', v_decision_hash,
      'execution_required', v_proposal.execution_required,
      'execution_manifest_sha256', case when v_proposal.execution_required then v_hash else null end,
      'execution_intent_created', false,
      'execution_receipt_created', false
    )
  );
  return query select v_decision_id, v_decision_hash, v_proposal.execution_required, false, false;
end;
$$;

comment on function public.finalize_governance_decision_v1(uuid,text,text,text,text) is
  'Finalizes an immutable off-chain governance decision with deterministic SHA-256 manifest binding. Approval is not execution or payment and creates no intent or receipt.';

revoke all on function public.finalize_governance_decision_v1(uuid,text,text,text,text) from public, anon, authenticated, service_role;
grant execute on function public.finalize_governance_decision_v1(uuid,text,text,text,text) to authenticated;

create or replace function public.prepare_treasury_execution_intent_v1(
  p_governance_decision_id uuid,
  p_pool text,
  p_relief_application_id uuid,
  p_network text,
  p_asset_symbol text,
  p_asset_decimals smallint,
  p_asset_mint text,
  p_destination_wallet text,
  p_amount_base_units numeric,
  p_recipient_verification_reference text,
  p_purpose_reference text,
  p_private_note text,
  p_audit_reference text
)
returns table (execution_intent_id uuid, intent_hash text, status text, transaction_sent boolean, receipt_created boolean)
language plpgsql
security definer
set search_path = public, pg_temp
as $$
declare
  v_actor_id uuid := auth.uid();
  v_actor_role text := public.current_operations_role_v1();
  v_decision public.governance_decisions%rowtype;
  v_submission public.governance_proposal_submissions%rowtype;
  v_manifest jsonb;
  v_intent_id uuid;
  v_intent_hash text;
  v_decision_finalizer uuid;
begin
  if v_actor_id is null or v_actor_role is null or v_actor_role <> 'treasury_preparer' then
    raise exception 'operations role is not authorized to prepare treasury execution intents';
  end if;
  if p_audit_reference is null or char_length(btrim(p_audit_reference)) not between 10 and 200 or p_audit_reference ~ '[[:cntrl:]]' then
    raise exception 'invalid intent preparation audit reference';
  end if;
  if p_private_note is null or char_length(btrim(p_private_note)) not between 10 and 5000 then
    raise exception 'private preparation note is required';
  end if;
  if p_purpose_reference is null or char_length(btrim(p_purpose_reference)) not between 10 and 200 or p_purpose_reference ~ '[[:cntrl:]]' then
    raise exception 'invalid execution purpose reference';
  end if;
  if p_recipient_verification_reference is null or char_length(btrim(p_recipient_verification_reference)) not between 12 and 2000 then
    raise exception 'recipient verification reference is required';
  end if;
  if p_network not in ('devnet', 'mainnet-beta') or p_asset_symbol <> 'USDC' or p_asset_decimals <> 6
    or p_amount_base_units is null or p_amount_base_units <= 0 or trunc(p_amount_base_units) <> p_amount_base_units
    or not public.is_base58_bytes_v1(p_asset_mint, 32)
    or not public.is_base58_bytes_v1(p_destination_wallet, 32) then
    raise exception 'invalid network, asset, amount, mint, or recipient wallet';
  end if;

  select * into v_decision from public.governance_decisions
  where id = p_governance_decision_id for share;
  if not found or v_decision.decision <> 'approved' or not v_decision.execution_required
    or v_decision.decision_hash !~ '^[0-9a-f]{64}$'
    or v_decision.execution_manifest_sha256 !~ '^[0-9a-f]{64}$' then
    raise exception 'only an approved execution-required deterministic governance decision can prepare an intent';
  end if;
  if exists (select 1 from public.treasury_execution_intents where governance_decision_id = v_decision.id) then
    raise exception 'governance decision already has an execution intent';
  end if;
  select * into v_submission from public.governance_proposal_submissions
  where published_proposal_id = v_decision.proposal_id and review_status = 'published';
  if not found or v_submission.execution_manifest_sha256 is distinct from v_decision.execution_manifest_sha256 then
    raise exception 'governance decision has no matching immutable private manifest';
  end if;
  v_manifest := v_submission.private_execution_manifest;
  if jsonb_typeof(v_manifest) <> 'object'
    or v_manifest ->> 'pool' is distinct from p_pool
    or v_manifest ->> 'network' is distinct from p_network
    or v_manifest ->> 'asset_symbol' is distinct from p_asset_symbol
    or (v_manifest ->> 'asset_decimals')::smallint is distinct from p_asset_decimals
    or v_manifest ->> 'asset_mint' is distinct from p_asset_mint
    or v_manifest ->> 'destination_wallet' is distinct from p_destination_wallet
    or (v_manifest ->> 'amount_base_units')::numeric is distinct from p_amount_base_units
    or v_manifest ->> 'recipient_verification_reference' is distinct from p_recipient_verification_reference
    or v_manifest ->> 'purpose_reference' is distinct from p_purpose_reference
    or coalesce(nullif(v_manifest ->> 'relief_application_id', ''), null) is distinct from coalesce(p_relief_application_id::text, null)
  then
    raise exception 'execution intent must exactly match the audited governance manifest';
  end if;
  select event.actor_id into v_decision_finalizer from public.operations_governance_workflow_events event
  where event.action = 'decision_finalized' and event.public_entity_id = v_decision.id;
  if v_decision_finalizer = v_actor_id then
    raise exception 'treasury intent preparer must be independent from the governance decision finalizer';
  end if;
  if p_pool = 'relief' then
    if p_relief_application_id is null then raise exception 'relief pool intent requires a relief application'; end if;
    if not exists (
      select 1 from public.relief_applications application
      where application.id = p_relief_application_id and application.status = 'approved'
        and application.payment_receipt_id is null
        and application.wallet_address = p_destination_wallet
        and p_amount_base_units <= application.requested_amount_usdc * 1000000
    ) then raise exception 'relief execution intent must match one approved unpaid relief application'; end if;
  elsif p_relief_application_id is not null then
    raise exception 'non-relief execution intent cannot bind a relief application';
  end if;
  v_intent_hash := encode(sha256(convert_to(concat_ws(E'\n',
    'alpha-treasury-execution-intent-v1', v_decision.id::text, v_decision.decision_hash,
    v_decision.execution_manifest_sha256, p_pool, coalesce(p_relief_application_id::text, ''),
    p_network, p_asset_symbol, p_asset_decimals::text, p_asset_mint, p_destination_wallet,
    p_amount_base_units::text, btrim(p_recipient_verification_reference), btrim(p_purpose_reference)
  ), 'utf8')), 'hex');
  insert into public.treasury_execution_intents (
    governance_decision_id, relief_application_id, decision_hash, manifest_sha256, intent_hash, pool,
    network, asset_symbol, asset_decimals, asset_mint, destination_wallet, amount_base_units,
    recipient_verification_reference, purpose_reference, status, prepared_by
  ) values (
    v_decision.id, p_relief_application_id, v_decision.decision_hash, v_decision.execution_manifest_sha256, v_intent_hash, p_pool,
    p_network, p_asset_symbol, p_asset_decimals, p_asset_mint, p_destination_wallet, p_amount_base_units,
    btrim(p_recipient_verification_reference), btrim(p_purpose_reference), 'prepared', v_actor_id
  ) returning id into v_intent_id;
  insert into public.treasury_execution_private_notes values (
    default, v_intent_id, 'preparation', btrim(p_private_note), v_actor_id, timezone('utc', now())
  );
  insert into public.operations_treasury_execution_workflow_events (
    execution_intent_id, governance_decision_id, action, previous_status, new_status, actor_id,
    actor_role, audit_reference, decision_hash, manifest_sha256, intent_hash, event_payload
  ) values (
    v_intent_id, v_decision.id, 'intent_prepared', null, 'prepared', v_actor_id,
    v_actor_role, btrim(p_audit_reference), v_decision.decision_hash, v_decision.execution_manifest_sha256, v_intent_hash,
    jsonb_build_object('transaction_sent', false, 'receipt_created', false)
  );
  insert into public.treasury_execution_public_registry (
    intent_public_id, governance_decision_id, decision_hash, manifest_sha256, intent_hash, purpose_reference,
    asset_symbol, asset_decimals, asset_mint, destination_wallet_display, amount_base_units, network,
    public_status, prepared_at
  ) values (
    v_intent_id, v_decision.id, v_decision.decision_hash, v_decision.execution_manifest_sha256, v_intent_hash, btrim(p_purpose_reference),
    p_asset_symbol, p_asset_decimals, p_asset_mint, left(p_destination_wallet, 4) || '…' || right(p_destination_wallet, 4), p_amount_base_units, p_network,
    'prepared', timezone('utc', now())
  );
  return query select v_intent_id, v_intent_hash, 'prepared'::text, false, false;
end;
$$;

comment on function public.prepare_treasury_execution_intent_v1(uuid,text,uuid,text,text,smallint,text,text,numeric,text,text,text,text) is
  'Prepares an audited treasury execution intent only. Preparation sends no transaction and creates no receipt.';

revoke all on function public.prepare_treasury_execution_intent_v1(uuid,text,uuid,text,text,smallint,text,text,numeric,text,text,text,text) from public, anon, authenticated, service_role;
grant execute on function public.prepare_treasury_execution_intent_v1(uuid,text,uuid,text,text,smallint,text,text,numeric,text,text,text,text) to authenticated;

commit;
