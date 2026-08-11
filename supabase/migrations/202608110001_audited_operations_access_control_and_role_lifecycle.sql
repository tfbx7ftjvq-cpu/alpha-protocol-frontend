-- Alpha Protocol Phase 2E-6E
-- Audited operations access control and role lifecycle.
--
-- This migration replaces JWT app_metadata.operations_role as the final
-- operations authorization source with an audited database registry. Supabase
-- Web3 wallet authentication remains unchanged. No network request, email,
-- Solana transaction, or treasury movement is introduced here.

begin;

do $$
declare
  v_legacy_claim_count integer := 0;
begin
  if to_regclass('public.operations_role_assignments') is not null
    or to_regclass('public.operations_role_assignment_events') is not null
  then
    raise exception
      'unexpected preexisting operations role lifecycle tables; inspect prior role state before applying Phase 2E-6E';
  end if;

  if exists (
    select 1
    from information_schema.columns
    where table_schema = 'auth'
      and table_name = 'users'
      and column_name = 'raw_app_meta_data'
  ) then
    execute $inventory$
      select count(*)::integer
      from auth.users user_record
      where nullif(
        btrim(coalesce(user_record.raw_app_meta_data ->> 'operations_role', '')),
        ''
      ) is not null
    $inventory$
    into v_legacy_claim_count;
  end if;

  if v_legacy_claim_count > 0 then
    raise exception
      'legacy operations_role JWT claims detected; refuse automatic conversion. Manually inventory those users and re-grant audited access with grant_operations_role_v1 after Phase 2E-6E.';
  end if;
end;
$$;

create table public.operations_role_assignments (
  assignment_id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users(id),
  role_name text not null
    check (
      role_name in (
        'reviewer',
        'relief_reviewer',
        'operator',
        'moderator',
        'governance_admin',
        'treasury_preparer',
        'treasury_authorizer',
        'executor',
        'treasury_reconciler'
      )
    ),
  status text not null
    check (status in ('active', 'revoked', 'expired')),
  granted_at timestamptz not null default timezone('utc', now()),
  expires_at timestamptz,
  revoked_at timestamptz,
  grant_reference text not null
    check (
      char_length(trim(grant_reference)) between 10 and 200
      and grant_reference !~ '[[:cntrl:]]'
    ),
  revoke_reference text
    check (
      revoke_reference is null
      or (
        char_length(trim(revoke_reference)) between 10 and 200
        and revoke_reference !~ '[[:cntrl:]]'
      )
    ),
  created_at timestamptz not null default timezone('utc', now()),
  check (expires_at is null or expires_at > granted_at),
  check (
    (status = 'active' and revoked_at is null and revoke_reference is null)
    or (status = 'revoked' and revoked_at is not null and revoke_reference is not null)
    or (status = 'expired' and revoked_at is null)
  )
);

comment on table public.operations_role_assignments is
  'Audited database source of truth for operations staff access. JWT app_metadata.operations_role has no authorization effect after Phase 2E-6E.';

create unique index operations_role_assignments_one_active_per_user
on public.operations_role_assignments (user_id)
where status = 'active';

create index operations_role_assignments_user_status_granted_idx
on public.operations_role_assignments (user_id, status, granted_at desc);

