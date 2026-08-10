-- Alpha Protocol Phase 2E-6D: audited treasury execution registry and reconciliation.
-- Registry only: this migration has no network function, signer, broadcaster, payment
-- authority, balance mutation, Solana RPC sender, or automatic intent/receipt creation.
--
-- Read-only pre-application inventory (run manually before applying):
-- select id, governance_decision_id, status, manifest_sha256, created_at
-- from public.treasury_execution_intents order by created_at;
-- select id, execution_intent_id, governance_decision_id, transaction_signature, recorded_at
-- from public.treasury_execution_receipts order by recorded_at;

do $$
begin
  if exists (select 1 from public.treasury_execution_intents)
    or exists (select 1 from public.treasury_execution_receipts) then
    raise exception using
      message = 'Phase 2E-6D refusing unsafe in-place migration: legacy treasury execution tables must be empty',
      detail = 'Inventory and manually migrate legacy rows; do not synthesize hashes, recipient verification, authorization, or reconciliation evidence.',
      hint = 'Run the read-only inventory queries in the migration header before retrying.';
  end if;
end;
$$;

create or replace function public.is_base58_bytes_v1(value text, expected_bytes integer)
returns boolean
language plpgsql
immutable
strict
set search_path = public, pg_temp
as $$
declare
  alphabet constant text := '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
  numeric_value numeric := 0;
  leading_zero_bytes integer := 0;
  encoded_bytes integer := 0;
  index_value integer;
  character_value text;
begin
  if expected_bytes <= 0 or value = '' then return false; end if;
  for index_value in 1..char_length(value) loop
    character_value := substr(value, index_value, 1);
    if strpos(alphabet, character_value) = 0 then return false; end if;
    if character_value = '1' and numeric_value = 0 and encoded_bytes = 0 then
      leading_zero_bytes := leading_zero_bytes + 1;
    end if;
    numeric_value := numeric_value * 58 + strpos(alphabet, character_value) - 1;
  end loop;
  while numeric_value > 0 loop
    encoded_bytes := encoded_bytes + 1;
    numeric_value := trunc(numeric_value / 256);
  end loop;
  return leading_zero_bytes + encoded_bytes = expected_bytes;
end;
$$;
revoke all on function public.is_base58_bytes_v1(text, integer) from public;

alter table public.treasury_execution_intents
  drop constraint treasury_execution_intents_status_check,
  drop constraint treasury_execution_intent_signature_state,
  add column decision_hash text not null check (decision_hash ~ '^[0-9a-f]{64}$'),
  add column intent_hash text not null unique check (intent_hash ~ '^[0-9a-f]{64}$'),
  add column purpose_reference text not null check (
    char_length(purpose_reference) between 10 and 200
    and purpose_reference !~ '[[:cntrl:]]'
  ),
  add column authorized_by uuid references auth.users(id),
  add column authorization_reference text check (
    authorization_reference is null
    or (char_length(authorization_reference) between 10 and 200 and authorization_reference !~ '[[:cntrl:]]')
  ),
  add column authorized_at timestamptz,
  add column reported_by uuid references auth.users(id),
  add column reported_at timestamptz,
  add column reconciled_by uuid references auth.users(id),
  add column reconciliation_reference text check (
    reconciliation_reference is null
    or (char_length(reconciliation_reference) between 10 and 200 and reconciliation_reference !~ '[[:cntrl:]]')
  ),
  add column reconciled_at timestamptz,
  add column cancelled_by uuid references auth.users(id),
  add column cancellation_reference text check (
    cancellation_reference is null
    or (char_length(cancellation_reference) between 10 and 200 and cancellation_reference !~ '[[:cntrl:]]')
  ),
  add column cancelled_at timestamptz,
  add constraint treasury_execution_intents_status_check check (
    status in ('prepared', 'authorized', 'reported', 'reconciled', 'cancelled', 'failed')
  ),
  add constraint treasury_execution_intent_signature_state check (
    (status in ('prepared', 'authorized', 'cancelled') and submitted_signature is null)
    or (status in ('reported', 'reconciled', 'failed') and submitted_signature is not null)
  ),
  add constraint treasury_execution_intent_actor_state check (
    (status = 'prepared'
      and authorized_by is null and authorization_reference is null and authorized_at is null
      and reported_by is null and reported_at is null
      and reconciled_by is null and reconciliation_reference is null and reconciled_at is null
      and cancelled_by is null and cancellation_reference is null and cancelled_at is null)
    or (status = 'authorized'
      and authorized_by is not null and authorization_reference is not null and authorized_at is not null
      and reported_by is null and reported_at is null
      and reconciled_by is null and reconciliation_reference is null and reconciled_at is null
      and cancelled_by is null and cancellation_reference is null and cancelled_at is null)
    or (status = 'reported'
      and authorized_by is not null and authorization_reference is not null and authorized_at is not null
      and reported_by is not null and reported_at is not null
      and reconciled_by is null and reconciliation_reference is null and reconciled_at is null
      and cancelled_by is null and cancellation_reference is null and cancelled_at is null)
    or (status in ('reconciled', 'failed')
      and authorized_by is not null and authorization_reference is not null and authorized_at is not null
      and reported_by is not null and reported_at is not null
      and reconciled_by is not null and reconciliation_reference is not null and reconciled_at is not null
      and cancelled_by is null and cancellation_reference is null and cancelled_at is null)
    or (status = 'cancelled'
      and reported_by is null and reported_at is null
      and reconciled_by is null and reconciliation_reference is null and reconciled_at is null
      and cancelled_by is not null and cancellation_reference is not null and cancelled_at is not null)
  );

