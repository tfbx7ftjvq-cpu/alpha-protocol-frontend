-- Alpha Protocol Phase 2E-6B-4O
-- Strict service-role-only cleanup for the exact relief Staging E2E graph.

begin;

create or replace function public.protect_relief_workflow_immutable_v1()
returns trigger language plpgsql set search_path = '' as $$
declare
  cleanup_reference text := current_setting(
    'alpha.operations_relief_staging_e2e_cleanup_reference', true
  );
  cleanup_owner name;
begin
  select pg_catalog.pg_get_userbyid(procedure.proowner)
  into cleanup_owner
  from pg_catalog.pg_proc procedure
  where procedure.oid = pg_catalog.to_regprocedure(
    'public.cleanup_operations_relief_staging_e2e_v1(text,uuid[])'
  );

  if tg_op = 'DELETE'
    and cleanup_owner is not null
    and current_user = cleanup_owner
    and cleanup_reference ~ '^phase-2e-6b-4o-staging-e2e:[0-9]{13}-[0-9a-f]{8}$'
  then return old;
  end if;

  raise exception 'immutable operations record cannot be updated or deleted';
end;
$$;

revoke all on function public.protect_relief_workflow_immutable_v1() from public;

drop trigger relief_public_updates_immutable on public.relief_public_updates;
create trigger relief_public_updates_immutable
before update or delete on public.relief_public_updates
for each row execute function public.protect_relief_workflow_immutable_v1();

drop trigger operations_relief_workflow_events_immutable
on public.operations_relief_workflow_events;
create trigger operations_relief_workflow_events_immutable
before update or delete on public.operations_relief_workflow_events
for each row execute function public.protect_relief_workflow_immutable_v1();

create or replace function public.cleanup_operations_relief_staging_e2e_v1(
  p_run_reference text,
  p_relief_application_ids uuid[]
)
returns table (
  public_updates_deleted integer,
  events_deleted integer,
  applications_deleted integer
)
language plpgsql security definer set search_path = '' as $$
declare
  v_run_id text;
  v_updates integer;
  v_events integer;
  v_apps integer;
begin
  if p_run_reference is null
    or p_run_reference <> trim(p_run_reference)
    or p_run_reference !~ '^phase-2e-6b-4o-staging-e2e:[0-9]{13}-[0-9a-f]{8}$'
  then raise exception 'invalid Phase 4O Staging E2E cleanup reference';
  end if;

  if p_relief_application_ids is null
    or cardinality(p_relief_application_ids) <> 2
    or (select count(distinct id) from unnest(p_relief_application_ids) fixture(id)) <> 2
  then raise exception 'Phase 4O cleanup requires two distinct application ids';
  end if;

  v_run_id := substring(
    p_run_reference from char_length('phase-2e-6b-4o-staging-e2e:') + 1
  );

  if (select count(*) from public.relief_applications application
      where application.id = any(p_relief_application_ids)
        and application.evidence_url in (
          'https://example.com/alpha-staging-relief-' || v_run_id || '-approve',
          'https://example.com/alpha-staging-relief-' || v_run_id || '-reject'
        )
        and application.status in ('approved', 'rejected')
        and application.payment_receipt_id is null) <> 2
  then raise exception 'cleanup applications are not exact Phase 4O fixtures';
  end if;

  if exists (
    select 1 from public.treasury_execution_intents intent
    where intent.relief_application_id = any(p_relief_application_ids)
  ) then raise exception 'cleanup refused because a treasury execution intent exists';
  end if;

  if (select count(*) from public.relief_public_updates update_record
      where update_record.case_reference = p_run_reference || ':approved'
        and update_record.outcome = 'approved') <> 1
  then raise exception 'cleanup public update is not the exact Phase 4O fixture';
  end if;

  if (select count(*) from public.operations_relief_workflow_events event
      where event.relief_application_id = any(p_relief_application_ids)
        and event.event_reference in (
          p_run_reference || ':approved', p_run_reference || ':rejected'
        )) <> 2
  then raise exception 'cleanup events are not exact Phase 4O fixtures';
  end if;

  perform set_config(
    'alpha.operations_relief_staging_e2e_cleanup_reference', p_run_reference, true
  );

  delete from public.relief_public_updates
  where case_reference = p_run_reference || ':approved';
  get diagnostics v_updates = row_count;

  delete from public.operations_relief_workflow_events
  where relief_application_id = any(p_relief_application_ids);
  get diagnostics v_events = row_count;

  delete from public.relief_applications
  where id = any(p_relief_application_ids);
  get diagnostics v_apps = row_count;

  if v_updates <> 1 or v_events <> 2 or v_apps <> 2 then
    raise exception 'Phase 4O Staging E2E cleanup count mismatch';
  end if;

  return query select v_updates, v_events, v_apps;
end;
$$;

comment on function public.cleanup_operations_relief_staging_e2e_v1(text, uuid[]) is
  'Deletes only the exact Phase 4O Staging relief fixture after strict validation.';

revoke all on function public.cleanup_operations_relief_staging_e2e_v1(text, uuid[])
from public, anon, authenticated, service_role;
grant execute on function public.cleanup_operations_relief_staging_e2e_v1(text, uuid[])
to service_role;

commit;