create table public.operations_role_assignment_events (
  event_id bigint generated always as identity primary key,
  assignment_id uuid not null references public.operations_role_assignments(assignment_id),
  user_id uuid not null references auth.users(id),
  role_name text not null
    check (
      role_name in (
        'reviewer',
        'relief_reviewer',
        'operator',
        'moderator',
        'governance_admin',
        'treasury_preparer',
        'treasury_authorizer',
        'executor',
        'treasury_reconciler'
      )
    ),
  event_type text not null
    check (event_type in ('granted', 'revoked', 'expired')),
  previous_status text
    check (previous_status is null or previous_status in ('active', 'revoked', 'expired')),
  new_status text not null
    check (new_status in ('active', 'revoked', 'expired')),
  change_reference text not null
    check (
      char_length(trim(change_reference)) between 10 and 200
      and change_reference !~ '[[:cntrl:]]'
    ),
  actor_type text not null
    check (actor_type = 'service_role'),
  actor_user_id uuid references auth.users(id),
  changed_at timestamptz not null default timezone('utc', now()),
  check (
    (actor_type = 'service_role' and actor_user_id is null)
    or (actor_type <> 'service_role' and actor_user_id is not null)
  ),
  check (
    (event_type = 'granted' and previous_status is null and new_status = 'active')
    or (event_type = 'revoked' and previous_status = 'active' and new_status = 'revoked')
    or (event_type = 'expired' and previous_status = 'active' and new_status = 'expired')
  )
);

comment on table public.operations_role_assignment_events is
  'Append-only audit history for service-role-managed operations access changes.';

alter table public.operations_role_assignments enable row level security;
alter table public.operations_role_assignment_events enable row level security;

revoke all on table public.operations_role_assignments
from public, anon, authenticated, service_role;
revoke all on table public.operations_role_assignment_events
from public, anon, authenticated, service_role;

grant select on table public.operations_role_assignments to service_role;
grant select on table public.operations_role_assignment_events to service_role;

create trigger operations_role_assignment_events_immutable
before update or delete on public.operations_role_assignment_events
for each row execute function public.reject_immutable_operations_mutation();

create or replace function public.is_service_role_session_v1()
returns boolean
language sql
stable
security definer
set search_path = ''
as $$
  select current_user = 'service_role'
    or coalesce(auth.jwt() ->> 'role', '') = 'service_role';
$$;

revoke all on function public.is_service_role_session_v1() from public;

create or replace function public.current_operations_role_v1()
returns text
language plpgsql
stable
security definer
set search_path = ''
as $$
declare
  v_actor_id uuid := auth.uid();
  v_role text;
  v_active_count integer := 0;
begin
  if v_actor_id is null then
    return null;
  end if;

  select
    count(*)::integer,
    min(assignment.role_name)
  into v_active_count, v_role
  from public.operations_role_assignments assignment
  where assignment.user_id = v_actor_id
    and assignment.status = 'active'
    and (
      assignment.expires_at is null
      or assignment.expires_at > timezone('utc', now())
    );

  if v_active_count = 1 then
    return v_role;
  end if;

  return null;
end;
$$;

comment on function public.current_operations_role_v1() is
  'Fail-closed resolver for the caller''s current audited operations role. JWT app_metadata.operations_role is ignored.';

revoke all on function public.current_operations_role_v1() from public;
grant execute on function public.current_operations_role_v1() to anon, authenticated;

create or replace function public.has_operations_role(allowed_roles text[])
returns boolean
language sql
stable
set search_path = ''
as $$
  select coalesce(
    public.current_operations_role_v1() = any(allowed_roles),
    false
  );
$$;

revoke all on function public.has_operations_role(text[]) from public;
grant execute on function public.has_operations_role(text[]) to anon, authenticated;

create or replace function public.get_my_operations_access_v1()
returns table (
  role_name text,
  status text,
  expires_at timestamptz
)
language plpgsql
stable
security definer
set search_path = ''
as $$
declare
  v_actor_id uuid := auth.uid();
  v_active_count integer := 0;
  v_assignment public.operations_role_assignments%rowtype;
