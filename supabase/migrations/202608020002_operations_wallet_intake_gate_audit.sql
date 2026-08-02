-- Alpha Protocol Phase 2E-6B-4L
-- Auditable, service-role-only control for the wallet-authenticated Staging
-- intake gate.
--
-- Applying this migration does not enable intake. It adds an append-only event
-- history and one narrowly granted RPC for a later, explicitly confirmed gate
-- change. No browser role can read or mutate the control or audit records.

begin;

create table public.operations_intake_gate_events (
  event_id bigint generated always as identity primary key,
  previous_mode text not null check (previous_mode in ('disabled', 'wallet_staging')),
  new_mode text not null check (new_mode in ('disabled', 'wallet_staging')),
  change_reference text not null
    check (
      char_length(trim(change_reference)) between 10 and 200
      and change_reference !~ '[[:cntrl:]]'
    ),
  changed_at timestamptz not null default timezone('utc', now()),
  check (previous_mode <> new_mode)
);

comment on table public.operations_intake_gate_events is
  'Append-only audit history for explicitly confirmed Staging intake gate changes.';

alter table public.operations_intake_gate_events enable row level security;

revoke all on table public.operations_intake_gate_events
from public, anon, authenticated, service_role;
revoke all on table public.operations_intake_control from service_role;

grant select on table public.operations_intake_control to service_role;
grant select on table public.operations_intake_gate_events to service_role;

create trigger operations_intake_gate_events_immutable
before update or delete on public.operations_intake_gate_events
for each row execute function public.reject_immutable_operations_mutation();

create or replace function public.set_operations_wallet_intake_mode(
  p_requested_mode text,
  p_change_reference text
)
returns table (
  mode text,
  activation_reference text,
  updated_at timestamptz,
  event_id bigint
)
language plpgsql
security definer
set search_path = ''
as $$
declare
  previous_gate_mode text;
  normalized_reference text;
  resulting_activation_reference text;
  resulting_updated_at timestamptz;
  created_event_id bigint;
begin
  if p_requested_mode is null
    or p_requested_mode not in ('disabled', 'wallet_staging')
  then
    raise exception 'unsupported operations intake mode';
  end if;

  normalized_reference := trim(p_change_reference);
  if normalized_reference is null
    or char_length(normalized_reference) not between 10 and 200
    or normalized_reference ~ '[[:cntrl:]]'
  then
    raise exception 'operations intake gate change reference must be 10 to 200 characters without control characters';
  end if;

  select control.mode
  into previous_gate_mode
  from public.operations_intake_control control
  where control.singleton
  for update;

  if not found then
    raise exception 'operations intake control singleton is missing';
  end if;

  if previous_gate_mode = p_requested_mode then
    raise exception 'operations intake gate is already in requested mode';
  end if;

  update public.operations_intake_control control
  set
    mode = p_requested_mode,
    activation_reference = case
      when p_requested_mode = 'wallet_staging' then normalized_reference
      else null
    end
  where control.singleton
  returning control.activation_reference, control.updated_at
  into resulting_activation_reference, resulting_updated_at;

  insert into public.operations_intake_gate_events (
    previous_mode,
    new_mode,
    change_reference,
    changed_at
  ) values (
    previous_gate_mode,
    p_requested_mode,
    normalized_reference,
    resulting_updated_at
  )
  returning operations_intake_gate_events.event_id
  into created_event_id;

  return query
  select
    p_requested_mode,
    resulting_activation_reference,
    resulting_updated_at,
    created_event_id;
end;
$$;

comment on function public.set_operations_wallet_intake_mode(text, text) is
  'Service-role-only, audited Staging gate transition. Applying this migration does not call it.';

revoke all on function public.set_operations_wallet_intake_mode(text, text) from public;
revoke all on function public.set_operations_wallet_intake_mode(text, text) from anon;
revoke all on function public.set_operations_wallet_intake_mode(text, text) from authenticated;
grant execute on function public.set_operations_wallet_intake_mode(text, text) to service_role;

commit;
