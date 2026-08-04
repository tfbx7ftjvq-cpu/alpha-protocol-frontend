-- Alpha Protocol Phase 2E-6B-4N
-- Strict service-role-only cleanup for controlled risk-workflow Staging E2E.
-- Immutable production records remain protected outside the exact fixture
-- graph validated by this SECURITY DEFINER RPC.

begin;

create or replace function public.protect_risk_workflow_immutable_v1()
returns trigger
language plpgsql
set search_path = ''
as $$
declare
  cleanup_reference text := current_setting(
    'alpha.operations_risk_staging_e2e_cleanup_reference',
    true
  );
  cleanup_owner name;
begin
  select pg_catalog.pg_get_userbyid(procedure.proowner)
  into cleanup_owner
  from pg_catalog.pg_proc procedure
  where procedure.oid = pg_catalog.to_regprocedure(
    'public.cleanup_operations_risk_staging_e2e_v1(text,uuid[])'
  );

  if tg_op = 'DELETE'
    and cleanup_owner is not null
    and current_user = cleanup_owner
    and cleanup_reference ~ '^phase-2e-6b-4n-staging-e2e:[0-9]{13}-[0-9a-f]{8}$'
  then
    return old;
  end if;

  raise exception 'immutable operations record cannot be updated or deleted';
end;
$$;

revoke all on function public.protect_risk_workflow_immutable_v1() from public;

drop trigger risk_publications_immutable on public.risk_publications;
create trigger risk_publications_immutable
before update or delete on public.risk_publications
for each row execute function public.protect_risk_workflow_immutable_v1();

drop trigger operations_risk_workflow_events_immutable
on public.operations_risk_workflow_events;
create trigger operations_risk_workflow_events_immutable
before update or delete on public.operations_risk_workflow_events
for each row execute function public.protect_risk_workflow_immutable_v1();

create or replace function public.cleanup_operations_risk_staging_e2e_v1(
  p_run_reference text,
  p_risk_report_ids uuid[]
)
returns table (
  publications_deleted integer,
  events_deleted integer,
  evidence_deleted integer,
  reports_deleted integer
)
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_run_reference text := p_run_reference;
  v_run_id text;
  v_report_count integer := coalesce(cardinality(p_risk_report_ids), 0);
  v_fixture_reports integer := 0;
  v_fixture_evidence integer := 0;
  v_fixture_publications integer := 0;
  v_fixture_events integer := 0;
  v_publications_deleted integer := 0;
  v_events_deleted integer := 0;
  v_evidence_deleted integer := 0;
  v_reports_deleted integer := 0;