begin
  if v_actor_id is null then
    return;
  end if;

  select count(*)::integer
  into v_active_count
  from public.operations_role_assignments assignment
  where assignment.user_id = v_actor_id
    and assignment.status = 'active'
    and (
      assignment.expires_at is null
      or assignment.expires_at > timezone('utc', now())
    );

  if v_active_count = 1 then
    return query
    select
      assignment.role_name,
      'active'::text,
      assignment.expires_at
    from public.operations_role_assignments assignment
    where assignment.user_id = v_actor_id
      and assignment.status = 'active'
      and (
        assignment.expires_at is null
        or assignment.expires_at > timezone('utc', now())
      )
    limit 1;
    return;
  end if;

  if v_active_count > 1 then
    return;
  end if;

  select assignment.*
  into v_assignment
  from public.operations_role_assignments assignment
  where assignment.user_id = v_actor_id
  order by assignment.granted_at desc, assignment.created_at desc
  limit 1;

  if not found then
    return;
  end if;

  if v_assignment.status = 'active'
    and v_assignment.expires_at is not null
    and v_assignment.expires_at <= timezone('utc', now())
  then
    return query select v_assignment.role_name, 'expired'::text, v_assignment.expires_at;
  elsif v_assignment.status in ('revoked', 'expired') then
    return query select v_assignment.role_name, v_assignment.status, v_assignment.expires_at;
  end if;
end;
$$;

comment on function public.get_my_operations_access_v1() is
  'Authenticated caller view of only their own audited operations access state.';

revoke all on function public.get_my_operations_access_v1() from public;
revoke all on function public.get_my_operations_access_v1() from anon;
grant execute on function public.get_my_operations_access_v1() to authenticated;

create or replace function public.grant_operations_role_v1(
  p_user_id uuid,
  p_role_name text,
  p_grant_reference text,
  p_expires_at timestamptz default null
)
returns table (
  assignment_id uuid,
  user_id uuid,
  role_name text,
  status text,
  expires_at timestamptz
)
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_role_name text := nullif(trim(p_role_name), '');
  v_reference text := trim(p_grant_reference);
  v_now timestamptz := timezone('utc', now());
  v_existing public.operations_role_assignments%rowtype;
  v_active_count integer := 0;
  v_created public.operations_role_assignments%rowtype;
begin
  if not public.is_service_role_session_v1() then
    raise exception 'granting operations roles requires the service_role credential';
  end if;

  if p_user_id is null then
    raise exception 'operations role grant target is required';
  end if;

  if v_role_name is null
    or v_role_name not in (
      'reviewer',
      'relief_reviewer',
      'operator',
      'moderator',
      'governance_admin',
      'treasury_preparer',
      'treasury_authorizer',
      'executor',
      'treasury_reconciler'
    )
  then
    raise exception 'unsupported operations role';
  end if;

  if v_reference is null
    or char_length(v_reference) not between 10 and 200
    or v_reference ~ '[[:cntrl:]]'
  then
    raise exception 'operations role grant reference must be 10 to 200 characters without control characters';
  end if;

  if p_expires_at is not null and p_expires_at <= v_now then
    raise exception 'operations role expiration must be in the future';
  end if;

  perform 1 from auth.users user_record where user_record.id = p_user_id;
  if not found then
    raise exception 'operations role grant target user was not found';
  end if;

  for v_existing in
    select assignment.*
    from public.operations_role_assignments assignment
    where assignment.user_id = p_user_id
      and assignment.status = 'active'
    for update
  loop
    if v_existing.expires_at is not null and v_existing.expires_at <= v_now then
      update public.operations_role_assignments assignment
      set status = 'expired'
      where assignment.assignment_id = v_existing.assignment_id;

      insert into public.operations_role_assignment_events (
        assignment_id,
        user_id,
        role_name,
        event_type,
        previous_status,
        new_status,
        change_reference,
        actor_type,
        actor_user_id,
        changed_at
      ) values (
        v_existing.assignment_id,
        v_existing.user_id,
        v_existing.role_name,
        'expired',
        'active',
        'expired',
        v_reference,
        'service_role',
        null,
        v_now
      );
    end if;
  end loop;

  select count(*)::integer
  into v_active_count
  from public.operations_role_assignments assignment
  where assignment.user_id = p_user_id
    and assignment.status = 'active'
    and (
      assignment.expires_at is null
      or assignment.expires_at > v_now
    );

  if v_active_count <> 0 then
    raise exception 'user already has an active operations role assignment';
  end if;

  insert into public.operations_role_assignments (
    user_id,
    role_name,
    status,
    granted_at,
    expires_at,
    grant_reference,
    created_at
  ) values (
    p_user_id,
    v_role_name,
    'active',
    v_now,
    p_expires_at,
    v_reference,
    v_now
  )
  returning *
  into v_created;

  insert into public.operations_role_assignment_events (
    assignment_id,
    user_id,
    role_name,
    event_type,
    previous_status,
    new_status,
    change_reference,
    actor_type,
    actor_user_id,
    changed_at
  ) values (
    v_created.assignment_id,
    v_created.user_id,
    v_created.role_name,
    'granted',
    null,
    'active',
    v_reference,
    'service_role',
    null,
    v_now
  );

  return query
  select
    v_created.assignment_id,
    v_created.user_id,
    v_created.role_name,
    v_created.status,
    v_created.expires_at;
