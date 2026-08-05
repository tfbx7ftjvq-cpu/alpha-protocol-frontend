-- Alpha Protocol Phase 2E-6B-4O corrective migration
-- Narrow read-only payment-state proof for the exact relief Staging E2E graph.

begin;

create or replace function public.inspect_operations_relief_staging_e2e_payment_state_v1(
  p_run_reference text,
  p_relief_application_ids uuid[]
)
returns table (
  applications_matched integer,
  treasury_intents_found integer,
  payment_receipts_found integer
)
language plpgsql security definer set search_path = '' as $$
declare
  v_run_id text;
  v_applications integer;
  v_intents integer;
  v_receipts integer;
begin
  if p_run_reference is null
    or p_run_reference <> trim(p_run_reference)
    or p_run_reference !~ '^phase-2e-6b-4o-staging-e2e:[0-9]{13}-[0-9a-f]{8}$'
  then raise exception 'invalid Phase 4O Staging E2E payment inspection reference';
  end if;

  if p_relief_application_ids is null
    or cardinality(p_relief_application_ids) <> 2
    or (select count(distinct id) from unnest(p_relief_application_ids) fixture(id)) <> 2
  then raise exception 'Phase 4O payment inspection requires two distinct application ids';
  end if;

  v_run_id := substring(
    p_run_reference from char_length('phase-2e-6b-4o-staging-e2e:') + 1
  );

  select count(*)::integer
  into v_applications
  from public.relief_applications application
  where application.id = any(p_relief_application_ids)
    and application.evidence_url in (
      'https://example.com/alpha-staging-relief-' || v_run_id || '-approve',
      'https://example.com/alpha-staging-relief-' || v_run_id || '-reject'
    )
    and application.status in ('approved', 'rejected');

  if v_applications <> 2 then
    raise exception 'payment inspection applications are not exact Phase 4O fixtures';
  end if;

  select count(*)::integer
  into v_intents
  from public.treasury_execution_intents intent
  where intent.relief_application_id = any(p_relief_application_ids);

  select count(distinct receipt_id)::integer
  into v_receipts
  from (
    select application.payment_receipt_id as receipt_id
    from public.relief_applications application
    where application.id = any(p_relief_application_ids)
      and application.payment_receipt_id is not null
    union
    select receipt.id as receipt_id
    from public.treasury_execution_receipts receipt
    join public.treasury_execution_intents intent
      on intent.id = receipt.execution_intent_id
    where intent.relief_application_id = any(p_relief_application_ids)
  ) matching_receipts;

  return query select v_applications, v_intents, v_receipts;
end;
$$;

comment on function public.inspect_operations_relief_staging_e2e_payment_state_v1(
  text, uuid[]
) is
  'Returns read-only payment-state counts for only the exact Phase 4O Staging relief fixture.';

revoke all on function public.inspect_operations_relief_staging_e2e_payment_state_v1(
  text, uuid[]
) from public, anon, authenticated, service_role;
grant execute on function public.inspect_operations_relief_staging_e2e_payment_state_v1(
  text, uuid[]
) to service_role;

commit;