begin
  if v_run_reference is null
    or v_run_reference <> trim(v_run_reference)
    or v_run_reference !~ '^phase-2e-6b-4n-staging-e2e:[0-9]{13}-[0-9a-f]{8}$'
  then
    raise exception 'invalid Phase 4N Staging E2E cleanup reference';
  end if;

  if p_risk_report_ids is null
    or v_report_count <> 2
    or (
      select count(distinct fixture.report_id)::integer
      from unnest(p_risk_report_ids) fixture(report_id)
    ) <> 2
  then
    raise exception 'Phase 4N Staging E2E cleanup requires two distinct report ids';
  end if;

  v_run_id := substring(
    v_run_reference from char_length('phase-2e-6b-4n-staging-e2e:') + 1
  );

  select count(*)::integer
  into v_fixture_reports
  from public.risk_reports report
  where report.id = any(p_risk_report_ids)
    and report.project_identifier in (
      'Staging risk publish ' || v_run_id,
      'Staging risk dismiss ' || v_run_id
    )
    and report.reference_url in (
      'https://example.com/alpha-staging-risk-' || v_run_id || '-publish',
      'https://example.com/alpha-staging-risk-' || v_run_id || '-dismiss'
    )
    and report.wallet_verified = false
    and (
      (report.review_status = 'resolved' and report.publication_status = 'published')
      or (report.review_status = 'dismissed' and report.publication_status = 'private')
    );

  if v_fixture_reports <> 2 then
    raise exception 'cleanup reports are not exact Phase 4N Staging E2E fixtures';
  end if;

  select count(*)::integer
  into v_fixture_evidence
  from public.risk_evidence evidence
  where evidence.risk_report_id = any(p_risk_report_ids)
    and evidence.evidence_url =
      'https://example.com/alpha-staging-risk-' || v_run_id || '-additional-evidence'
    and evidence.summary =
      'Additional private Phase 4N Staging evidence ' || v_run_id || '.'
    and evidence.is_public = false;

  if v_fixture_evidence <> 1
    or v_fixture_evidence <> (
      select count(*)::integer from public.risk_evidence evidence
      where evidence.risk_report_id = any(p_risk_report_ids)
    )
  then
    raise exception 'cleanup evidence is not the exact Phase 4N Staging E2E fixture';
  end if;

  select count(*)::integer
  into v_fixture_publications
  from public.risk_publications publication
  where publication.report_reference = v_run_reference || ':published'
    and publication.project_identifier = 'Staging risk publish ' || v_run_id
    and publication.summary =
      'Sanitized Phase 4N Staging risk finding ' || v_run_id
      || ' with private reporter and evidence metadata removed.'
    and publication.reference_url =
      'https://example.com/alpha-staging-risk-' || v_run_id || '-publish'
    and publication.public_status = 'published';

  if v_fixture_publications <> 1 then
    raise exception 'cleanup publication is not the exact Phase 4N Staging E2E fixture';
  end if;

  select count(*)::integer
  into v_fixture_events
  from public.operations_risk_workflow_events event
  where event.risk_report_id = any(p_risk_report_ids)
    and (
      (event.action = 'report_published'
        and event.event_reference = v_run_reference || ':published')
      or (event.action = 'report_dismissed'
        and event.event_reference = v_run_reference || ':dismissed')
    );

  if v_fixture_events <> 2
    or v_fixture_events <> (
      select count(*)::integer from public.operations_risk_workflow_events event
      where event.risk_report_id = any(p_risk_report_ids)
    )
  then
    raise exception 'cleanup events are not exact Phase 4N Staging E2E fixtures';
  end if;

  if exists (
    select 1 from public.risk_publications publication
    where publication.report_reference like v_run_reference || ':%'
      and publication.report_reference <> v_run_reference || ':published'
  ) or exists (
    select 1 from public.operations_risk_workflow_events event
    where event.event_reference like v_run_reference || ':%'
      and not (event.risk_report_id = any(p_risk_report_ids))
  ) then
    raise exception 'Phase 4N Staging E2E cleanup reference is not isolated';
  end if;

  perform set_config(
    'alpha.operations_risk_staging_e2e_cleanup_reference',
    v_run_reference,
    true
  );

  delete from public.risk_publications publication
  where publication.report_reference = v_run_reference || ':published';
  get diagnostics v_publications_deleted = row_count;

  delete from public.operations_risk_workflow_events event
  where event.risk_report_id = any(p_risk_report_ids);
  get diagnostics v_events_deleted = row_count;

  delete from public.risk_evidence evidence
  where evidence.risk_report_id = any(p_risk_report_ids);
  get diagnostics v_evidence_deleted = row_count;

  delete from public.risk_reports report
  where report.id = any(p_risk_report_ids);
  get diagnostics v_reports_deleted = row_count;

  if v_publications_deleted <> 1
    or v_events_deleted <> 2
    or v_evidence_deleted <> 1
    or v_reports_deleted <> 2
  then
    raise exception 'Phase 4N Staging E2E cleanup count mismatch';
  end if;

  return query select
    v_publications_deleted,
    v_events_deleted,
    v_evidence_deleted,
    v_reports_deleted;
end;
$$;

comment on function public.cleanup_operations_risk_staging_e2e_v1(text, uuid[]) is
  'Deletes only one exact Phase 4N Staging risk fixture graph after strict validation.';

revoke all on function public.cleanup_operations_risk_staging_e2e_v1(text, uuid[])
from public, anon, authenticated, service_role;
grant execute on function public.cleanup_operations_risk_staging_e2e_v1(text, uuid[])
to service_role;

commit;