end;
$$;

comment on function public.grant_operations_role_v1(uuid, text, text, timestamptz) is
  'Service-role-only audited operations access grant. Legacy JWT claims are not trusted.';

revoke all on function public.grant_operations_role_v1(uuid, text, text, timestamptz)
from public, anon, authenticated, service_role;
grant execute on function public.grant_operations_role_v1(uuid, text, text, timestamptz)
to service_role;

create or replace function public.revoke_operations_role_v1(
  p_user_id uuid,
  p_revoke_reference text
)
returns table (
  assignment_id uuid,
  user_id uuid,
  role_name text,
  status text,
  revoked_at timestamptz
)
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_reference text := trim(p_revoke_reference);
  v_now timestamptz := timezone('utc', now());
  v_active public.operations_role_assignments%rowtype;
  v_active_count integer := 0;
begin
  if not public.is_service_role_session_v1() then
    raise exception 'revoking operations roles requires the service_role credential';
  end if;

  if p_user_id is null then
    raise exception 'operations role revoke target is required';
  end if;

  if v_reference is null
    or char_length(v_reference) not between 10 and 200
    or v_reference ~ '[[:cntrl:]]'
  then
    raise exception 'operations role revoke reference must be 10 to 200 characters without control characters';
  end if;

  for v_active in
    select assignment.*
    from public.operations_role_assignments assignment
    where assignment.user_id = p_user_id
      and assignment.status = 'active'
    for update
  loop
    if v_active.expires_at is not null and v_active.expires_at <= v_now then
      update public.operations_role_assignments assignment
      set status = 'expired'
      where assignment.assignment_id = v_active.assignment_id;

      insert into public.operations_role_assignment_events (
        assignment_id,
        user_id,
        role_name,
        event_type,
        previous_status,
        new_status,
        change_reference,
        actor_type,
        actor_user_id,
        changed_at
      ) values (
        v_active.assignment_id,
        v_active.user_id,
        v_active.role_name,
        'expired',
        'active',
        'expired',
        v_reference,
        'service_role',
        null,
        v_now
      );
    end if;
  end loop;

  select count(*)::integer
  into v_active_count
  from public.operations_role_assignments assignment
  where assignment.user_id = p_user_id
    and assignment.status = 'active'
    and (
      assignment.expires_at is null
      or assignment.expires_at > v_now
    );

  if v_active_count <> 1 then
    raise exception 'user must have exactly one active operations role assignment to revoke';
  end if;

  select assignment.*
  into v_active
  from public.operations_role_assignments assignment
  where assignment.user_id = p_user_id
    and assignment.status = 'active'
    and (
      assignment.expires_at is null
      or assignment.expires_at > v_now
    )
  for update;

  update public.operations_role_assignments assignment
  set
    status = 'revoked',
    revoked_at = v_now,
    revoke_reference = v_reference
  where assignment.assignment_id = v_active.assignment_id
  returning *
  into v_active;

  insert into public.operations_role_assignment_events (
    assignment_id,
    user_id,
    role_name,
    event_type,
    previous_status,
    new_status,
    change_reference,
    actor_type,
    actor_user_id,
    changed_at
  ) values (
    v_active.assignment_id,
    v_active.user_id,
    v_active.role_name,
    'revoked',
    'active',
    'revoked',
    v_reference,
    'service_role',
    null,
    v_now
  );

  return query
  select
    v_active.assignment_id,
    v_active.user_id,
    v_active.role_name,
    v_active.status,
    v_active.revoked_at;