alter table public.treasury_execution_receipts
  add column decision_hash text not null check (decision_hash ~ '^[0-9a-f]{64}$'),
  add column intent_hash text not null check (intent_hash ~ '^[0-9a-f]{64}$'),
  add column asset_symbol text not null check (asset_symbol = 'USDC'),
  add column asset_decimals smallint not null check (asset_decimals = 6),
  add column asset_mint text not null,
  add column destination_wallet text not null,
  add column amount_base_units numeric(30, 0) not null check (amount_base_units > 0),
  add column recorded_by uuid not null references auth.users(id),
  add constraint treasury_execution_receipt_asset_mint_key check (
    public.is_base58_bytes_v1(asset_mint, 32)
  ),
  add constraint treasury_execution_receipt_destination_key check (
    public.is_base58_bytes_v1(destination_wallet, 32)
  ),
  add constraint treasury_execution_receipt_signature_key check (
    public.is_base58_bytes_v1(transaction_signature, 64)
  );

alter table public.treasury_execution_intents
  add constraint treasury_execution_intent_asset_mint_key check (
    public.is_base58_bytes_v1(asset_mint, 32)
  ),
  add constraint treasury_execution_intent_destination_key check (
    public.is_base58_bytes_v1(destination_wallet, 32)
  );

create or replace function public.validate_relief_payment_receipt()
returns trigger
language plpgsql
security definer
set search_path = ''
as $$
begin
  if new.status = 'paid' and not exists (
    select 1
    from public.treasury_execution_receipts receipt
    join public.treasury_execution_intents intent on intent.id = receipt.execution_intent_id
    where receipt.id = new.payment_receipt_id
      and intent.relief_application_id = new.id
      and intent.pool = 'relief'
      and intent.destination_wallet = new.wallet_address
      and intent.status = 'reconciled'
      and receipt.intent_hash = intent.intent_hash
      and receipt.decision_hash = intent.decision_hash
      and receipt.manifest_sha256 = intent.manifest_sha256
  ) then
    raise exception 'paid relief application requires a matching reconciled execution receipt';
  end if;
  return new;
end;
$$;
revoke all on function public.validate_relief_payment_receipt() from public;

create table public.treasury_execution_private_notes (
  note_id bigint generated always as identity primary key,
  execution_intent_id uuid not null references public.treasury_execution_intents(id),
  note_kind text not null check (
    note_kind in ('preparation', 'authorization', 'cancellation', 'reporting', 'reconciliation', 'failure')
  ),
  note_text text not null check (char_length(note_text) between 10 and 5000),
  actor_id uuid not null references auth.users(id),
  created_at timestamptz not null default timezone('utc', now())
);
comment on table public.treasury_execution_private_notes is
  'Append-only private staff notes. Never exposed through the public execution registry.';

create table public.operations_treasury_execution_workflow_events (
  event_id bigint generated always as identity primary key,
  execution_intent_id uuid not null references public.treasury_execution_intents(id),
  governance_decision_id uuid not null references public.governance_decisions(id),
  action text not null check (
    action in ('intent_prepared', 'intent_authorized', 'intent_cancelled', 'execution_reported', 'execution_reconciled', 'execution_failed')
  ),
  previous_status text,
  new_status text not null check (
    new_status in ('prepared', 'authorized', 'reported', 'reconciled', 'cancelled', 'failed')
  ),
  actor_id uuid not null references auth.users(id),
  actor_role text not null check (
    actor_role in ('treasury_preparer', 'treasury_authorizer', 'executor', 'treasury_reconciler')
  ),
  audit_reference text not null unique check (
    char_length(audit_reference) between 10 and 200 and audit_reference !~ '[[:cntrl:]]'
  ),
  decision_hash text not null check (decision_hash ~ '^[0-9a-f]{64}$'),
  manifest_sha256 text not null check (manifest_sha256 ~ '^[0-9a-f]{64}$'),
  intent_hash text not null check (intent_hash ~ '^[0-9a-f]{64}$'),
  event_payload jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default timezone('utc', now())
);
comment on table public.operations_treasury_execution_workflow_events is
  'Append-only private execution registry audit. Events record human workflow only and are not chain verification.';

create table public.treasury_execution_public_registry (
  intent_public_id uuid primary key,
  governance_decision_id uuid not null unique references public.governance_decisions(id),
  decision_hash text not null unique check (decision_hash ~ '^[0-9a-f]{64}$'),
  manifest_sha256 text not null unique check (manifest_sha256 ~ '^[0-9a-f]{64}$'),
  intent_hash text not null unique check (intent_hash ~ '^[0-9a-f]{64}$'),
  purpose_reference text not null,
  asset_symbol text not null check (asset_symbol = 'USDC'),
  asset_decimals smallint not null check (asset_decimals = 6),
  asset_mint text not null,
  destination_wallet_display text not null check (
    destination_wallet_display ~ '^[1-9A-HJ-NP-Za-km-z]{4}…[1-9A-HJ-NP-Za-km-z]{4}$'
  ),
  amount_base_units numeric(30, 0) not null check (amount_base_units > 0),
  network text not null check (network in ('devnet', 'mainnet-beta')),
  public_status text not null check (
    public_status in ('prepared', 'authorized', 'reported', 'reconciled', 'cancelled', 'failed')
  ),
  external_execution_reference text unique,
  prepared_at timestamptz not null,
  authorized_at timestamptz,
  reported_at timestamptz,
  reconciled_at timestamptz,
  cancelled_at timestamptz,
  reconciliation_reference text,
  updated_at timestamptz not null default timezone('utc', now()),
  constraint treasury_execution_public_registry_state check (
    (public_status = 'prepared' and authorized_at is null and reported_at is null and reconciled_at is null)
    or (public_status = 'authorized' and authorized_at is not null and reported_at is null and reconciled_at is null)
    or (public_status = 'reported' and authorized_at is not null and reported_at is not null and reconciled_at is null and external_execution_reference is not null)
    or (public_status in ('reconciled', 'failed') and authorized_at is not null and reported_at is not null and reconciled_at is not null and external_execution_reference is not null and reconciliation_reference is not null)
    or (public_status = 'cancelled' and reported_at is null and reconciled_at is null and cancelled_at is not null and reconciliation_reference is not null)
  )
);
comment on table public.treasury_execution_public_registry is
  'Sanitized public registry. It contains no staff Auth ID, full recipient wallet, verification material, or private notes.';
