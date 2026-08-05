-- Alpha Protocol Phase 2E-6B-4O
-- Audited relief review and sanitized public-progress closure.
-- Approval is a review outcome only. This migration cannot create a treasury
-- intent, payment receipt, Solana transaction, token action, or funds movement.

begin;

alter table public.relief_applications
  add column public_update_consent boolean not null default false;

comment on column public.relief_applications.public_update_consent is
  'Claimant consent for a reviewer-written sanitized public progress update. Consent never authorizes payment.';

create table public.operations_relief_workflow_events (
  event_id bigint generated always as identity primary key,
  relief_application_id uuid not null references public.relief_applications(id),
  action text not null check (action in ('application_approved', 'application_rejected')),
  actor_id uuid not null references auth.users(id),
  actor_role text not null check (
    actor_role in ('relief_reviewer', 'operator', 'governance_admin')
  ),
  event_reference text not null unique check (
    char_length(trim(event_reference)) between 10 and 180
    and event_reference !~ '[[:cntrl:]]'
  ),
  event_data jsonb not null default '{}'::jsonb
    check (jsonb_typeof(event_data) = 'object'),
  created_at timestamptz not null default timezone('utc', now())
);

comment on table public.operations_relief_workflow_events is
  'Private append-only relief-review audit history. Approval is not payment authority.';

create trigger operations_relief_workflow_events_immutable
before update or delete on public.operations_relief_workflow_events
for each row execute function public.reject_immutable_operations_mutation();

create index operations_relief_workflow_events_application_idx
on public.operations_relief_workflow_events (relief_application_id, created_at desc);

alter table public.operations_relief_workflow_events enable row level security;

create policy operations_relief_workflow_events_staff_read
on public.operations_relief_workflow_events
for select to authenticated
using (public.has_operations_role(array['relief_reviewer', 'operator', 'governance_admin']));

revoke all on table public.operations_relief_workflow_events
from public, anon, authenticated;
grant select on table public.operations_relief_workflow_events to authenticated;

-- All terminal review mutations must use the atomic RPC below.
revoke update, delete on table public.relief_applications from authenticated;
revoke insert, update, delete on table public.relief_public_updates from authenticated;

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
  v_actor_role text := auth.jwt() -> 'app_metadata' ->> 'operations_role';
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

  -- Traverse only listed state transitions; the terminal transition remains
  -- protected by the existing status-state trigger.
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

comment on function public.review_relief_application_v1(uuid, text, text, text, text, text, text) is
  'Atomically reviews one relief application, optionally publishes a consented sanitized update, and appends audit evidence. It cannot pay.';

revoke all on function public.review_relief_application_v1(uuid, text, text, text, text, text, text)
from public, anon, authenticated, service_role;
grant execute on function public.review_relief_application_v1(uuid, text, text, text, text, text, text)
to authenticated;

commit;
