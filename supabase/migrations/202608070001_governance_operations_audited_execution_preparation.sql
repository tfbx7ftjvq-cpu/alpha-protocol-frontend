-- Alpha Protocol Phase 2E-6C: governance operations and audited execution preparation.
-- This migration is intentionally off-chain only. Governance approval never creates
-- a treasury execution intent, transaction, payment, or execution receipt.

create table public.governance_proposal_submissions (
  id uuid primary key default gen_random_uuid(),
  submitted_by uuid not null references auth.users(id),
  wallet_address text not null check (char_length(wallet_address) between 32 and 44),
  title text not null check (char_length(title) between 4 and 160),
  private_summary text not null check (char_length(private_summary) between 20 and 8000),
  proposal_kind text not null check (
    proposal_kind in (
      'task_acceptance', 'risk_finding', 'relief_recommendation',
      'builders_spend', 'buyback_policy', 'staking_policy',
      'protocol_parameter', 'other'
    )
  ),
  execution_required boolean not null default false,
  private_execution_manifest jsonb,
  execution_manifest_sha256 text check (
    execution_manifest_sha256 is null
    or execution_manifest_sha256 ~ '^[0-9a-f]{64}$'
  ),
  public_proposal_consent boolean not null default false,
  review_status text not null default 'pending'
    check (review_status in ('pending', 'published', 'rejected')),
  reviewed_by uuid references auth.users(id),
  reviewer_notes text check (
    reviewer_notes is null or char_length(reviewer_notes) between 1 and 5000
  ),
  published_proposal_id uuid unique references public.governance_proposals(id),
  created_at timestamptz not null default timezone('utc', now()),
  updated_at timestamptz not null default timezone('utc', now()),
  reviewed_at timestamptz,
  constraint governance_proposal_submission_execution_manifest check (
    (execution_required and private_execution_manifest is not null and execution_manifest_sha256 is not null)
    or (not execution_required and private_execution_manifest is null and execution_manifest_sha256 is null)
  ),
  constraint governance_proposal_submission_review_fields check (
    (review_status = 'pending' and reviewed_by is null and reviewer_notes is null
      and published_proposal_id is null and reviewed_at is null)
    or (review_status = 'rejected' and reviewed_by is not null and reviewer_notes is not null
      and published_proposal_id is null and reviewed_at is not null)
    or (review_status = 'published' and reviewed_by is not null and reviewer_notes is not null
      and published_proposal_id is not null and reviewed_at is not null)
  )
);

comment on table public.governance_proposal_submissions is
  'Private wallet-authenticated governance proposal intake. Raw summaries, manifests, owner IDs, and reviewer notes never belong to the public proposal table.';

create or replace function public.enforce_operations_submission_rate_limit()
returns trigger
language plpgsql
security definer
set search_path = ''
as $$
declare
  maximum_rows integer;
  lookback interval;
  recent_rows bigint;
begin
  if not public.is_operations_wallet_intake_enabled() then raise exception 'operations wallet intake is disabled'; end if;
  if new.submitted_by is distinct from auth.uid() then raise exception 'operations submission owner does not match auth.uid()'; end if;
  new.created_at := now();
  new.updated_at := now();
  case tg_table_name
    when 'task_submissions' then maximum_rows := 8; lookback := interval '1 hour';
    when 'risk_reports' then maximum_rows := 6; lookback := interval '1 hour';
    when 'relief_applications' then maximum_rows := 3; lookback := interval '24 hours';
    when 'governance_discussions' then maximum_rows := 20; lookback := interval '1 hour';
    when 'governance_proposal_submissions' then maximum_rows := 5; lookback := interval '1 hour';
    else raise exception 'unsupported operations rate-limit table: %', tg_table_name;
  end case;
  perform pg_advisory_xact_lock(hashtextextended(new.submitted_by::text || ':' || tg_table_name, 0));
  execute format('select count(*) from public.%I where submitted_by = $1 and created_at > now() - $2', tg_table_name)
  into recent_rows using new.submitted_by, lookback;
  if recent_rows >= maximum_rows then raise exception 'operations submission rate limit exceeded for %', tg_table_name; end if;
  return new;
end;
$$;
revoke all on function public.enforce_operations_submission_rate_limit() from public;