comment on table public.treasury_execution_intents is
  'Private immutable execution bindings and human workflow state. Prepared/authorized never mean paid; no sender or signing key exists.';
comment on table public.treasury_execution_receipts is
  'Immutable registration of an externally supplied execution reference. The database does not verify Solana finality.';

create index operations_treasury_execution_events_intent_idx
on public.operations_treasury_execution_workflow_events (execution_intent_id, created_at desc);
create index treasury_execution_private_notes_intent_idx
on public.treasury_execution_private_notes (execution_intent_id, created_at desc);

drop trigger treasury_execution_intents_protect_manifest on public.treasury_execution_intents;
create trigger treasury_execution_intents_protect_binding
before update on public.treasury_execution_intents
for each row execute function public.protect_operations_columns(
  'status', 'authorized_by', 'authorization_reference', 'authorized_at',
  'reported_by', 'reported_at', 'submitted_signature',
  'reconciled_by', 'reconciliation_reference', 'reconciled_at',
  'cancelled_by', 'cancellation_reference', 'cancelled_at', 'updated_at'
);

drop trigger treasury_execution_intents_enforce_status_transition on public.treasury_execution_intents;
create trigger treasury_execution_intents_enforce_status_transition
before update on public.treasury_execution_intents
for each row execute function public.enforce_operations_state_transition(
  'status',
  'prepared->authorized', 'prepared->cancelled',
  'authorized->reported', 'authorized->cancelled',
  'reported->reconciled', 'reported->failed'
);

create or replace function public.validate_treasury_execution_receipt_binding_v1()
returns trigger
language plpgsql
set search_path = public, pg_temp
as $$
begin
  if not exists (
    select 1 from public.treasury_execution_intents intent
    where intent.id = new.execution_intent_id
      and intent.governance_decision_id = new.governance_decision_id
      and intent.decision_hash = new.decision_hash
      and intent.manifest_sha256 = new.manifest_sha256
      and intent.intent_hash = new.intent_hash
      and intent.asset_symbol = new.asset_symbol
      and intent.asset_decimals = new.asset_decimals
      and intent.asset_mint = new.asset_mint
      and intent.destination_wallet = new.destination_wallet
      and intent.amount_base_units = new.amount_base_units
      and intent.network = new.network
      and intent.status = 'authorized'
      and intent.submitted_signature is null
  ) then
    raise exception 'execution receipt fields must exactly match one authorized intent';
  end if;
  return new;
end;
$$;
revoke all on function public.validate_treasury_execution_receipt_binding_v1() from public;
create trigger treasury_execution_receipts_validate_binding
before insert on public.treasury_execution_receipts
for each row execute function public.validate_treasury_execution_receipt_binding_v1();

create trigger treasury_execution_private_notes_immutable
before update or delete on public.treasury_execution_private_notes
for each row execute function public.reject_immutable_operations_mutation();
create trigger operations_treasury_execution_workflow_events_immutable
before update or delete on public.operations_treasury_execution_workflow_events
for each row execute function public.reject_immutable_operations_mutation();
create trigger treasury_execution_public_registry_protect_binding
before update on public.treasury_execution_public_registry
for each row execute function public.protect_operations_columns(
  'public_status', 'external_execution_reference', 'authorized_at', 'reported_at',
  'reconciled_at', 'cancelled_at', 'reconciliation_reference', 'updated_at'
);

alter table public.treasury_execution_private_notes enable row level security;
alter table public.operations_treasury_execution_workflow_events enable row level security;
alter table public.treasury_execution_public_registry enable row level security;

drop policy treasury_execution_intents_executor_read on public.treasury_execution_intents;
drop policy treasury_execution_intents_executor_insert on public.treasury_execution_intents;
drop policy treasury_execution_intents_executor_update on public.treasury_execution_intents;
drop policy treasury_execution_receipts_public_read on public.treasury_execution_receipts;
drop policy treasury_execution_receipts_executor_insert on public.treasury_execution_receipts;

