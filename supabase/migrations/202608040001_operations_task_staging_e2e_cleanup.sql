-- Alpha Protocol Phase 2E-6B-4M
-- Strict, service-role-only cleanup for controlled task-workflow Staging E2E.
--
-- The production workflow keeps task publications and audit events immutable.
-- This migration does not weaken UPDATE immutability and does not grant any
-- browser or service-role table mutation privilege. It exposes one narrowly
-- validated SECURITY DEFINER cleanup RPC for exact Phase 4M Staging fixtures.
-- No network request, Solana transaction, treasury mutation, or funds movement
-- is performed.

begin;

create or replace function public.protect_task_workflow_immutable_v1()
returns trigger
language plpgsql
set search_path = ''
as $$
declare
  cleanup_reference text := current_setting(
    'alpha.operations_task_staging_e2e_cleanup_reference',
    true
  );
  cleanup_owner name;
begin
  select pg_catalog.pg_get_userbyid(procedure.proowner)
  into cleanup_owner
  from pg_catalog.pg_proc procedure
  where procedure.oid = pg_catalog.to_regprocedure(
    'public.cleanup_operations_task_staging_e2e_v1(text,uuid,uuid[])'
  );

  -- The exception is deliberately limited to DELETE statements issued from
  -- the SECURITY DEFINER cleanup function below. A caller that merely knows
  -- the custom setting cannot assume the cleanup function owner's identity.
  if tg_op = 'DELETE'
    and cleanup_owner is not null
    and current_user = cleanup_owner
    and cleanup_reference ~ '^phase-2e-6b-4m-staging-e2e:[0-9]{13}-[0-9a-f]{8}$'
  then
    return old;
  end if;

  raise exception 'immutable operations record cannot be updated or deleted';
end;
$$;

revoke all on function public.protect_task_workflow_immutable_v1() from public;

drop trigger task_submission_publications_immutable
on public.task_submission_publications;

create trigger task_submission_publications_immutable
before update or delete on public.task_submission_publications
for each row execute function public.protect_task_workflow_immutable_v1();

drop trigger operations_task_workflow_events_immutable
on public.operations_task_workflow_events;

create trigger operations_task_workflow_events_immutable
before update or delete on public.operations_task_workflow_events
for each row execute function public.protect_task_workflow_immutable_v1();

-- Phase 4H granted these two direct DELETE privileges for its smaller test
-- graph. Phase 4M has immutable child rows and must clean the whole graph
-- atomically, so the historical direct path is no longer needed. Keep the
-- separate governance-discussion cleanup privilege unchanged.
revoke delete on table
  public.task_submissions,
  public.community_tasks
from service_role;

create or replace function public.cleanup_operations_task_staging_e2e_v1(
  p_run_reference text,
  p_task_id uuid,
  p_submission_ids uuid[]
)
returns table (
  publications_deleted integer,
  events_deleted integer,
  submissions_deleted integer,
  tasks_deleted integer
)
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_run_reference text := p_run_reference;
  v_run_id text;
  v_submission_count integer := coalesce(cardinality(p_submission_ids), 0);
  v_distinct_submission_count integer := 0;
  v_task_fixture_count integer := 0;
  v_submission_fixture_count integer := 0;
  v_publication_fixture_count integer := 0;
  v_event_fixture_count integer := 0;
  v_prefix_collision_count integer := 0;
  v_publications_deleted integer := 0;
  v_events_deleted integer := 0;
  v_submissions_deleted integer := 0;
  v_tasks_deleted integer := 0;