create trigger governance_proposal_submissions_rate_limit
before insert on public.governance_proposal_submissions
for each row execute function public.enforce_operations_submission_rate_limit();

alter table public.governance_discussions
  add column public_body_consent boolean not null default false,
  add column public_wallet_consent boolean not null default false,
  add column reviewer_notes text check (
    reviewer_notes is null or char_length(reviewer_notes) between 1 and 5000
  );

alter table public.governance_discussions
  add constraint governance_discussions_public_wallet_consent check (
    not public_wallet_consent or public_body_consent
  );

drop trigger governance_discussions_protect_content on public.governance_discussions;
create trigger governance_discussions_protect_content
before update on public.governance_discussions
for each row execute function public.protect_operations_columns(
  'moderation_status', 'moderated_by', 'reviewer_notes', 'updated_at'
);

do $$
begin
  if exists (select 1 from public.governance_decisions) then
    raise exception 'Phase 2E-6C requires an empty legacy governance_decisions table; export and explicitly migrate legacy decisions before applying';
  end if;
end;
$$;

alter table public.governance_decisions
  add column execution_manifest_sha256 text check (
    execution_manifest_sha256 is null
    or execution_manifest_sha256 ~ '^[0-9a-f]{64}$'
  ),
  add column finalization_reference text not null check (
    char_length(finalization_reference) between 10 and 200
    and finalization_reference !~ '[[:cntrl:]]'
  ),
  add constraint governance_decision_manifest_binding check (
    (execution_required and execution_manifest_sha256 is not null)
    or (not execution_required and execution_manifest_sha256 is null)
  );

create or replace function public.validate_governance_execution_intent_manifest_v1()
returns trigger language plpgsql set search_path = public, pg_temp as $$
begin
  if not exists (
    select 1 from public.governance_decisions decision
    where decision.id = new.governance_decision_id
      and decision.decision = 'approved'
      and decision.execution_required
      and decision.execution_manifest_sha256 = new.manifest_sha256
  ) then
    raise exception 'execution intent manifest sha256 must match an approved execution-required governance decision';
  end if;
  return new;
end;
$$;
revoke all on function public.validate_governance_execution_intent_manifest_v1() from public;

create trigger treasury_execution_intents_validate_governance_manifest
before insert or update on public.treasury_execution_intents
for each row execute function public.validate_governance_execution_intent_manifest_v1();

create table public.operations_governance_workflow_events (
  event_id bigint generated always as identity primary key,
  entity_type text not null check (
    entity_type in ('proposal_submission', 'proposal', 'discussion', 'decision')
  ),
  private_entity_id uuid,
  public_entity_id uuid,
  action text not null check (
    action in (
      'proposal_submitted', 'proposal_published', 'proposal_rejected',
      'discussion_submitted', 'discussion_published', 'discussion_rejected',
      'decision_finalized'
    )
  ),
  actor_id uuid not null references auth.users(id),
  actor_role text not null check (
    actor_role in ('wallet_submitter', 'operator', 'moderator', 'governance_admin')
  ),
  event_reference text not null unique check (
    char_length(event_reference) between 10 and 200
    and event_reference !~ '[[:cntrl:]]'
  ),
  event_payload jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default timezone('utc', now()),
  constraint operations_governance_event_entity check (
    private_entity_id is not null or public_entity_id is not null
  )
);

comment on table public.operations_governance_workflow_events is
  'Append-only private governance workflow audit. Events prove review and binding state but are not execution or payment receipts.';

create index operations_governance_events_private_idx
on public.operations_governance_workflow_events (private_entity_id, created_at desc);

create index operations_governance_events_public_idx
on public.operations_governance_workflow_events (public_entity_id, created_at desc);

create trigger governance_proposal_submissions_protect_content
before update on public.governance_proposal_submissions
for each row execute function public.protect_operations_columns(
  'review_status', 'reviewed_by', 'reviewer_notes',
  'published_proposal_id', 'reviewed_at'
  , 'updated_at'
);

create trigger operations_governance_workflow_events_immutable
before update or delete on public.operations_governance_workflow_events
for each row execute function public.reject_immutable_operations_mutation();

alter table public.governance_proposal_submissions enable row level security;
alter table public.operations_governance_workflow_events enable row level security;

create policy governance_proposal_submissions_owner_read
on public.governance_proposal_submissions
for select to authenticated
using (submitted_by = auth.uid());