create policy treasury_execution_intents_staff_read
on public.treasury_execution_intents for select to authenticated
using (public.has_operations_role(array['treasury_preparer', 'treasury_authorizer', 'executor', 'treasury_reconciler']));
create policy treasury_execution_receipts_staff_read
on public.treasury_execution_receipts for select to authenticated
using (public.has_operations_role(array['treasury_preparer', 'treasury_authorizer', 'executor', 'treasury_reconciler']));
create policy treasury_execution_private_notes_staff_read
on public.treasury_execution_private_notes for select to authenticated
using (public.has_operations_role(array['treasury_preparer', 'treasury_authorizer', 'executor', 'treasury_reconciler']));
create policy operations_treasury_execution_events_staff_read
on public.operations_treasury_execution_workflow_events for select to authenticated
using (public.has_operations_role(array['treasury_preparer', 'treasury_authorizer', 'executor', 'treasury_reconciler']));
create policy treasury_execution_public_registry_public_read
on public.treasury_execution_public_registry for select to anon, authenticated
using (true);

revoke all on table public.treasury_execution_intents from public, anon, authenticated, service_role;
revoke all on table public.treasury_execution_receipts from public, anon, authenticated, service_role;
revoke all on table public.treasury_execution_private_notes from public, anon, authenticated, service_role;
revoke all on table public.operations_treasury_execution_workflow_events from public, anon, authenticated, service_role;
revoke all on table public.treasury_execution_public_registry from public, anon, authenticated, service_role;
grant select on table public.treasury_execution_intents to authenticated;
grant select on table public.treasury_execution_receipts to authenticated;
grant select on table public.treasury_execution_private_notes to authenticated;
grant select on table public.operations_treasury_execution_workflow_events to authenticated;
grant select on table public.treasury_execution_public_registry to anon, authenticated;

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
  v_actor_role text := auth.jwt() -> 'app_metadata' ->> 'operations_role';
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
    or v_manifest ->> 'asset_decimals' is distinct from p_asset_decimals::text
    or v_manifest ->> 'asset_mint' is distinct from p_asset_mint
    or v_manifest ->> 'destination_wallet' is distinct from p_destination_wallet
    or v_manifest ->> 'amount_base_units' is distinct from p_amount_base_units::text
    or v_manifest ->> 'recipient_verification_reference' is distinct from btrim(p_recipient_verification_reference)
    or v_manifest ->> 'purpose_reference' is distinct from btrim(p_purpose_reference)
    or coalesce(v_manifest ->> 'relief_application_id', '') is distinct from coalesce(p_relief_application_id::text, '') then
    raise exception 'execution intent fields must exactly match the approved private manifest';
  end if;
  if (p_pool = 'relief') is distinct from (p_relief_application_id is not null) then
    raise exception 'relief intent requires exactly one relief application binding';
  end if;
  if p_pool not in ('relief', 'buyback', 'builders', 'staking') then raise exception 'invalid treasury pool'; end if;
  if p_pool = 'relief' and not exists (
    select 1 from public.relief_applications application
    where application.id = p_relief_application_id and application.status = 'approved'
      and application.payment_receipt_id is null and application.wallet_address = p_destination_wallet
      and p_amount_base_units <= application.requested_amount_usdc * 1000000
  ) then raise exception 'relief intent does not match an approved unpaid application'; end if;
  select event.actor_id into v_decision_finalizer
  from public.operations_governance_workflow_events event
  where event.action = 'decision_finalized' and event.public_entity_id = v_decision.id;
  if v_decision_finalizer is null or v_decision_finalizer = v_actor_id then
    raise exception 'intent preparer must be independent from the governance decision finalizer';
  end if;

  v_intent_hash := encode(sha256(convert_to(concat_ws(E'\n',
    'alpha-treasury-execution-intent-v1', v_decision.id::text, v_decision.decision_hash,
    v_decision.execution_manifest_sha256, p_pool, coalesce(p_relief_application_id::text, ''),
    p_network, p_asset_symbol, p_asset_decimals::text, p_asset_mint, p_destination_wallet,
    p_amount_base_units::text, btrim(p_recipient_verification_reference), btrim(p_purpose_reference)
  ), 'utf8')), 'hex');

  insert into public.treasury_execution_intents (
    governance_decision_id, decision_hash, intent_hash, pool, relief_application_id,
    network, asset_symbol, asset_decimals, asset_mint, destination_wallet,
    amount_base_units, recipient_verification_reference, purpose_reference,
    manifest_sha256, status, prepared_by
  ) values (
    v_decision.id, v_decision.decision_hash, v_intent_hash, p_pool, p_relief_application_id,
    p_network, p_asset_symbol, p_asset_decimals, p_asset_mint, p_destination_wallet,
    p_amount_base_units, btrim(p_recipient_verification_reference), btrim(p_purpose_reference),
    v_decision.execution_manifest_sha256, 'prepared', v_actor_id
  ) returning id into v_intent_id;
  insert into public.treasury_execution_private_notes (execution_intent_id, note_kind, note_text, actor_id)
  values (v_intent_id, 'preparation', btrim(p_private_note), v_actor_id);
  insert into public.operations_treasury_execution_workflow_events (
    execution_intent_id, governance_decision_id, action, previous_status, new_status,
    actor_id, actor_role, audit_reference, decision_hash, manifest_sha256, intent_hash, event_payload
  ) values (
    v_intent_id, v_decision.id, 'intent_prepared', null, 'prepared', v_actor_id, v_actor_role,
    btrim(p_audit_reference), v_decision.decision_hash, v_decision.execution_manifest_sha256,
    v_intent_hash, jsonb_build_object('authorized', false, 'transaction_sent', false, 'receipt_created', false)
  );
  insert into public.treasury_execution_public_registry (
    intent_public_id, governance_decision_id, decision_hash, manifest_sha256, intent_hash,
    purpose_reference, asset_symbol, asset_decimals, asset_mint, destination_wallet_display,
    amount_base_units, network, public_status, prepared_at
  ) values (
    v_intent_id, v_decision.id, v_decision.decision_hash, v_decision.execution_manifest_sha256,
    v_intent_hash, btrim(p_purpose_reference), p_asset_symbol, p_asset_decimals, p_asset_mint,
    left(p_destination_wallet, 4) || '…' || right(p_destination_wallet, 4),
    p_amount_base_units, p_network, 'prepared', timezone('utc', now())
  );
  return query select v_intent_id, v_intent_hash, 'prepared'::text, false, false;