end;
$$;

comment on function public.revoke_operations_role_v1(uuid, text) is
  'Service-role-only audited operations access revocation. Revocation takes effect immediately.';

revoke all on function public.revoke_operations_role_v1(uuid, text)
from public, anon, authenticated, service_role;
grant execute on function public.revoke_operations_role_v1(uuid, text)
to service_role;

create or replace function public.inspect_operations_role_v1(
  p_user_id uuid default null
)
returns table (
  assignment_id uuid,
  user_id uuid,
  role_name text,
  status text,
  granted_at timestamptz,
  expires_at timestamptz,
  revoked_at timestamptz,
  grant_reference text,
  revoke_reference text,
  changed_at timestamptz
)
language sql
stable
security definer
set search_path = ''
as $$
  select
    assignment.assignment_id,
    assignment.user_id,
    assignment.role_name,
    case
      when assignment.status = 'active'
        and assignment.expires_at is not null
        and assignment.expires_at <= timezone('utc', now())
      then 'expired'
      else assignment.status
    end as status,
    assignment.granted_at,
    assignment.expires_at,
    assignment.revoked_at,
    assignment.grant_reference,
    assignment.revoke_reference,
    assignment.created_at as changed_at
  from public.operations_role_assignments assignment
  where public.is_service_role_session_v1()
    and (p_user_id is null or assignment.user_id = p_user_id)
  order by assignment.granted_at desc, assignment.created_at desc;
$$;

comment on function public.inspect_operations_role_v1(uuid) is
  'Service-role-only read path for audited operations access assignments.';

revoke all on function public.inspect_operations_role_v1(uuid)
from public, anon, authenticated, service_role;
grant execute on function public.inspect_operations_role_v1(uuid)
to service_role;

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
  v_actor_role text := public.current_operations_role_v1();
  v_title text := trim(p_title);
  v_summary text := trim(p_summary);
  v_requirements text := trim(p_requirements);
  v_audit_reference text := trim(p_audit_reference);
  v_task_id uuid;
begin
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
  v_actor_role text := public.current_operations_role_v1();
  v_notes text := nullif(trim(p_reviewer_notes), '');
  v_public_summary text := nullif(trim(p_public_result_summary), '');
  v_public_url text := nullif(trim(p_public_deliverable_url), '');
  v_audit_reference text := trim(p_audit_reference);
  v_submission public.task_submissions%rowtype;
  v_task public.community_tasks%rowtype;
  v_publication_id uuid;
  v_reviewed_at timestamptz := timezone('utc', now());
