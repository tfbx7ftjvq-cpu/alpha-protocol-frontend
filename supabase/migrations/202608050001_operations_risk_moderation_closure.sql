-- Alpha Protocol Phase 2E-6B-4N
-- Audited private risk evidence and sanitized publication closure.
--
-- Raw reports and evidence remain private. A separate reviewer may dismiss a
-- report or publish only reviewer-written sanitized fields after the reporter
-- explicitly consented. This migration performs no network request, Solana
-- transaction, treasury mutation, token action, or funds movement.

begin;

alter table public.risk_reports
  add column public_report_consent boolean not null default false,
  add column public_reference_consent boolean not null default false,
  add constraint risk_reports_reference_consent_requires_report_consent
    check (not public_reference_consent or public_report_consent);

comment on column public.risk_reports.public_report_consent is
  'Reporter consent for a reviewer-written sanitized public risk record after independent review.';
comment on column public.risk_reports.public_reference_consent is
  'Separate consent allowing a reviewer to include one safe public reference URL.';

create table public.operations_risk_workflow_events (
  event_id bigint generated always as identity primary key,
  risk_report_id uuid not null references public.risk_reports(id),
  action text not null check (action in ('report_published', 'report_dismissed')),
  actor_id uuid not null references auth.users(id),
  actor_role text not null check (
    actor_role in ('reviewer', 'operator', 'governance_admin')
  ),
  event_reference text not null unique check (
    char_length(trim(event_reference)) between 10 and 180
    and event_reference !~ '[[:cntrl:]]'
  ),
  event_data jsonb not null default '{}'::jsonb
    check (jsonb_typeof(event_data) = 'object'),
  created_at timestamptz not null default timezone('utc', now())
);

comment on table public.operations_risk_workflow_events is
  'Private append-only risk review audit history. It is evidence, not an insurance, credit-rating, payment, or treasury authority.';

create trigger operations_risk_workflow_events_immutable
before update or delete on public.operations_risk_workflow_events
for each row execute function public.reject_immutable_operations_mutation();

create index operations_risk_workflow_events_report_idx
on public.operations_risk_workflow_events (risk_report_id, created_at desc);

alter table public.operations_risk_workflow_events enable row level security;

create policy operations_risk_workflow_events_staff_read
on public.operations_risk_workflow_events
for select
to authenticated
using (public.has_operations_role(array['reviewer', 'operator', 'governance_admin']));

revoke all on table public.operations_risk_workflow_events
from public, anon, authenticated;
grant select on table public.operations_risk_workflow_events to authenticated;

-- Owners may add evidence only while their report remains non-terminal. The
-- evidence itself is never made public by this policy.
drop policy risk_evidence_owner_insert on public.risk_evidence;
create policy risk_evidence_owner_insert
on public.risk_evidence
for insert
to authenticated
with check (
  public.is_operations_wallet_intake_enabled()
  and submitted_by = auth.uid()
  and is_public = false
  and reviewed_by is null
  and exists (
    select 1
    from public.risk_reports report
    where report.id = risk_report_id
      and report.submitted_by = auth.uid()
      and report.review_status in ('submitted', 'triaged', 'investigating')
      and report.publication_status = 'private'
  )
);

create or replace function public.enforce_risk_evidence_rate_limit_v1()
returns trigger
language plpgsql
security definer
set search_path = ''
as $$
declare
  recent_rows bigint;
begin
  if not public.is_operations_wallet_intake_enabled() then
    raise exception 'operations wallet intake is disabled';
  end if;

  if new.submitted_by is distinct from auth.uid() then
    raise exception 'risk evidence owner does not match auth.uid()';
  end if;

  perform pg_advisory_xact_lock(
    hashtextextended(new.submitted_by::text || ':risk_evidence', 0)
  );

  select count(*)
  into recent_rows
  from public.risk_evidence evidence
  where evidence.submitted_by = new.submitted_by
    and evidence.created_at > now() - interval '1 hour';

  if recent_rows >= 12 then
    raise exception 'operations submission rate limit exceeded for risk_evidence';
  end if;

  new.created_at := now();
  return new;
end;
$$;

revoke all on function public.enforce_risk_evidence_rate_limit_v1() from public;

create trigger risk_evidence_rate_limit
before insert on public.risk_evidence
for each row execute function public.enforce_risk_evidence_rate_limit_v1();

-- Direct staff mutation is removed. Review, terminal-state transition,
-- sanitized publication and audit must succeed atomically through the RPC.
revoke update, delete on table public.risk_reports from authenticated;
revoke update, delete on table public.risk_evidence from authenticated;
revoke insert, update, delete on table public.risk_publications from authenticated;

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
  v_actor_role text := auth.jwt() -> 'app_metadata' ->> 'operations_role';
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

comment on function public.review_risk_report_v1(uuid, text, text, text, text, text, text) is
  'Role-gated atomic risk review. Public output is reviewer-written, consent-bound and sanitized; no payment or on-chain action.';

revoke all on function public.review_risk_report_v1(uuid, text, text, text, text, text, text) from public;
revoke all on function public.review_risk_report_v1(uuid, text, text, text, text, text, text) from anon;
grant execute on function public.review_risk_report_v1(uuid, text, text, text, text, text, text) to authenticated;

commit;