create policy governance_proposal_submissions_operator_read
on public.governance_proposal_submissions
for select to authenticated
using (public.has_operations_role(array['operator', 'governance_admin']));

drop policy governance_discussions_moderator_read on public.governance_discussions;
create policy governance_discussions_moderator_read
on public.governance_discussions
for select to authenticated
using (public.has_operations_role(array['moderator', 'governance_admin']));

create policy operations_governance_workflow_events_staff_read
on public.operations_governance_workflow_events
for select to authenticated
using (public.has_operations_role(array['operator', 'moderator', 'governance_admin']));

revoke all on table public.governance_proposal_submissions from public, anon, authenticated, service_role;
revoke all on table public.operations_governance_workflow_events from public, anon, authenticated, service_role;
grant select on table public.governance_proposal_submissions to authenticated;
grant select on table public.operations_governance_workflow_events to authenticated;

-- Remove the legacy direct-write surface. All governance mutations below are atomic RPCs.
revoke insert, update, delete on table public.governance_proposals from authenticated;
revoke insert, update, delete on table public.governance_discussions from authenticated;
revoke insert, update, delete on table public.governance_discussion_publications from authenticated;
revoke insert, update, delete on table public.governance_decisions from authenticated;
revoke insert, update, delete on table public.treasury_execution_intents from authenticated;
revoke insert, update, delete on table public.treasury_execution_receipts from authenticated;

create or replace function public.submit_governance_proposal_v1(
  p_title text,
  p_private_summary text,
  p_proposal_kind text,
  p_execution_required boolean,
  p_private_execution_manifest jsonb,
  p_execution_manifest_sha256 text,
  p_public_proposal_consent boolean,
  p_submission_reference text
)
returns table (proposal_submission_id uuid, review_status text)
language plpgsql
security definer
set search_path = public, pg_temp
as $$
declare
  v_actor_id uuid := auth.uid();
  v_wallet text := public.current_verified_solana_wallet();
  v_id uuid;
  v_hash text := lower(nullif(btrim(p_execution_manifest_sha256), ''));
begin
  if v_actor_id is null or v_wallet is null then
    raise exception 'verified wallet authentication is required';
  end if;
  if not public.is_operations_wallet_intake_enabled() then
    raise exception 'operations wallet intake is disabled';
  end if;
  if p_submission_reference is null
    or char_length(btrim(p_submission_reference)) not between 10 and 200
    or p_submission_reference ~ '[[:cntrl:]]' then
    raise exception 'invalid proposal submission reference';
  end if;
  if p_execution_required is null or p_public_proposal_consent is null then
    raise exception 'proposal booleans are required';
  end if;
  if p_execution_required then
    if p_private_execution_manifest is null or v_hash !~ '^[0-9a-f]{64}$' then
      raise exception 'execution proposal requires a private manifest and sha256';
    end if;
  elsif p_private_execution_manifest is not null or v_hash is not null then
    raise exception 'non-execution proposal cannot include an execution manifest';
  end if;

  insert into public.governance_proposal_submissions (
    submitted_by, wallet_address, title, private_summary, proposal_kind,
    execution_required, private_execution_manifest, execution_manifest_sha256,
    public_proposal_consent
  ) values (
    v_actor_id, v_wallet, btrim(p_title), btrim(p_private_summary), p_proposal_kind,
    p_execution_required, p_private_execution_manifest, v_hash,
    p_public_proposal_consent
  ) returning id into v_id;

  insert into public.operations_governance_workflow_events (
    entity_type, private_entity_id, action, actor_id, actor_role,
    event_reference, event_payload
  ) values (
    'proposal_submission', v_id, 'proposal_submitted', v_actor_id, 'wallet_submitter',
    btrim(p_submission_reference),
    jsonb_build_object('execution_required', p_execution_required, 'public_consent', p_public_proposal_consent)
  );

  return query select v_id, 'pending'::text;
end;
$$;