begin
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

  if not found
    or v_task.publication_status <> 'published'
    or v_task.status not in ('open', 'under_review')
  then
    raise exception 'task submission is not attached to an active published task';
  end if;

  if p_decision = 'accepted' then
    if not v_submission.public_result_consent then
      raise exception 'accepted task review requires contributor public result consent';
    end if;
    if v_public_summary is null
      or char_length(v_public_summary) not between 20 and 3000
    then
      raise exception 'accepted task review requires a sanitized public result summary';
    end if;
    if v_public_url is null
      or char_length(v_public_url) > 2000
      or v_public_url !~ '^https://[^[:space:]@/]+(/[^[:space:]]*)?$'
      or v_public_url ~ '[[:cntrl:]]'
    then
      raise exception 'accepted task review requires a safe public deliverable URL';
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

  if p_decision = 'accepted' then
    insert into public.task_submission_publications (
      task_id,
      task_title,
      result_summary,
      deliverable_url,
      wallet_address,
      review_reference,
      accepted_at,
      published_at
    ) values (
      v_submission.task_id,
      v_task.title,
      v_public_summary,
      v_public_url,
      case when v_submission.public_wallet_consent then v_submission.wallet_address else null end,
      v_audit_reference,
      v_reviewed_at,
      v_reviewed_at
    )
    returning id into v_publication_id;
  end if;

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
      'decision', p_decision,
      'publication_id', v_publication_id,
      'public_result_consent', v_submission.public_result_consent,
      'public_wallet_consent', v_submission.public_wallet_consent,
      'accepted_never_means_paid', true
    )
  );

  if p_decision = 'accepted' then
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
        'publication_id', v_publication_id,
        'wallet_published', v_submission.public_wallet_consent
      )
    );
  end if;

  return query
  select
    v_submission.id,
    p_decision,
    v_publication_id;
end;
$$;

create or replace function public.review_risk_report_v1(
  p_risk_report_id uuid,
  p_decision text,
  p_reviewer_notes text,
  p_public_summary text,
  p_public_reference_url text,
  p_publication_basis text,
  p_audit_reference text
)
returns table (
  risk_report_id uuid,
  review_status text,
  publication_id uuid
)
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_actor_id uuid := auth.uid();
  v_actor_role text := public.current_operations_role_v1();
  v_notes text := nullif(trim(p_reviewer_notes), '');
  v_public_summary text := nullif(trim(p_public_summary), '');
  v_public_reference_url text := nullif(trim(p_public_reference_url), '');
  v_publication_basis text := nullif(trim(p_publication_basis), '');
  v_audit_reference text := trim(p_audit_reference);
  v_report public.risk_reports%rowtype;
  v_publication_id uuid;
  v_reviewed_at timestamptz := timezone('utc', now());
