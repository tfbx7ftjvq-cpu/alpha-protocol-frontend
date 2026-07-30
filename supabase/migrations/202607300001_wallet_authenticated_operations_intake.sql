-- Alpha Protocol Phase 2E-6B-4I
-- Wallet-authenticated operations intake.
--
-- A browser form is not an authorization boundary. These rules bind every
-- private intake row to a Solana address verified by Supabase Web3 Auth and
-- add database-side rate limits. No function in this migration sends a
-- Solana transaction or moves treasury funds.

begin;

create table public.operations_intake_control (
  singleton boolean primary key default true check (singleton),
  mode text not null default 'disabled' check (mode in ('disabled', 'wallet_staging')),
  activation_reference text,
  updated_at timestamptz not null default timezone('utc', now()),
  check (
    (mode = 'disabled' and activation_reference is null)
    or (
      mode = 'wallet_staging'
      and activation_reference is not null
      and char_length(trim(activation_reference)) between 10 and 200
    )
  )
);

comment on table public.operations_intake_control is
  'Postgres-controlled server gate. Applying this migration leaves wallet intake disabled.';

alter table public.operations_intake_control enable row level security;
revoke all on table public.operations_intake_control from public, anon, authenticated;

insert into public.operations_intake_control (singleton, mode)
values (true, 'disabled');

create or replace function public.touch_operations_intake_control()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  if new.singleton is distinct from old.singleton then
    raise exception 'operations intake control singleton cannot change';
  end if;

  new.updated_at := timezone('utc', now());
  return new;
end;
$$;

revoke all on function public.touch_operations_intake_control() from public;

create trigger operations_intake_control_touch
before update on public.operations_intake_control
for each row execute function public.touch_operations_intake_control();

create or replace function public.is_operations_wallet_intake_enabled()
returns boolean
language sql
stable
security definer
set search_path = ''
as $$
  select coalesce(
    (
      select control.mode = 'wallet_staging'
      from public.operations_intake_control control
      where control.singleton
    ),
    false
  );
$$;

comment on function public.is_operations_wallet_intake_enabled() is
  'Returns true only after the Postgres-controlled wallet Staging intake gate is explicitly activated.';

revoke all on function public.is_operations_wallet_intake_enabled() from public;
revoke all on function public.is_operations_wallet_intake_enabled() from anon;
grant execute on function public.is_operations_wallet_intake_enabled() to authenticated;

create or replace function public.current_verified_solana_wallet()
returns text
language sql
stable
security definer
set search_path = ''
as $$
  with verified_wallets as (
    select distinct identity.identity_data ->> 'address' as address
    from auth.identities identity
    where identity.user_id = (select auth.uid())
      and identity.provider = 'web3'
      and identity.identity_data ->> 'chain' = 'solana'
      and nullif(identity.identity_data ->> 'address', '') is not null
  )
  select case
    when count(*) = 1 then min(address)
    else null
  end
  from verified_wallets;
$$;

comment on function public.current_verified_solana_wallet() is
  'Returns the one Solana address verified by Supabase Web3 Auth for auth.uid(); otherwise NULL.';

revoke all on function public.current_verified_solana_wallet() from public;
revoke all on function public.current_verified_solana_wallet() from anon;
grant execute on function public.current_verified_solana_wallet() to authenticated;

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
  if not public.is_operations_wallet_intake_enabled() then
    raise exception 'operations wallet intake is disabled';
  end if;

  if new.submitted_by is distinct from auth.uid() then
    raise exception 'operations submission owner does not match auth.uid()';
  end if;

  new.created_at := now();
  new.updated_at := now();

  case tg_table_name
    when 'task_submissions' then
      maximum_rows := 8;
      lookback := interval '1 hour';
    when 'risk_reports' then
      maximum_rows := 6;
      lookback := interval '1 hour';
    when 'relief_applications' then
      maximum_rows := 3;
      lookback := interval '24 hours';
    when 'governance_discussions' then
      maximum_rows := 20;
      lookback := interval '1 hour';
    else
      raise exception 'unsupported operations rate-limit table: %', tg_table_name;
  end case;

  perform pg_advisory_xact_lock(
    hashtextextended(new.submitted_by::text || ':' || tg_table_name, 0)
  );

  execute format(
    'select count(*) from public.%I where submitted_by = $1 and created_at > now() - $2',
    tg_table_name
  )
  into recent_rows
  using new.submitted_by, lookback;

  if recent_rows >= maximum_rows then
    raise exception 'operations submission rate limit exceeded for %', tg_table_name;
  end if;

  return new;
end;
$$;

comment on function public.enforce_operations_submission_rate_limit() is
  'Per-auth-user intake throttle. This is separate from Supabase Web3 Auth and CAPTCHA limits.';

revoke all on function public.enforce_operations_submission_rate_limit() from public;

drop policy task_submissions_owner_insert on public.task_submissions;
create policy task_submissions_owner_insert
on public.task_submissions
for insert
to authenticated
with check (
  public.is_operations_wallet_intake_enabled()
  and submitted_by = auth.uid()
  and wallet_address = public.current_verified_solana_wallet()
  and status = 'submitted'
  and wallet_verified = false
  and reviewer_notes is null
  and reviewed_by is null
  and reviewed_at is null
  and exists (
    select 1
    from public.community_tasks task
    where task.id = task_id
      and task.publication_status = 'published'
      and task.status = 'open'
      and (task.submission_deadline is null or task.submission_deadline > timezone('utc', now()))
  )
);

drop policy risk_reports_owner_insert on public.risk_reports;
create policy risk_reports_owner_insert
on public.risk_reports
for insert
to authenticated
with check (
  public.is_operations_wallet_intake_enabled()
  and submitted_by = auth.uid()
  and wallet_address = public.current_verified_solana_wallet()
  and review_status = 'submitted'
  and publication_status = 'private'
  and published_at is null
  and wallet_verified = false
  and reviewer_notes is null
  and reviewed_by is null
  and reviewed_at is null
);

drop policy relief_applications_owner_insert on public.relief_applications;
create policy relief_applications_owner_insert
on public.relief_applications
for insert
to authenticated
with check (
  public.is_operations_wallet_intake_enabled()
  and submitted_by = auth.uid()
  and wallet_address = public.current_verified_solana_wallet()
  and status = 'submitted'
  and wallet_verified = false
  and reviewer_notes is null
  and reviewed_by is null
  and reviewed_at is null
);

drop policy governance_discussions_owner_insert on public.governance_discussions;
create policy governance_discussions_owner_insert
on public.governance_discussions
for insert
to authenticated
with check (
  public.is_operations_wallet_intake_enabled()
  and submitted_by = auth.uid()
  and wallet_address = public.current_verified_solana_wallet()
  and moderation_status = 'pending'
  and wallet_verified = false
  and moderated_by is null
);

create trigger task_submissions_rate_limit
before insert on public.task_submissions
for each row execute function public.enforce_operations_submission_rate_limit();

create trigger risk_reports_rate_limit
before insert on public.risk_reports
for each row execute function public.enforce_operations_submission_rate_limit();

create trigger relief_applications_rate_limit
before insert on public.relief_applications
for each row execute function public.enforce_operations_submission_rate_limit();

create trigger governance_discussions_rate_limit
before insert on public.governance_discussions
for each row execute function public.enforce_operations_submission_rate_limit();

commit;