create or replace function public.publish_governance_proposal_v1(
  p_proposal_submission_id uuid,
  p_decision text,
  p_reviewer_notes text,
  p_public_title text,
  p_public_summary text,
  p_public_source_reference text,
  p_execution_manifest_url text,
  p_execution_manifest_sha256 text,
  p_audit_reference text
)
returns table (proposal_submission_id uuid, public_proposal_id uuid, review_status text)
language plpgsql
security definer
set search_path = public, pg_temp
as $$
declare
  v_actor_id uuid := auth.uid();
  v_actor_role text := auth.jwt() -> 'app_metadata' ->> 'operations_role';
  v_submission public.governance_proposal_submissions%rowtype;
  v_public_id uuid;
  v_reviewed_at timestamptz := timezone('utc', now());
  v_hash text := lower(nullif(btrim(p_execution_manifest_sha256), ''));
begin
  if v_actor_id is null or v_actor_role is null or v_actor_role not in ('operator', 'governance_admin') then
    raise exception 'operations role is not authorized to publish governance proposals';
  end if;
  if p_decision is null or p_decision not in ('published', 'rejected') then
    raise exception 'proposal review decision must be published or rejected';
  end if;
  if p_audit_reference is null or char_length(btrim(p_audit_reference)) not between 10 and 200
    or p_audit_reference ~ '[[:cntrl:]]' then
    raise exception 'invalid proposal review audit reference';
  end if;

  select * into v_submission
  from public.governance_proposal_submissions
  where id = p_proposal_submission_id
  for update;
  if not found then raise exception 'proposal submission not found'; end if;
  if v_submission.review_status <> 'pending' then raise exception 'proposal submission is already terminal'; end if;
  if v_submission.submitted_by = v_actor_id then raise exception 'operators cannot review their own governance proposal'; end if;
  if p_reviewer_notes is null or char_length(btrim(p_reviewer_notes)) not between 1 and 5000 then
    raise exception 'reviewer notes are required';
  end if;

  if p_decision = 'published' then
    if not v_submission.public_proposal_consent then
      raise exception 'sanitized proposal publication requires submitter consent';
    end if;
    if v_submission.execution_required then
      if v_hash is distinct from v_submission.execution_manifest_sha256
        or p_execution_manifest_url is null or p_execution_manifest_url !~ '^https://' then
        raise exception 'public execution manifest must match the private proposal sha256';
      end if;
    elsif v_hash is not null or nullif(btrim(p_execution_manifest_url), '') is not null then
      raise exception 'non-execution proposal cannot publish an execution manifest';
    end if;

    insert into public.governance_proposals (
      title, summary, proposal_kind, public_source_reference,
      execution_required, execution_manifest_url, status,
      publication_status, published_at
    ) values (
      btrim(p_public_title), btrim(p_public_summary), v_submission.proposal_kind,
      nullif(btrim(p_public_source_reference), ''), v_submission.execution_required,
      nullif(btrim(p_execution_manifest_url), ''), 'discussion', 'published', v_reviewed_at
    ) returning id into v_public_id;
  elsif nullif(btrim(p_public_title), '') is not null
    or nullif(btrim(p_public_summary), '') is not null
    or nullif(btrim(p_public_source_reference), '') is not null
    or nullif(btrim(p_execution_manifest_url), '') is not null
    or v_hash is not null then
    raise exception 'rejected proposal cannot create public content';
  end if;

  update public.governance_proposal_submissions
  set review_status = p_decision, reviewed_by = v_actor_id,
      reviewer_notes = btrim(p_reviewer_notes), published_proposal_id = v_public_id,
      reviewed_at = v_reviewed_at
  where id = v_submission.id;

  insert into public.operations_governance_workflow_events (
    entity_type, private_entity_id, public_entity_id, action, actor_id,
    actor_role, event_reference, event_payload
  ) values (
    'proposal', v_submission.id, v_public_id,
    case when p_decision = 'published' then 'proposal_published' else 'proposal_rejected' end,
    v_actor_id, v_actor_role, btrim(p_audit_reference),
    jsonb_build_object(
      'execution_required', v_submission.execution_required,
      'manifest_sha256', v_submission.execution_manifest_sha256,
      'approval_is_execution', false,
      'execution_intent_created', false,
      'execution_receipt_created', false
    )
  );
  return query select v_submission.id, v_public_id, p_decision;
end;
$$;

create or replace function public.submit_governance_discussion_v1(
  p_proposal_id uuid,
  p_topic text,
  p_body text,
  p_public_body_consent boolean,
  p_public_wallet_consent boolean,
  p_submission_reference text
)
returns table (discussion_id uuid, moderation_status text)
language plpgsql
security definer
set search_path = public, pg_temp
as $$
declare
  v_actor_id uuid := auth.uid();
  v_wallet text := public.current_verified_solana_wallet();
  v_id uuid;
