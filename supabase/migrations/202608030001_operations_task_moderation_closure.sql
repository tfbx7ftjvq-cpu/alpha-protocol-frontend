-- Alpha Protocol Phase 2E-6B-4M
-- Audited community-task publication and moderation closure.
--
-- This migration keeps raw submissions private, publishes only contributor-
-- consented sanitized results, and records immutable workflow events. It does
-- not create a treasury intent, send a Solana transaction, or move funds.

begin;

alter table public.task_submissions
  add column public_result_consent boolean not null default false,
  add column public_wallet_consent boolean not null default false,
  add constraint task_submissions_wallet_consent_requires_result_consent
    check (not public_wallet_consent or public_result_consent),
  add constraint task_submissions_deliverable_https_safe
    check (
      deliverable_url ~ '^https://[^[:space:]@/]+(/[^[:space:]]*)?$'
      and deliverable_url !~ '[[:cntrl:]]'
    );

comment on column public.task_submissions.public_result_consent is
  'Contributor consent for a reviewer-written sanitized result and deliverable URL to be published after acceptance.';
comment on column public.task_submissions.public_wallet_consent is
  'Separate optional consent to include the submission wallet in the sanitized public result.';

create table public.task_submission_publications (
  id uuid primary key default gen_random_uuid(),
  task_id uuid not null references public.community_tasks(id),
  task_title text not null check (char_length(trim(task_title)) between 4 and 160),
  result_summary text not null check (char_length(trim(result_summary)) between 20 and 3000),
  deliverable_url text not null check (
    deliverable_url ~ '^https://[^[:space:]@/]+(/[^[:space:]]*)?$'
    and deliverable_url !~ '[[:cntrl:]]'
    and char_length(deliverable_url) <= 2000
  ),
  wallet_address text check (
    wallet_address is null or char_length(wallet_address) between 32 and 44
  ),
  review_reference text not null unique check (
    char_length(trim(review_reference)) between 10 and 180
    and review_reference !~ '[[:cntrl:]]'
  ),
  accepted_at timestamptz not null,
  published_at timestamptz not null default timezone('utc', now())
);

comment on table public.task_submission_publications is
  'Immutable sanitized public task results. Contains no Auth user foreign key or private submission identifier.';

create table public.operations_task_workflow_events (
  event_id bigint generated always as identity primary key,
  entity_type text not null check (entity_type in ('community_task', 'task_submission')),
  entity_reference uuid not null,
  action text not null check (
    action in (
      'task_published',
      'submission_accepted',
      'submission_rejected',
      'result_published'
    )
  ),
  actor_id uuid not null references auth.users(id),
  actor_role text not null check (
    actor_role in ('reviewer', 'operator', 'governance_admin')
  ),
  event_reference text not null unique check (
    char_length(trim(event_reference)) between 10 and 200
    and event_reference !~ '[[:cntrl:]]'
  ),
  event_data jsonb not null default '{}'::jsonb check (jsonb_typeof(event_data) = 'object'),
  created_at timestamptz not null default timezone('utc', now())
);

comment on table public.operations_task_workflow_events is
  'Private append-only task publication and review audit history. It is evidence, never payment authority.';

create trigger task_submission_publications_immutable
before update or delete on public.task_submission_publications
for each row execute function public.reject_immutable_operations_mutation();

create trigger operations_task_workflow_events_immutable
before update or delete on public.operations_task_workflow_events
for each row execute function public.reject_immutable_operations_mutation();

create index task_submission_publications_published_at_idx
on public.task_submission_publications (published_at desc);

create index task_submission_publications_task_idx
on public.task_submission_publications (task_id, published_at desc);

create index operations_task_workflow_events_entity_idx
on public.operations_task_workflow_events (entity_type, entity_reference, created_at desc);

alter table public.task_submission_publications enable row level security;
alter table public.operations_task_workflow_events enable row level security;

create policy task_submission_publications_public_read
on public.task_submission_publications
for select
to anon, authenticated
using (true);

create policy operations_task_workflow_events_staff_read
on public.operations_task_workflow_events
for select
to authenticated
using (public.has_operations_role(array['reviewer', 'operator', 'governance_admin']));

revoke all on table public.task_submission_publications
from public, anon, authenticated;
revoke all on table public.operations_task_workflow_events
from public, anon, authenticated;

grant select on table public.task_submission_publications to anon, authenticated;
grant select on table public.operations_task_workflow_events to authenticated;

-- Existing RLS policies remain as defense in depth, while direct staff writes
-- are removed so that validation, locking, publication, and audit are atomic.
revoke insert, update, delete on table public.community_tasks from authenticated;
revoke update, delete on table public.task_submissions from authenticated;