begin
  if v_run_reference is null
    or v_run_reference <> trim(v_run_reference)
    or v_run_reference !~ '^phase-2e-6b-4m-staging-e2e:[0-9]{13}-[0-9a-f]{8}$'
  then
    raise exception 'invalid Phase 4M Staging E2E cleanup reference';
  end if;

  if p_task_id is null
    or p_submission_ids is null
    or v_submission_count not between 0 and 2
  then
    raise exception 'invalid Phase 4M Staging E2E cleanup identifiers';
  end if;

  select count(distinct fixture.submission_id)::integer
  into v_distinct_submission_count
  from unnest(p_submission_ids) as fixture(submission_id);

  if v_distinct_submission_count <> v_submission_count then
    raise exception 'Phase 4M Staging E2E cleanup identifiers must be distinct';
  end if;

  v_run_id := substring(
    v_run_reference
    from char_length('phase-2e-6b-4m-staging-e2e:') + 1
  );

  select count(*)::integer
  into v_task_fixture_count
  from public.community_tasks task
  where task.id = p_task_id
    and task.title = 'Staging task workflow ' || v_run_id
    and task.summary =
      'Temporary Phase 4M Staging task for audited publication and review E2E '
      || v_run_id || '.'
    and task.requirements =
      'Submit only the two reserved example.com fixtures for this controlled Staging E2E run '
      || v_run_id || '.'
    and task.reward_budget_usdc = 0
    and task.reward_source = 'none'
    and task.publication_status = 'published';

  if v_task_fixture_count <> 1 then
    raise exception 'cleanup target is not the exact Phase 4M Staging E2E task';
  end if;

  select count(*)::integer
  into v_submission_fixture_count
  from public.task_submissions submission
  where submission.id = any(p_submission_ids)
    and submission.task_id = p_task_id
    and submission.wallet_verified = false
    and submission.public_wallet_consent = false
    and (
      (
        submission.summary =
          'Accepted Phase 4M Staging submission ' || v_run_id
          || ' for sanitized publication and immutable audit verification.'
        and submission.deliverable_url =
          'https://example.com/alpha-staging-task-' || v_run_id || '-accepted'
        and submission.public_result_consent = true
        and submission.status in ('submitted', 'in_review', 'accepted')
      )
      or
      (
        submission.summary =
          'Rejected Phase 4M Staging submission ' || v_run_id
          || ' for terminal-state and no-publication verification.'
        and submission.deliverable_url =
          'https://example.com/alpha-staging-task-' || v_run_id || '-rejected'
        and submission.public_result_consent = false
        and submission.status in ('submitted', 'in_review', 'rejected')
      )
    );

  if v_submission_fixture_count <> v_submission_count
    or (
      select count(*)::integer
      from public.task_submissions submission
      where submission.task_id = p_task_id
    ) <> v_submission_count
  then
    raise exception 'cleanup submissions are not the exact Phase 4M Staging E2E fixtures';
  end if;

  select count(*)::integer
  into v_publication_fixture_count
  from public.task_submission_publications publication
  where publication.task_id = p_task_id
    and publication.review_reference = v_run_reference || ':accepted'
    and publication.task_title = 'Staging task workflow ' || v_run_id
    and publication.result_summary =
      'Sanitized accepted result for Phase 4M Staging workflow ' || v_run_id
      || '; no payment or treasury action occurred.'
    and publication.deliverable_url =
      'https://example.com/alpha-staging-task-' || v_run_id || '-accepted'
    and publication.wallet_address is null;

  if v_publication_fixture_count <> (
      select count(*)::integer
      from public.task_submission_publications publication
      where publication.task_id = p_task_id
    )
  then
    raise exception 'cleanup publications are not exact Phase 4M Staging E2E fixtures';
  end if;

  select count(*)::integer
  into v_event_fixture_count
  from public.operations_task_workflow_events event
  where (
      event.entity_type = 'community_task'
      and event.entity_reference = p_task_id
      and event.action = 'task_published'
      and event.event_reference = v_run_reference || ':task:publish'
    )
    or (
      event.entity_type = 'task_submission'
      and event.entity_reference = any(p_submission_ids)
      and (
        event.event_reference = v_run_reference || ':accepted:decision'
          and event.action = 'submission_accepted'
        or event.event_reference = v_run_reference || ':accepted:publication'
          and event.action = 'result_published'
        or event.event_reference = v_run_reference || ':rejected:decision'
          and event.action = 'submission_rejected'
      )
    );

  if v_event_fixture_count <> (
      select count(*)::integer
      from public.operations_task_workflow_events event
      where event.entity_reference = p_task_id
        or event.entity_reference = any(p_submission_ids)
    )
  then
    raise exception 'cleanup events are not exact Phase 4M Staging E2E fixtures';
  end if;

  select count(*)::integer
  into v_prefix_collision_count
  from public.operations_task_workflow_events event
  where event.event_reference like v_run_reference || ':%'
    and event.entity_reference <> p_task_id
    and not (event.entity_reference = any(p_submission_ids));

  if v_prefix_collision_count <> 0 then
    raise exception 'Phase 4M Staging E2E cleanup reference is not isolated';
  end if;

  perform set_config(
    'alpha.operations_task_staging_e2e_cleanup_reference',
    v_run_reference,
    true
  );

  delete from public.task_submission_publications publication
  where publication.task_id = p_task_id;
  get diagnostics v_publications_deleted = row_count;

  delete from public.operations_task_workflow_events event
  where event.entity_reference = p_task_id
    or event.entity_reference = any(p_submission_ids);
  get diagnostics v_events_deleted = row_count;

  perform set_config(
    'alpha.operations_task_staging_e2e_cleanup_reference',
    '',
    true
  );

  delete from public.task_submissions submission
  where submission.id = any(p_submission_ids)
    and submission.task_id = p_task_id;
  get diagnostics v_submissions_deleted = row_count;

  delete from public.community_tasks task
  where task.id = p_task_id;
  get diagnostics v_tasks_deleted = row_count;

  if v_publications_deleted <> v_publication_fixture_count
    or v_events_deleted <> v_event_fixture_count
    or v_submissions_deleted <> v_submission_count
    or v_tasks_deleted <> 1
  then
    raise exception 'Phase 4M Staging E2E cleanup count mismatch';
  end if;

  return query
  select
    v_publications_deleted,
    v_events_deleted,
    v_submissions_deleted,
    v_tasks_deleted;
end;
$$;

comment on function public.cleanup_operations_task_staging_e2e_v1(text, uuid, uuid[]) is
  'Service-role-only atomic cleanup for exact reserved Phase 4M Staging E2E fixtures. Never a production workflow mutation path.';

revoke all on function public.cleanup_operations_task_staging_e2e_v1(text, uuid, uuid[])
from public, anon, authenticated, service_role;
grant execute on function public.cleanup_operations_task_staging_e2e_v1(text, uuid, uuid[])
to service_role;

commit;