begin
  if v_actor_id is null or v_wallet is null then raise exception 'verified wallet authentication is required'; end if;
  if not public.is_operations_wallet_intake_enabled() then raise exception 'operations wallet intake is disabled'; end if;
  if p_public_body_consent is null or p_public_wallet_consent is null
    or (p_public_wallet_consent and not p_public_body_consent) then
    raise exception 'public wallet consent requires public body consent';
  end if;
  if p_submission_reference is null or char_length(btrim(p_submission_reference)) not between 10 and 200
    or p_submission_reference ~ '[[:cntrl:]]' then raise exception 'invalid discussion submission reference'; end if;
  if p_proposal_id is not null and not exists (
    select 1 from public.governance_proposals
    where id = p_proposal_id and publication_status = 'published' and status in ('discussion', 'voting')
  ) then raise exception 'discussion proposal is not publicly active'; end if;

  insert into public.governance_discussions (
    proposal_id, submitted_by, topic, body, wallet_address,
    public_body_consent, public_wallet_consent
  ) values (
    p_proposal_id, v_actor_id, btrim(p_topic), btrim(p_body), v_wallet,
    p_public_body_consent, p_public_wallet_consent
  ) returning id into v_id;
  insert into public.operations_governance_workflow_events (
    entity_type, private_entity_id, action, actor_id, actor_role,
    event_reference, event_payload
  ) values (
    'discussion', v_id, 'discussion_submitted', v_actor_id, 'wallet_submitter',
    btrim(p_submission_reference), jsonb_build_object(
      'public_body_consent', p_public_body_consent,
      'public_wallet_consent', p_public_wallet_consent
    )
  );
  return query select v_id, 'pending'::text;
end;
$$;

create or replace function public.review_governance_discussion_v1(
  p_discussion_id uuid,
  p_decision text,
  p_reviewer_notes text,
  p_public_topic text,
  p_public_body text,
  p_publication_basis text,
  p_audit_reference text
)
returns table (discussion_id uuid, publication_id uuid, moderation_status text)
language plpgsql
security definer
set search_path = public, pg_temp
as $$
declare
  v_actor_id uuid := auth.uid();
  v_actor_role text := auth.jwt() -> 'app_metadata' ->> 'operations_role';
  v_discussion public.governance_discussions%rowtype;
  v_public_id uuid;
begin
  if v_actor_id is null or v_actor_role is null or v_actor_role not in ('moderator', 'governance_admin') then
    raise exception 'operations role is not authorized to review governance discussions';
  end if;
  if p_decision is null or p_decision not in ('published', 'rejected') then
    raise exception 'discussion review decision must be published or rejected';
  end if;
  if p_audit_reference is null or char_length(btrim(p_audit_reference)) not between 10 and 200
    or p_audit_reference ~ '[[:cntrl:]]' then raise exception 'invalid discussion review audit reference'; end if;
  select * into v_discussion from public.governance_discussions where id = p_discussion_id for update;
  if not found then raise exception 'governance discussion not found'; end if;
  if v_discussion.moderation_status <> 'pending' then raise exception 'governance discussion is already terminal'; end if;
  if v_discussion.submitted_by = v_actor_id then raise exception 'moderators cannot review their own governance discussion'; end if;
  if p_reviewer_notes is null or char_length(btrim(p_reviewer_notes)) not between 1 and 5000 then
    raise exception 'reviewer notes are required';
  end if;

  if p_decision = 'published' then
    if not v_discussion.public_body_consent then
      raise exception 'sanitized discussion publication requires submitter consent';
    end if;
    insert into public.governance_discussion_publications (
      discussion_reference, proposal_id, topic, body, wallet_address,
      publication_basis
    ) values (
      btrim(p_audit_reference), v_discussion.proposal_id, btrim(p_public_topic),
      btrim(p_public_body), case when v_discussion.public_wallet_consent then v_discussion.wallet_address end,
      btrim(p_publication_basis)
    ) returning id into v_public_id;
  elsif nullif(btrim(p_public_topic), '') is not null
    or nullif(btrim(p_public_body), '') is not null
    or nullif(btrim(p_publication_basis), '') is not null then
    raise exception 'rejected discussion cannot create public content';
  end if;

  update public.governance_discussions
  set moderation_status = p_decision, moderated_by = v_actor_id,
      reviewer_notes = btrim(p_reviewer_notes)
  where id = v_discussion.id;
  insert into public.operations_governance_workflow_events (
    entity_type, private_entity_id, public_entity_id, action, actor_id,
    actor_role, event_reference, event_payload
  ) values (
    'discussion', v_discussion.id, v_public_id,
    case when p_decision = 'published' then 'discussion_published' else 'discussion_rejected' end,
    v_actor_id, v_actor_role, btrim(p_audit_reference),
    jsonb_build_object('private_content_published', false, 'wallet_published', v_discussion.public_wallet_consent and p_decision = 'published')
  );
  return query select v_discussion.id, v_public_id, p_decision;