create or replace function public.publish_community_task_v1(
  p_title text,
  p_summary text,
  p_requirements text,
  p_reward_budget_usdc numeric,
  p_reward_source text,
  p_submission_deadline timestamptz,
  p_audit_reference text
)
returns uuid
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_actor_id uuid := auth.uid();
  v_actor_role text := auth.jwt() -> 'app_metadata' ->> 'operations_role';
  v_title text := trim(p_title);
  v_summary text := trim(p_summary);
  v_requirements text := trim(p_requirements);
  v_audit_reference text := trim(p_audit_reference);
  v_task_id uuid;
begin
  -- Do not use only NOT IN here: NULL NOT IN (...) evaluates to NULL and can
  -- bypass a PL/pgSQL IF. The explicit NULL branch is security-critical.
  if v_actor_id is null
    or v_actor_role is null
    or v_actor_role not in ('operator', 'governance_admin')
  then
    raise exception 'operations role is not authorized to publish tasks';
  end if;

  if char_length(v_title) not between 4 and 160
    or char_length(v_summary) not between 20 and 3000
    or char_length(v_requirements) not between 20 and 5000
  then
    raise exception 'community task content is outside allowed bounds';
  end if;

  if p_reward_budget_usdc is not null
    and (
      p_reward_budget_usdc < 0
      or p_reward_budget_usdc > 1000000000
      or p_reward_budget_usdc <> trunc(p_reward_budget_usdc, 6)
    )
  then
    raise exception 'reward budget must be between 0 and 1000000000 USDC with at most 6 decimals';
  end if;

  if p_reward_source is null
    or p_reward_source not in ('builders_pool', 'grant', 'sponsor', 'none')
  then
    raise exception 'unsupported community task reward source';
  end if;

  if p_submission_deadline is not null
    and p_submission_deadline <= timezone('utc', now())
  then
    raise exception 'community task submission deadline must be in the future';
  end if;

  if v_audit_reference is null
    or char_length(v_audit_reference) not between 10 and 180
    or v_audit_reference ~ '[[:cntrl:]]'
  then
    raise exception 'task publication audit reference must be 10 to 180 characters without control characters';
  end if;

  insert into public.community_tasks (
    title,
    summary,
    requirements,
    reward_budget_usdc,
    reward_source,
    status,
    publication_status,
    submission_deadline,
    published_at
  ) values (
    v_title,
    v_summary,
    v_requirements,
    p_reward_budget_usdc,
    p_reward_source,
    'open',
    'published',
    p_submission_deadline,
    timezone('utc', now())
  )
  returning id into v_task_id;

  insert into public.operations_task_workflow_events (
    entity_type,
    entity_reference,
    action,
    actor_id,
    actor_role,
    event_reference,
    event_data
  ) values (
    'community_task',
    v_task_id,
    'task_published',
    v_actor_id,
    v_actor_role,
    v_audit_reference,
    jsonb_build_object(
      'reward_source', p_reward_source,
      'reward_budget_usdc', p_reward_budget_usdc,
      'submission_deadline', p_submission_deadline
    )
  );

  return v_task_id;
end;
$$;

comment on function public.publish_community_task_v1(text, text, text, numeric, text, timestamptz, text) is
  'Role-gated atomic community task publication with an immutable audit event. No treasury action.';

revoke all on function public.publish_community_task_v1(text, text, text, numeric, text, timestamptz, text) from public;
revoke all on function public.publish_community_task_v1(text, text, text, numeric, text, timestamptz, text) from anon;
grant execute on function public.publish_community_task_v1(text, text, text, numeric, text, timestamptz, text) to authenticated;

create or replace function public.review_task_submission_v1(
  p_submission_id uuid,
  p_decision text,
  p_reviewer_notes text,
  p_public_result_summary text,
  p_public_deliverable_url text,
  p_audit_reference text
)
returns table (
  submission_id uuid,
  submission_status text,
  publication_id uuid
)
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_actor_id uuid := auth.uid();
  v_actor_role text := auth.jwt() -> 'app_metadata' ->> 'operations_role';
  v_notes text := nullif(trim(p_reviewer_notes), '');
  v_public_summary text := nullif(trim(p_public_result_summary), '');
  v_public_url text := nullif(trim(p_public_deliverable_url), '');
  v_audit_reference text := trim(p_audit_reference);
  v_submission public.task_submissions%rowtype;
  v_task public.community_tasks%rowtype;
  v_publication_id uuid;
  v_reviewed_at timestamptz := timezone('utc', now());