end;
$$;

create or replace function public.authorize_treasury_execution_intent_v1(
  p_execution_intent_id uuid, p_authorization_reference text, p_private_note text, p_audit_reference text
)
returns table (execution_intent_id uuid, status text, payment_executed boolean, receipt_created boolean)
language plpgsql security definer set search_path = public, pg_temp
as $$
declare
  v_actor_id uuid := auth.uid();
  v_actor_role text := auth.jwt() -> 'app_metadata' ->> 'operations_role';
  v_intent public.treasury_execution_intents%rowtype;
  v_finalizer uuid;
  v_now timestamptz := timezone('utc', now());
begin
  if v_actor_id is null or v_actor_role is null or v_actor_role <> 'treasury_authorizer' then
    raise exception 'operations role is not authorized to authorize treasury execution intents';
  end if;
  if p_authorization_reference is null or char_length(btrim(p_authorization_reference)) not between 10 and 200 or p_authorization_reference ~ '[[:cntrl:]]'
    or p_audit_reference is null or char_length(btrim(p_audit_reference)) not between 10 and 200 or p_audit_reference ~ '[[:cntrl:]]'
    or p_private_note is null or char_length(btrim(p_private_note)) not between 10 and 5000 then
    raise exception 'authorization reference, audit reference, and private note are required';
  end if;
  select * into v_intent from public.treasury_execution_intents where id = p_execution_intent_id for update;
  if not found or v_intent.status <> 'prepared' then raise exception 'only a prepared intent can be authorized'; end if;
  if v_intent.prepared_by = v_actor_id then raise exception 'intent preparer cannot authorize their own intent'; end if;
  select event.actor_id into v_finalizer from public.operations_governance_workflow_events event
  where event.action = 'decision_finalized' and event.public_entity_id = v_intent.governance_decision_id;
  if v_finalizer = v_actor_id then raise exception 'governance decision finalizer cannot authorize its execution intent'; end if;
  update public.treasury_execution_intents set status = 'authorized', authorized_by = v_actor_id,
    authorization_reference = btrim(p_authorization_reference), authorized_at = v_now where id = v_intent.id;
  insert into public.treasury_execution_private_notes values (default, v_intent.id, 'authorization', btrim(p_private_note), v_actor_id, v_now);
  insert into public.operations_treasury_execution_workflow_events (
    execution_intent_id, governance_decision_id, action, previous_status, new_status, actor_id,
    actor_role, audit_reference, decision_hash, manifest_sha256, intent_hash, event_payload
  ) values (v_intent.id, v_intent.governance_decision_id, 'intent_authorized', 'prepared', 'authorized',
    v_actor_id, v_actor_role, btrim(p_audit_reference), v_intent.decision_hash, v_intent.manifest_sha256,
    v_intent.intent_hash, jsonb_build_object('authorization_is_payment', false, 'transaction_sent', false, 'receipt_created', false));
  update public.treasury_execution_public_registry set public_status = 'authorized', authorized_at = v_now, updated_at = v_now
  where intent_public_id = v_intent.id;
  return query select v_intent.id, 'authorized'::text, false, false;
end;
$$;

create or replace function public.cancel_treasury_execution_intent_v1(
  p_execution_intent_id uuid, p_cancellation_reference text, p_private_note text, p_audit_reference text
)
returns table (execution_intent_id uuid, status text, transaction_sent boolean, receipt_created boolean)
language plpgsql security definer set search_path = public, pg_temp
as $$
declare
  v_actor_id uuid := auth.uid();
  v_actor_role text := auth.jwt() -> 'app_metadata' ->> 'operations_role';
  v_intent public.treasury_execution_intents%rowtype;
  v_now timestamptz := timezone('utc', now());
begin
  if v_actor_id is null or v_actor_role is null or v_actor_role not in ('treasury_preparer', 'treasury_authorizer') then
    raise exception 'operations role is not authorized to cancel treasury execution intents';
  end if;
  if p_cancellation_reference is null or char_length(btrim(p_cancellation_reference)) not between 10 and 200 or p_cancellation_reference ~ '[[:cntrl:]]'
    or p_audit_reference is null or char_length(btrim(p_audit_reference)) not between 10 and 200 or p_audit_reference ~ '[[:cntrl:]]'
    or p_private_note is null or char_length(btrim(p_private_note)) not between 10 and 5000 then
    raise exception 'cancellation reference, audit reference, and private note are required';
  end if;
  select * into v_intent from public.treasury_execution_intents where id = p_execution_intent_id for update;
  if not found or v_intent.status not in ('prepared', 'authorized') then raise exception 'only prepared or authorized intent can be cancelled'; end if;
  update public.treasury_execution_intents set status = 'cancelled', cancelled_by = v_actor_id,
    cancellation_reference = btrim(p_cancellation_reference), cancelled_at = v_now where id = v_intent.id;
  insert into public.treasury_execution_private_notes values (default, v_intent.id, 'cancellation', btrim(p_private_note), v_actor_id, v_now);
  insert into public.operations_treasury_execution_workflow_events (
    execution_intent_id, governance_decision_id, action, previous_status, new_status, actor_id,
    actor_role, audit_reference, decision_hash, manifest_sha256, intent_hash, event_payload
  ) values (v_intent.id, v_intent.governance_decision_id, 'intent_cancelled', v_intent.status, 'cancelled',
    v_actor_id, v_actor_role, btrim(p_audit_reference), v_intent.decision_hash, v_intent.manifest_sha256,
    v_intent.intent_hash, jsonb_build_object('transaction_sent', false, 'receipt_created', false));
  update public.treasury_execution_public_registry set public_status = 'cancelled', cancelled_at = v_now,
    reconciliation_reference = btrim(p_cancellation_reference), updated_at = v_now where intent_public_id = v_intent.id;
  return query select v_intent.id, 'cancelled'::text, false, false;