end;
$$;

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
  v_actor_role text := auth.jwt() -> 'app_metadata' ->> 'operations_role';
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
    execution_reference, execution_manifest_sha256, finalization_reference
  ) values (
    v_proposal.id, p_decision, btrim(p_rationale), v_decision_hash,
    v_proposal.execution_required, v_proposal.execution_manifest_url,
    v_hash, btrim(p_finalization_reference)
  ) returning id into v_decision_id;

  insert into public.operations_governance_workflow_events (
    entity_type, private_entity_id, public_entity_id, action, actor_id,
    actor_role, event_reference, event_payload
  ) values (
    'decision', v_submission.id, v_decision_id, 'decision_finalized',
    v_actor_id, v_actor_role, btrim(p_finalization_reference),
    jsonb_build_object(
      'decision', p_decision, 'decision_hash', v_decision_hash,
      'manifest_sha256', v_hash, 'approval_is_execution', false,
      'approval_is_payment', false, 'execution_intent_created', false,
      'execution_receipt_created', false
    )
  );
  return query select v_decision_id, v_decision_hash, v_proposal.execution_required, false, false;
end;
$$;

comment on function public.finalize_governance_decision_v1(uuid,text,text,text,text) is
  'Finalizes an immutable off-chain governance decision with deterministic SHA-256 manifest binding. Approval is not execution or payment and creates no intent or receipt.';

revoke all on function public.submit_governance_proposal_v1(text,text,text,boolean,jsonb,text,boolean,text) from public, anon, authenticated, service_role;
grant execute on function public.submit_governance_proposal_v1(text,text,text,boolean,jsonb,text,boolean,text) to authenticated;
revoke all on function public.publish_governance_proposal_v1(uuid,text,text,text,text,text,text,text,text) from public, anon, authenticated, service_role;
grant execute on function public.publish_governance_proposal_v1(uuid,text,text,text,text,text,text,text,text) to authenticated;
revoke all on function public.submit_governance_discussion_v1(uuid,text,text,boolean,boolean,text) from public, anon, authenticated, service_role;
grant execute on function public.submit_governance_discussion_v1(uuid,text,text,boolean,boolean,text) to authenticated;
revoke all on function public.review_governance_discussion_v1(uuid,text,text,text,text,text,text) from public, anon, authenticated, service_role;
grant execute on function public.review_governance_discussion_v1(uuid,text,text,text,text,text,text) to authenticated;
revoke all on function public.finalize_governance_decision_v1(uuid,text,text,text,text) from public, anon, authenticated, service_role;
grant execute on function public.finalize_governance_decision_v1(uuid,text,text,text,text) to authenticated;

-- Exact, owner-bound, service-role-only cleanup for reserved Phase 2E-6C Staging fixtures.
create or replace function public.protect_governance_6c_immutable_v1()
returns trigger language plpgsql set search_path = public, pg_temp as $$
declare
  v_reference text := current_setting('alpha.governance_6c_cleanup_reference', true);
  v_owner name;
begin
  select pg_get_userbyid(proowner) into v_owner
  from pg_proc where oid = 'public.cleanup_governance_operations_staging_e2e_v1(text,uuid,uuid[],uuid[])'::regprocedure;
  if v_reference ~ '^phase-2e-6c-staging-e2e:[0-9]{13}-[0-9a-f]{8}$'
    and v_owner is not null and current_user = v_owner then
    return old;
  end if;
  raise exception 'immutable governance record cannot be changed';