begin
  -- Explicit NULL handling prevents an unauthenticated/no-role bypass.
  if v_actor_id is null
    or v_actor_role is null
    or v_actor_role not in ('reviewer', 'operator', 'governance_admin')
  then
    raise exception 'operations role is not authorized to review task submissions';
  end if;

  if p_decision is null or p_decision not in ('accepted', 'rejected') then
    raise exception 'task submission decision must be accepted or rejected';
  end if;

  if v_notes is null or char_length(v_notes) > 5000 then
    raise exception 'reviewer notes are required and cannot exceed 5000 characters';
  end if;

  if v_audit_reference is null
    or char_length(v_audit_reference) not between 10 and 160
    or v_audit_reference ~ '[[:cntrl:]]'
  then
    raise exception 'task review audit reference must be 10 to 160 characters without control characters';
  end if;

  select submission.*
  into v_submission
  from public.task_submissions submission
  where submission.id = p_submission_id
  for update;

  if not found then
    raise exception 'task submission was not found';
  end if;

  if v_submission.submitted_by = v_actor_id then
    raise exception 'reviewers cannot review their own task submission';
  end if;

  if v_submission.status not in ('submitted', 'in_review') then
    raise exception 'task submission is already in a terminal review state';
  end if;

  select task.*
  into v_task
  from public.community_tasks task
  where task.id = v_submission.task_id
  for share;

  if not found then
    raise exception 'task referenced by submission was not found';
  end if;

  if p_decision = 'accepted' then
    if not v_submission.public_result_consent then
      raise exception 'accepted task result requires contributor public result consent';
    end if;

    if v_public_summary is null
      or char_length(v_public_summary) not between 20 and 3000
    then
      raise exception 'accepted task result requires a sanitized public summary of 20 to 3000 characters';
    end if;

    if v_public_url is null
      or char_length(v_public_url) > 2000
      or v_public_url !~ '^https://[^[:space:]@/]+(/[^[:space:]]*)?$'
      or v_public_url ~ '[[:cntrl:]]'
    then
      raise exception 'accepted task result requires a safe HTTPS public deliverable URL';
    end if;
  elsif v_public_summary is not null or v_public_url is not null then
    raise exception 'rejected task submissions cannot publish a public result';
  end if;

  if v_submission.status = 'submitted' then
    update public.task_submissions submission
    set status = 'in_review'
    where submission.id = v_submission.id;
  end if;

  update public.task_submissions submission
  set
    status = p_decision,
    reviewer_notes = v_notes,
    reviewed_by = v_actor_id,
    reviewed_at = v_reviewed_at
  where submission.id = v_submission.id;

  insert into public.operations_task_workflow_events (
    entity_type,
    entity_reference,
    action,
    actor_id,
    actor_role,
    event_reference,
    event_data
  ) values (
    'task_submission',
    v_submission.id,
    case when p_decision = 'accepted' then 'submission_accepted' else 'submission_rejected' end,
    v_actor_id,
    v_actor_role,
    v_audit_reference || ':decision',
    jsonb_build_object(
      'task_id', v_submission.task_id,
      'decision', p_decision,
      'public_result_consent', v_submission.public_result_consent,
      'public_wallet_consent', v_submission.public_wallet_consent
    )
  );

  if p_decision = 'accepted' then
    insert into public.task_submission_publications (
      task_id,
      task_title,
      result_summary,
      deliverable_url,
      wallet_address,
      review_reference,
      accepted_at
    ) values (
      v_submission.task_id,
      v_task.title,
      v_public_summary,
      v_public_url,
      case when v_submission.public_wallet_consent then v_submission.wallet_address else null end,
      v_audit_reference,
      v_reviewed_at
    )
    returning id into v_publication_id;

    insert into public.operations_task_workflow_events (
      entity_type,
      entity_reference,
      action,
      actor_id,
      actor_role,
      event_reference,
      event_data
    ) values (
      'task_submission',
      v_submission.id,
      'result_published',
      v_actor_id,
      v_actor_role,
      v_audit_reference || ':publication',
      jsonb_build_object(
        'task_id', v_submission.task_id,
        'publication_id', v_publication_id,
        'wallet_published', v_submission.public_wallet_consent
      )
    );
  end if;

  return query
  select v_submission.id, p_decision, v_publication_id;
end;
$$;

comment on function public.review_task_submission_v1(uuid, text, text, text, text, text) is
  'Role-gated atomic task review. Accepted results require contributor consent and create a sanitized immutable publication; accepted never means paid.';

revoke all on function public.review_task_submission_v1(uuid, text, text, text, text, text) from public;
revoke all on function public.review_task_submission_v1(uuid, text, text, text, text, text) from anon;
grant execute on function public.review_task_submission_v1(uuid, text, text, text, text, text) to authenticated;

commit;