end;
$$;

create or replace function public.report_treasury_execution_receipt_v1(
  p_execution_intent_id uuid, p_transaction_signature text, p_confirmed_at timestamptz,
  p_private_note text, p_audit_reference text
)
returns table (execution_intent_id uuid, execution_receipt_id uuid, status text, chain_verified_by_database boolean)
language plpgsql security definer set search_path = public, pg_temp
as $$
declare
  v_actor_id uuid := auth.uid();
  v_actor_role text := auth.jwt() -> 'app_metadata' ->> 'operations_role';
  v_intent public.treasury_execution_intents%rowtype;
  v_receipt_id uuid;
  v_receipt_hash text;
  v_now timestamptz := timezone('utc', now());
begin
  if v_actor_id is null or v_actor_role is null or v_actor_role <> 'executor' then
    raise exception 'operations role is not authorized to report external execution receipts';
  end if;
  if not public.is_base58_bytes_v1(p_transaction_signature, 64) then raise exception 'invalid Solana transaction signature'; end if;
  if p_confirmed_at is null or p_confirmed_at > v_now + interval '5 minutes' then raise exception 'invalid externally reported confirmation time'; end if;
  if p_audit_reference is null or char_length(btrim(p_audit_reference)) not between 10 and 200 or p_audit_reference ~ '[[:cntrl:]]'
    or p_private_note is null or char_length(btrim(p_private_note)) not between 10 and 5000 then
    raise exception 'report audit reference and private note are required';
  end if;
  select * into v_intent from public.treasury_execution_intents where id = p_execution_intent_id for update;
  if not found or v_intent.status <> 'authorized' then raise exception 'only an authorized intent can record an external receipt'; end if;
  if v_intent.prepared_by = v_actor_id or v_intent.authorized_by = v_actor_id then
    raise exception 'external execution reporter must be independent from intent preparation and authorization';
  end if;
  if p_confirmed_at < v_intent.authorized_at then raise exception 'external confirmation cannot predate intent authorization'; end if;
  v_receipt_hash := encode(sha256(convert_to(concat_ws(E'\n',
    'alpha-treasury-execution-receipt-v1', v_intent.id::text, v_intent.governance_decision_id::text,
    v_intent.decision_hash, v_intent.manifest_sha256, v_intent.intent_hash,
    v_intent.network, v_intent.asset_symbol, v_intent.asset_decimals::text, v_intent.asset_mint,
    v_intent.destination_wallet, v_intent.amount_base_units::text, p_transaction_signature, p_confirmed_at::text
  ), 'utf8')), 'hex');
  insert into public.treasury_execution_receipts (
    execution_intent_id, governance_decision_id, decision_hash, intent_hash, chain, network,
    asset_symbol, asset_decimals, asset_mint, destination_wallet, amount_base_units,
    transaction_signature, manifest_sha256, receipt_sha256, confirmed_at, recorded_by
  ) values (
    v_intent.id, v_intent.governance_decision_id, v_intent.decision_hash, v_intent.intent_hash,
    'solana', v_intent.network, v_intent.asset_symbol, v_intent.asset_decimals, v_intent.asset_mint,
    v_intent.destination_wallet, v_intent.amount_base_units, p_transaction_signature,
    v_intent.manifest_sha256, v_receipt_hash, p_confirmed_at, v_actor_id
  ) returning id into v_receipt_id;
  update public.treasury_execution_intents set status = 'reported', submitted_signature = p_transaction_signature,
    reported_by = v_actor_id, reported_at = v_now where id = v_intent.id;
  insert into public.treasury_execution_private_notes values (default, v_intent.id, 'reporting', btrim(p_private_note), v_actor_id, v_now);
  insert into public.operations_treasury_execution_workflow_events (
    execution_intent_id, governance_decision_id, action, previous_status, new_status, actor_id,
    actor_role, audit_reference, decision_hash, manifest_sha256, intent_hash, event_payload
  ) values (v_intent.id, v_intent.governance_decision_id, 'execution_reported', 'authorized', 'reported',
    v_actor_id, v_actor_role, btrim(p_audit_reference), v_intent.decision_hash, v_intent.manifest_sha256,
    v_intent.intent_hash, jsonb_build_object('receipt_id', v_receipt_id, 'receipt_sha256', v_receipt_hash, 'chain_verified_by_database', false));
  update public.treasury_execution_public_registry set public_status = 'reported',
    external_execution_reference = p_transaction_signature, reported_at = v_now, updated_at = v_now
  where intent_public_id = v_intent.id;
  return query select v_intent.id, v_receipt_id, 'reported'::text, false;