end;
$$;

drop trigger governance_discussion_publications_immutable on public.governance_discussion_publications;
create trigger governance_discussion_publications_immutable
before update or delete on public.governance_discussion_publications
for each row execute function public.protect_governance_6c_immutable_v1();
drop trigger governance_decisions_immutable on public.governance_decisions;
create trigger governance_decisions_immutable
before update or delete on public.governance_decisions
for each row execute function public.protect_governance_6c_immutable_v1();
drop trigger operations_governance_workflow_events_immutable on public.operations_governance_workflow_events;
create trigger operations_governance_workflow_events_immutable
before update or delete on public.operations_governance_workflow_events
for each row execute function public.protect_governance_6c_immutable_v1();

create or replace function public.cleanup_governance_operations_staging_e2e_v1(
  p_run_reference text,
  p_owner_id uuid,
  p_proposal_submission_ids uuid[],
  p_discussion_ids uuid[]
)
returns table (
  events_deleted integer, decisions_deleted integer,
  discussion_publications_deleted integer, discussions_deleted integer,
  proposals_deleted integer, proposal_submissions_deleted integer
)
language plpgsql security definer set search_path = public, pg_temp
as $$
declare
  v_run_id text;
  v_public_proposal_ids uuid[];
  v_public_discussion_ids uuid[];
  v_events integer; v_decisions integer; v_discussion_publications integer;
  v_discussions integer; v_proposals integer; v_submissions integer;
begin
  if p_run_reference !~ '^phase-2e-6c-staging-e2e:[0-9]{13}-[0-9a-f]{8}$' then
    raise exception 'invalid Phase 2E-6C Staging E2E cleanup reference';
  end if;
  if p_owner_id is null or cardinality(p_proposal_submission_ids) <> 2
    or cardinality(p_discussion_ids) <> 2
    or cardinality(array(select distinct value from unnest(p_proposal_submission_ids) value)) <> 2
    or cardinality(array(select distinct value from unnest(p_discussion_ids) value)) <> 2 then
    raise exception 'invalid Phase 2E-6C Staging E2E cleanup identifiers';
  end if;
  v_run_id := split_part(p_run_reference, ':', 2);

  if (select count(*) from public.governance_proposal_submissions s
      where s.id = any(p_proposal_submission_ids) and s.submitted_by = p_owner_id
        and s.title in ('Staging governance publish ' || v_run_id, 'Staging governance reject ' || v_run_id)) <> 2 then
    raise exception 'cleanup proposals are not exact owner-bound Phase 2E-6C fixtures';
  end if;
  if (select count(*) from public.governance_discussions d
      where d.id = any(p_discussion_ids) and d.submitted_by = p_owner_id
        and d.topic in ('Staging discussion publish ' || v_run_id, 'Staging discussion reject ' || v_run_id)) <> 2 then
    raise exception 'cleanup discussions are not exact owner-bound Phase 2E-6C fixtures';
  end if;

  select coalesce(array_agg(s.published_proposal_id) filter (where s.published_proposal_id is not null), '{}')
  into v_public_proposal_ids from public.governance_proposal_submissions s
  where s.id = any(p_proposal_submission_ids);
  select coalesce(array_agg(e.public_entity_id) filter (where e.action = 'discussion_published'), '{}')
  into v_public_discussion_ids from public.operations_governance_workflow_events e
  where e.private_entity_id = any(p_discussion_ids) and e.event_reference like p_run_reference || ':%';

  if exists (select 1 from public.operations_governance_workflow_events e
    where (e.private_entity_id = any(p_proposal_submission_ids) or e.private_entity_id = any(p_discussion_ids)
      or e.public_entity_id = any(v_public_proposal_ids) or e.public_entity_id = any(v_public_discussion_ids))
      and e.event_reference not like p_run_reference || ':%') then
    raise exception 'Phase 2E-6C cleanup reference is not isolated';
  end if;

  perform set_config('alpha.governance_6c_cleanup_reference', p_run_reference, true);
  delete from public.operations_governance_workflow_events e
  where e.private_entity_id = any(p_proposal_submission_ids) or e.private_entity_id = any(p_discussion_ids)
    or e.public_entity_id = any(v_public_proposal_ids) or e.public_entity_id = any(v_public_discussion_ids);
  get diagnostics v_events = row_count;
  delete from public.governance_decisions d where d.proposal_id = any(v_public_proposal_ids);
  get diagnostics v_decisions = row_count;
  delete from public.governance_discussion_publications p where p.id = any(v_public_discussion_ids);
  get diagnostics v_discussion_publications = row_count;
  delete from public.governance_discussions d where d.id = any(p_discussion_ids);
  get diagnostics v_discussions = row_count;

  update public.governance_proposal_submissions set published_proposal_id = null
  where id = any(p_proposal_submission_ids);
  delete from public.governance_proposals p where p.id = any(v_public_proposal_ids);
  get diagnostics v_proposals = row_count;
  delete from public.governance_proposal_submissions s where s.id = any(p_proposal_submission_ids);
  get diagnostics v_submissions = row_count;
  perform set_config('alpha.governance_6c_cleanup_reference', '', true);

  if v_discussions <> 2 or v_submissions <> 2 then
    raise exception 'Phase 2E-6C Staging E2E cleanup count mismatch';
  end if;
  return query select v_events, v_decisions, v_discussion_publications,
    v_discussions, v_proposals, v_submissions;