begin
  if v_actor_id is null
    or v_actor_role is null
    or v_actor_role not in ('reviewer', 'operator', 'governance_admin')
  then
    raise exception 'operations role is not authorized to review risk reports';
  end if;

  if p_decision is null or p_decision not in ('published', 'dismissed') then
    raise exception 'risk report decision must be published or dismissed';
  end if;

  if v_notes is null or char_length(v_notes) > 5000 then
    raise exception 'risk reviewer notes are required and cannot exceed 5000 characters';
  end if;

  if v_audit_reference is null
    or char_length(v_audit_reference) not between 10 and 160
    or v_audit_reference ~ '[[:cntrl:]]'
  then
    raise exception 'risk review audit reference must be 10 to 160 characters without control characters';
  end if;

  select report.*
  into v_report
  from public.risk_reports report
  where report.id = p_risk_report_id
  for update;

  if not found then
    raise exception 'risk report was not found';
  end if;

  if v_report.submitted_by = v_actor_id then
    raise exception 'reviewers cannot review their own risk report';
  end if;

  if v_report.review_status in ('resolved', 'dismissed')
    or v_report.publication_status <> 'private'
  then
    raise exception 'risk report is already in a terminal review state';
  end if;

  if p_decision = 'published' then
    if not v_report.public_report_consent then
      raise exception 'sanitized risk publication requires reporter consent';
    end if;

    if v_public_summary is null
      or char_length(v_public_summary) not between 30 and 5000
    then
      raise exception 'published risk record requires a sanitized summary of 30 to 5000 characters';
    end if;

    if v_publication_basis is null
      or char_length(v_publication_basis) not between 10 and 1000
    then
      raise exception 'published risk record requires a publication basis of 10 to 1000 characters';
    end if;

    if v_public_reference_url is not null and not v_report.public_reference_consent then
      raise exception 'public reference URL requires separate reporter consent';
    end if;

    if v_public_reference_url is not null
      and (
        char_length(v_public_reference_url) > 2000
        or v_public_reference_url !~ '^https://[^[:space:]@/]+(/[^[:space:]]*)?$'
        or v_public_reference_url ~ '[[:cntrl:]]'
      )
    then
      raise exception 'published risk reference must be a safe HTTPS URL';
    end if;
  elsif v_public_summary is not null
    or v_public_reference_url is not null
    or v_publication_basis is not null
  then
    raise exception 'dismissed risk reports cannot create a public record';
  end if;

  if p_decision = 'published' then
    if v_report.review_status = 'submitted' then
      update public.risk_reports report
      set review_status = 'triaged'
      where report.id = v_report.id;
    end if;

    if v_report.review_status in ('submitted', 'triaged') then
      update public.risk_reports report
      set review_status = 'investigating'
      where report.id = v_report.id;
    end if;

    update public.risk_reports report
    set
      review_status = 'resolved',
      publication_status = 'published',
      reviewer_notes = v_notes,
      reviewed_by = v_actor_id,
      reviewed_at = v_reviewed_at,
      published_at = v_reviewed_at
    where report.id = v_report.id;

    insert into public.risk_publications (
      report_reference,
      project_identifier,
      summary,
      reference_url,
      public_status,
      publication_basis,
      published_at
    ) values (
      v_audit_reference,
      v_report.project_identifier,
      v_public_summary,
      v_public_reference_url,
      'published',
      v_publication_basis,
      v_reviewed_at
    )
    returning id into v_publication_id;
  else
    update public.risk_reports report
    set
      review_status = 'dismissed',
      reviewer_notes = v_notes,
      reviewed_by = v_actor_id,
      reviewed_at = v_reviewed_at
    where report.id = v_report.id;
  end if;

  insert into public.operations_risk_workflow_events (
    risk_report_id,
    action,
    actor_id,
    actor_role,
    event_reference,
    event_data
  ) values (
    v_report.id,
    case when p_decision = 'published' then 'report_published' else 'report_dismissed' end,
    v_actor_id,
    v_actor_role,
    v_audit_reference,
    jsonb_build_object(
      'decision', p_decision,
      'publication_id', v_publication_id,
      'reporter_publication_consent', v_report.public_report_consent,
      'reporter_reference_consent', v_report.public_reference_consent,
      'evidence_count', (
        select count(*) from public.risk_evidence evidence
        where evidence.risk_report_id = v_report.id
      )
    )
  );

  return query
  select
    v_report.id,
    case when p_decision = 'published' then 'resolved' else 'dismissed' end,
    v_publication_id;
end;
$$;

create or replace function public.review_relief_application_v1(
  p_relief_application_id uuid,
  p_decision text,
  p_reviewer_notes text,
  p_public_title text,
  p_public_summary text,
  p_publication_basis text,
  p_audit_reference text
)
returns table (
  relief_application_id uuid,
  review_status text,
  public_update_id uuid
)
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_actor_id uuid := auth.uid();
  v_actor_role text := public.current_operations_role_v1();
  v_notes text := nullif(trim(p_reviewer_notes), '');
  v_title text := nullif(trim(p_public_title), '');
  v_summary text := nullif(trim(p_public_summary), '');
  v_basis text := nullif(trim(p_publication_basis), '');
  v_reference text := trim(p_audit_reference);
  v_application public.relief_applications%rowtype;
  v_update_id uuid;
  v_reviewed_at timestamptz := timezone('utc', now());