end;
$$;

create or replace function public.reconcile_treasury_execution_v1(
  p_execution_intent_id uuid, p_outcome text, p_reconciliation_reference text,
  p_private_note text, p_audit_reference text
)
returns table (execution_intent_id uuid, status text, chain_verified_by_database boolean)
language plpgsql security definer set search_path = public, pg_temp
as $$
declare
  v_actor_id uuid := auth.uid();
  v_actor_role text := auth.jwt() -> 'app_metadata' ->> 'operations_role';
  v_intent public.treasury_execution_intents%rowtype;
  v_now timestamptz := timezone('utc', now());
begin
  if v_actor_id is null or v_actor_role is null or v_actor_role <> 'treasury_reconciler' then
    raise exception 'operations role is not authorized to reconcile treasury execution records';
  end if;
  if p_outcome is null or p_outcome not in ('reconciled', 'failed') then raise exception 'reconciliation outcome must be reconciled or failed'; end if;
  if p_reconciliation_reference is null or char_length(btrim(p_reconciliation_reference)) not between 10 and 200 or p_reconciliation_reference ~ '[[:cntrl:]]'
    or p_audit_reference is null or char_length(btrim(p_audit_reference)) not between 10 and 200 or p_audit_reference ~ '[[:cntrl:]]'
    or p_private_note is null or char_length(btrim(p_private_note)) not between 10 and 5000 then
    raise exception 'reconciliation reference, audit reference, and private note are required';
  end if;
  select * into v_intent from public.treasury_execution_intents where id = p_execution_intent_id for update;
  if not found or v_intent.status <> 'reported' then raise exception 'only a reported external execution can be reconciled or failed'; end if;
  if v_actor_id in (v_intent.prepared_by, v_intent.authorized_by, v_intent.reported_by) then
    raise exception 'reconciler must be independent from preparation, authorization, and reporting';
  end if;
  if not exists (select 1 from public.treasury_execution_receipts receipt
    where receipt.execution_intent_id = v_intent.id and receipt.intent_hash = v_intent.intent_hash
      and receipt.decision_hash = v_intent.decision_hash and receipt.manifest_sha256 = v_intent.manifest_sha256) then
    raise exception 'reported execution has no exactly bound immutable receipt';
  end if;
  update public.treasury_execution_intents set status = p_outcome, reconciled_by = v_actor_id,
    reconciliation_reference = btrim(p_reconciliation_reference), reconciled_at = v_now where id = v_intent.id;
  insert into public.treasury_execution_private_notes values (
    default, v_intent.id, case when p_outcome = 'reconciled' then 'reconciliation' else 'failure' end,
    btrim(p_private_note), v_actor_id, v_now
  );
  insert into public.operations_treasury_execution_workflow_events (
    execution_intent_id, governance_decision_id, action, previous_status, new_status, actor_id,
    actor_role, audit_reference, decision_hash, manifest_sha256, intent_hash, event_payload
  ) values (v_intent.id, v_intent.governance_decision_id,
    case when p_outcome = 'reconciled' then 'execution_reconciled' else 'execution_failed' end,
    'reported', p_outcome, v_actor_id, v_actor_role, btrim(p_audit_reference),
    v_intent.decision_hash, v_intent.manifest_sha256, v_intent.intent_hash,
    jsonb_build_object('chain_verified_by_database', false, 'registry_reconciliation_only', true));
  update public.treasury_execution_public_registry set public_status = p_outcome, reconciled_at = v_now,
    reconciliation_reference = btrim(p_reconciliation_reference), updated_at = v_now where intent_public_id = v_intent.id;
  return query select v_intent.id, p_outcome, false;
end;
$$;

comment on function public.authorize_treasury_execution_intent_v1(uuid,text,text,text) is
  'Human authorization registry only. Authorization is not payment and sends no transaction.';
comment on function public.report_treasury_execution_receipt_v1(uuid,text,timestamptz,text,text) is
  'Registers an externally supplied Solana signature. The database does not query or verify chain finality.';
comment on function public.reconcile_treasury_execution_v1(uuid,text,text,text,text) is
  'Records manual reconciliation outcome only; it is not database chain verification.';

revoke all on function public.prepare_treasury_execution_intent_v1(uuid,text,uuid,text,text,smallint,text,text,numeric,text,text,text,text) from public, anon, authenticated, service_role;
grant execute on function public.prepare_treasury_execution_intent_v1(uuid,text,uuid,text,text,smallint,text,text,numeric,text,text,text,text) to authenticated;
revoke all on function public.authorize_treasury_execution_intent_v1(uuid,text,text,text) from public, anon, authenticated, service_role;
grant execute on function public.authorize_treasury_execution_intent_v1(uuid,text,text,text) to authenticated;
revoke all on function public.cancel_treasury_execution_intent_v1(uuid,text,text,text) from public, anon, authenticated, service_role;
grant execute on function public.cancel_treasury_execution_intent_v1(uuid,text,text,text) to authenticated;
revoke all on function public.report_treasury_execution_receipt_v1(uuid,text,timestamptz,text,text) from public, anon, authenticated, service_role;
grant execute on function public.report_treasury_execution_receipt_v1(uuid,text,timestamptz,text,text) to authenticated;
revoke all on function public.reconcile_treasury_execution_v1(uuid,text,text,text,text) from public, anon, authenticated, service_role;
grant execute on function public.reconcile_treasury_execution_v1(uuid,text,text,text,text) to authenticated;