end;
$$;

comment on function public.cleanup_governance_operations_staging_e2e_v1(text,uuid,uuid[],uuid[]) is
  'Service-role-only atomic cleanup for exact, owner-bound Phase 2E-6C Staging governance fixtures.';
revoke all on function public.cleanup_governance_operations_staging_e2e_v1(text,uuid,uuid[],uuid[]) from public, anon, authenticated, service_role;
grant execute on function public.cleanup_governance_operations_staging_e2e_v1(text,uuid,uuid[],uuid[]) to service_role;

create or replace function public.cleanup_governance_discussion_staging_e2e_v1(
  p_run_reference text,
  p_owner_id uuid,
  p_discussion_id uuid
)
returns table (events_deleted integer, publications_deleted integer, discussions_deleted integer)
language plpgsql security definer set search_path = public, pg_temp
as $$
declare
  v_publication_id uuid;
  v_events integer; v_publications integer; v_discussions integer;
begin
  if p_run_reference !~ '^phase-2e-6b-4m-staging-e2e:[0-9]{13}-[0-9a-f]{8}$' then
    raise exception 'invalid controlled discussion cleanup reference';
  end if;
  if not exists (
    select 1 from public.governance_discussions d
    where d.id = p_discussion_id and d.submitted_by = p_owner_id
      and d.topic = 'Staging moderation ' || split_part(p_run_reference, ':', 2)
  ) then raise exception 'discussion cleanup target is not exact and owner-bound'; end if;
  select e.public_entity_id into v_publication_id
  from public.operations_governance_workflow_events e
  where e.private_entity_id = p_discussion_id and e.action = 'discussion_published'
    and e.event_reference like p_run_reference || ':%';
  if exists (select 1 from public.operations_governance_workflow_events e
    where e.private_entity_id = p_discussion_id and e.event_reference not like p_run_reference || ':%') then
    raise exception 'discussion cleanup reference is not isolated';
  end if;
  perform set_config('alpha.governance_6c_cleanup_reference',
    'phase-2e-6c-staging-e2e:' || split_part(p_run_reference, ':', 2), true);
  delete from public.operations_governance_workflow_events where private_entity_id = p_discussion_id;
  get diagnostics v_events = row_count;
  delete from public.governance_discussion_publications where id = v_publication_id;
  get diagnostics v_publications = row_count;
  delete from public.governance_discussions where id = p_discussion_id;
  get diagnostics v_discussions = row_count;
  perform set_config('alpha.governance_6c_cleanup_reference', '', true);
  if v_events <> 2 or v_publications <> 1 or v_discussions <> 1 then
    raise exception 'controlled discussion cleanup count mismatch';
  end if;
  return query select v_events, v_publications, v_discussions;
end;
$$;
revoke all on function public.cleanup_governance_discussion_staging_e2e_v1(text,uuid,uuid) from public, anon, authenticated, service_role;
grant execute on function public.cleanup_governance_discussion_staging_e2e_v1(text,uuid,uuid) to service_role;

revoke delete on table public.governance_proposal_submissions,
  public.governance_proposals, public.governance_discussions,
  public.governance_discussion_publications, public.governance_decisions,
  public.operations_governance_workflow_events from service_role;