begin
  if v_actor_id is null
    or v_actor_role is null
    or v_actor_role not in ('relief_reviewer', 'operator', 'governance_admin')
  then
    raise exception 'operations role is not authorized to review relief applications';
  end if;

  if p_decision is null or p_decision not in ('approved', 'rejected') then
    raise exception 'relief decision must be approved or rejected';
  end if;

  if v_notes is null or char_length(v_notes) > 5000 then
    raise exception 'relief reviewer notes are required and cannot exceed 5000 characters';
  end if;

  if v_reference is null
    or char_length(v_reference) not between 10 and 160
    or v_reference ~ '[[:cntrl:]]'
  then
    raise exception 'relief review audit reference must be 10 to 160 characters without control characters';
  end if;

  select application.*
  into v_application
  from public.relief_applications application
  where application.id = p_relief_application_id
  for update;

  if not found then
    raise exception 'relief application was not found';
  end if;

  if v_application.submitted_by = v_actor_id then
    raise exception 'reviewers cannot review their own relief application';
  end if;

  if v_application.status in ('approved', 'rejected', 'cancelled', 'paid') then
    raise exception 'relief application is already in a terminal review state';
  end if;

  if v_application.payment_receipt_id is not null
    or exists (
      select 1 from public.treasury_execution_intents intent
      where intent.relief_application_id = v_application.id
    )
  then
    raise exception 'relief review cannot run after a treasury execution record exists';
  end if;

  if p_decision = 'approved' then
    if (v_title is null) <> (v_summary is null)
      or (v_title is null) <> (v_basis is null)
    then
      raise exception 'public relief update fields must be supplied together';
    end if;

    if v_title is not null then
      if not v_application.public_update_consent then
        raise exception 'sanitized relief update requires claimant consent';
      end if;
      if char_length(v_title) not between 4 and 160
        or char_length(v_summary) not between 20 and 3000
        or char_length(v_basis) not between 10 and 1000
      then
        raise exception 'public relief update fields are outside allowed lengths';
      end if;
    end if;
  elsif v_title is not null or v_summary is not null or v_basis is not null then
    raise exception 'rejected relief applications cannot create a public update';
  end if;

  if v_application.status = 'submitted' then
    update public.relief_applications set status = 'triaged'
    where id = v_application.id;
  end if;
  if v_application.status in ('submitted', 'triaged', 'evidence_requested') then
    update public.relief_applications set status = 'under_review'
    where id = v_application.id;
  end if;

  update public.relief_applications
  set status = p_decision,
      reviewer_notes = v_notes,
      reviewed_by = v_actor_id,
      reviewed_at = v_reviewed_at
  where id = v_application.id;

  if p_decision = 'approved' and v_title is not null then
    insert into public.relief_public_updates (
      case_reference, title, summary, outcome, publication_basis, published_at
    ) values (
      v_reference, v_title, v_summary, 'approved', v_basis, v_reviewed_at
    ) returning id into v_update_id;
  end if;

  insert into public.operations_relief_workflow_events (
    relief_application_id, action, actor_id, actor_role,
    event_reference, event_data
  ) values (
    v_application.id,
    case when p_decision = 'approved' then 'application_approved' else 'application_rejected' end,
    v_actor_id,
    v_actor_role,
    v_reference,
    jsonb_build_object(
      'decision', p_decision,
      'public_update_created', v_update_id is not null,
      'payment_intent_created', false,
      'payment_receipt_created', false,
      'approval_is_payment', false
    )
  );

  return query select v_application.id, p_decision, v_update_id;
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
  v_actor_role text := public.current_operations_role_v1();
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
  v_actor_role text := public.current_operations_role_v1();
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
    publication_status, decided_by, decided_at
  ) values (
    v_proposal.id, p_decision, btrim(p_rationale), v_decision_hash, v_proposal.execution_required,
    case when v_proposal.execution_required then v_hash else null end,
    case when v_proposal.execution_required then v_hash else null end, btrim(p_finalization_reference),
    'published', v_actor_id, timezone('utc', now())
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
    p_asset_symbol, p_asset_decimals, p_asset_mint, public.mask_treasury_destination_wallet_v1(p_destination_wallet), p_amount_base_units, p_network,
    'prepared', timezone('utc', now())
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
  v_actor_role text := public.current_operations_role_v1();
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
  v_actor_role text := public.current_operations_role_v1();
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
  v_actor_role text := public.current_operations_role_v1();
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
  v_actor_role text := public.current_operations_role_v1();
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

commit;