-- Exact service-role-only deletion gate for reserved Phase 2E-6D Staging fixtures.
create or replace function public.protect_treasury_execution_registry_immutable_v1()
returns trigger language plpgsql set search_path = public, pg_temp as $$
declare
  v_reference text := current_setting('alpha.treasury_execution_6d_cleanup_reference', true);
  v_owner name;
begin
  select pg_get_userbyid(proowner) into v_owner from pg_proc
  where oid = 'public.cleanup_treasury_execution_staging_e2e_v1(text,uuid,uuid[])'::regprocedure;
  if v_reference ~ '^phase-2e-6d-staging-e2e:[0-9]{13}-[0-9a-f]{8}$'
    and v_owner is not null and current_user = v_owner then return old; end if;
  raise exception 'immutable treasury execution registry record cannot be changed';
end;
$$;
revoke all on function public.protect_treasury_execution_registry_immutable_v1() from public;

drop trigger treasury_execution_receipts_immutable on public.treasury_execution_receipts;
create trigger treasury_execution_receipts_immutable
before update or delete on public.treasury_execution_receipts
for each row execute function public.protect_treasury_execution_registry_immutable_v1();
drop trigger treasury_execution_private_notes_immutable on public.treasury_execution_private_notes;
create trigger treasury_execution_private_notes_immutable
before update or delete on public.treasury_execution_private_notes
for each row execute function public.protect_treasury_execution_registry_immutable_v1();
drop trigger operations_treasury_execution_workflow_events_immutable on public.operations_treasury_execution_workflow_events;
create trigger operations_treasury_execution_workflow_events_immutable
before update or delete on public.operations_treasury_execution_workflow_events
for each row execute function public.protect_treasury_execution_registry_immutable_v1();
create trigger treasury_execution_public_registry_immutable_delete
before delete on public.treasury_execution_public_registry
for each row execute function public.protect_treasury_execution_registry_immutable_v1();

create or replace function public.cleanup_treasury_execution_staging_e2e_v1(
  p_run_reference text, p_fixture_owner_id uuid, p_execution_intent_ids uuid[]
)
returns table (
  receipts_deleted integer, events_deleted integer, notes_deleted integer,
  public_records_deleted integer, intents_deleted integer
)
language plpgsql security definer set search_path = public, pg_temp
as $$
declare
  v_run_id text;
  v_receipts integer; v_events integer; v_notes integer; v_public integer; v_intents integer;
begin
  if p_run_reference !~ '^phase-2e-6d-staging-e2e:[0-9]{13}-[0-9a-f]{8}$' then
    raise exception 'invalid Phase 2E-6D Staging E2E cleanup reference';
  end if;
  if p_fixture_owner_id is null or cardinality(p_execution_intent_ids) <> 2
    or cardinality(array(select distinct value from unnest(p_execution_intent_ids) value)) <> 2 then
    raise exception 'Phase 2E-6D cleanup requires exactly two distinct intent identifiers';
  end if;
  v_run_id := split_part(p_run_reference, ':', 2);
  if (select count(*) from public.treasury_execution_intents intent
    where intent.id = any(p_execution_intent_ids) and intent.prepared_by = p_fixture_owner_id
      and intent.purpose_reference in ('Staging treasury execution reconcile ' || v_run_id, 'Staging treasury execution cancel ' || v_run_id)) <> 2 then
    raise exception 'cleanup intents are not exact owner-bound Phase 2E-6D fixtures';
  end if;
  if exists (select 1 from public.operations_treasury_execution_workflow_events event
    where event.execution_intent_id = any(p_execution_intent_ids)
      and event.audit_reference not like p_run_reference || ':%') then
    raise exception 'Phase 2E-6D cleanup audit reference is not isolated';
  end if;
  perform set_config('alpha.treasury_execution_6d_cleanup_reference', p_run_reference, true);
  delete from public.treasury_execution_receipts where execution_intent_id = any(p_execution_intent_ids);
  get diagnostics v_receipts = row_count;
  delete from public.operations_treasury_execution_workflow_events where execution_intent_id = any(p_execution_intent_ids);
  get diagnostics v_events = row_count;
  delete from public.treasury_execution_private_notes where execution_intent_id = any(p_execution_intent_ids);
  get diagnostics v_notes = row_count;
  delete from public.treasury_execution_public_registry where intent_public_id = any(p_execution_intent_ids);
  get diagnostics v_public = row_count;
  delete from public.treasury_execution_intents where id = any(p_execution_intent_ids);
  get diagnostics v_intents = row_count;
  perform set_config('alpha.treasury_execution_6d_cleanup_reference', '', true);
  if v_public <> 2 or v_intents <> 2 then raise exception 'Phase 2E-6D cleanup count mismatch'; end if;
  return query select v_receipts, v_events, v_notes, v_public, v_intents;
end;
$$;
comment on function public.cleanup_treasury_execution_staging_e2e_v1(text,uuid,uuid[]) is
  'Deletes only two exact owner-bound Phase 2E-6D Staging execution fixtures; never deletes governance decisions.';
revoke all on function public.cleanup_treasury_execution_staging_e2e_v1(text,uuid,uuid[]) from public, anon, authenticated, service_role;
grant execute on function public.cleanup_treasury_execution_staging_e2e_v1(text,uuid,uuid[]) to service_role;

revoke delete on table public.treasury_execution_intents, public.treasury_execution_receipts,
  public.treasury_execution_private_notes, public.operations_treasury_execution_workflow_events,
  public.treasury_execution_public_registry from service_role;
