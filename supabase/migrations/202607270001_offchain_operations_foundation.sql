-- Alpha Protocol Phase 2E-6B-4F
-- Off-chain operations foundation.
--
-- This database stores intake, review, discussion, publication, and execution
-- evidence. It has no private keys and no function that sends a Solana
-- transaction. A database decision is never treasury authority.

begin;

create extension if not exists pgcrypto;

create or replace function public.has_operations_role(allowed_roles text[])
returns boolean
language sql
stable
set search_path = ''
as $$
  select coalesce(
    (auth.jwt() -> 'app_metadata' ->> 'operations_role') = any(allowed_roles),
    false
  );
$$;

revoke all on function public.has_operations_role(text[]) from public;
grant execute on function public.has_operations_role(text[]) to anon, authenticated;

create or replace function public.set_operations_updated_at()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  new.updated_at = timezone('utc', now());
  return new;
end;
$$;

revoke all on function public.set_operations_updated_at() from public;

create or replace function public.reject_immutable_operations_mutation()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  raise exception 'immutable operations record cannot be updated or deleted';
end;
$$;

revoke all on function public.reject_immutable_operations_mutation() from public;

create or replace function public.protect_operations_columns()
returns trigger
language plpgsql
set search_path = ''
as $$
declare
  allowed_columns text[] := tg_argv;
begin
  if (to_jsonb(new) - allowed_columns) is distinct from (to_jsonb(old) - allowed_columns) then
    raise exception 'immutable fields on % cannot be changed', tg_table_name;
  end if;

  return new;
end;
$$;

revoke all on function public.protect_operations_columns() from public;

create or replace function public.protect_published_operations_record()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  if old.publication_status = 'published'
    and (to_jsonb(new) - array['status', 'publication_status', 'updated_at'])
      is distinct from
      (to_jsonb(old) - array['status', 'publication_status', 'updated_at'])
  then
    raise exception 'published fields on % cannot be changed', tg_table_name;
  end if;

  return new;
end;
$$;

revoke all on function public.protect_published_operations_record() from public;

create or replace function public.enforce_operations_state_transition()
returns trigger
language plpgsql
set search_path = ''
as $$
declare
  state_column text := tg_argv[0];
  old_state text := to_jsonb(old) ->> state_column;
  new_state text := to_jsonb(new) ->> state_column;
  edge text := old_state || '->' || new_state;
  edge_index integer;
begin
  if old_state = new_state then
    return new;
  end if;

  for edge_index in 1..(tg_nargs - 1) loop
    if edge = tg_argv[edge_index] then
      return new;
    end if;
  end loop;

  raise exception 'invalid % transition on %: %', state_column, tg_table_name, edge;
end;
$$;

revoke all on function public.enforce_operations_state_transition() from public;

create or replace function public.lock_operations_submission_signature()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  if old.submitted_signature is not null
    and new.submitted_signature is distinct from old.submitted_signature
  then
    raise exception 'submitted transaction signature cannot be changed or removed';
  end if;

  return new;
end;
$$;

revoke all on function public.lock_operations_submission_signature() from public;

create table public.community_tasks (
  id uuid primary key default gen_random_uuid(),
  title text not null check (char_length(title) between 4 and 160),
  summary text not null check (char_length(summary) between 20 and 3000),
  requirements text not null check (char_length(requirements) between 20 and 5000),
  reward_budget_usdc numeric(20, 6)
    check (reward_budget_usdc is null or reward_budget_usdc >= 0),
  reward_source text
    check (reward_source is null or reward_source in ('builders_pool', 'grant', 'sponsor', 'none')),
  status text not null default 'draft'
    check (status in ('draft', 'open', 'under_review', 'closed', 'cancelled')),
  publication_status text not null default 'draft'
    check (publication_status in ('draft', 'published', 'archived')),
  submission_deadline timestamptz,
  published_at timestamptz,
  created_at timestamptz not null default timezone('utc', now()),
  updated_at timestamptz not null default timezone('utc', now()),
  constraint community_tasks_published_at_required check (
    publication_status <> 'published' or published_at is not null
  ),
  constraint community_tasks_open_requires_publication check (
    status not in ('open', 'under_review') or publication_status = 'published'
  )
);

comment on table public.community_tasks is
  'Public community work definitions. Reward budgets are declarations, not payment instructions.';

create trigger community_tasks_set_updated_at
before update on public.community_tasks
for each row execute function public.set_operations_updated_at();

create trigger community_tasks_protect_published
before update on public.community_tasks
for each row execute function public.protect_published_operations_record();

create trigger community_tasks_enforce_status_transition
before update on public.community_tasks
for each row execute function public.enforce_operations_state_transition(
  'status',
  'draft->open',
  'draft->cancelled',
  'open->under_review',
  'open->closed',
  'open->cancelled',
  'under_review->open',
  'under_review->closed',
  'under_review->cancelled'
);

create table public.task_submissions (
  id uuid primary key default gen_random_uuid(),
  task_id uuid not null references public.community_tasks(id),
  submitted_by uuid not null references auth.users(id),
  summary text not null check (char_length(summary) between 20 and 5000),
  deliverable_url text not null check (
    deliverable_url ~ '^https://'
    and char_length(deliverable_url) <= 2000
  ),
  wallet_address text not null check (char_length(wallet_address) between 32 and 44),
  wallet_verified boolean not null default false check (wallet_verified = false),
  status text not null default 'submitted'
    check (status in ('submitted', 'in_review', 'accepted', 'rejected')),
  reviewer_notes text check (reviewer_notes is null or char_length(reviewer_notes) <= 5000),
  reviewed_by uuid references auth.users(id),
  reviewed_at timestamptz,
  created_at timestamptz not null default timezone('utc', now()),
  updated_at timestamptz not null default timezone('utc', now()),
  constraint task_submissions_review_fields check (
    (status in ('submitted', 'in_review') and reviewed_at is null)
    or
    (
      status in ('accepted', 'rejected')
      and reviewed_by is not null
      and reviewed_at is not null
    )
  )
);

comment on table public.task_submissions is
  'Private contributor submissions. Accepted does not mean paid and cannot trigger treasury execution.';

create trigger task_submissions_set_updated_at
before update on public.task_submissions
for each row execute function public.set_operations_updated_at();

create trigger task_submissions_protect_content
before update on public.task_submissions
for each row execute function public.protect_operations_columns(
  'status',
  'reviewer_notes',
  'reviewed_by',
  'reviewed_at',
  'updated_at'
);

create trigger task_submissions_enforce_status_transition
before update on public.task_submissions
for each row execute function public.enforce_operations_state_transition(
  'status',
  'submitted->in_review',
  'submitted->rejected',
  'in_review->accepted',
  'in_review->rejected'
);

create table public.risk_reports (
  id uuid primary key default gen_random_uuid(),
  submitted_by uuid not null references auth.users(id),
  project_identifier text not null check (char_length(project_identifier) between 2 and 160),
  summary text not null check (char_length(summary) between 30 and 5000),
  reference_url text not null check (
    reference_url ~ '^https://'
    and char_length(reference_url) <= 2000
  ),
  wallet_address text check (
    wallet_address is null or char_length(wallet_address) between 32 and 44
  ),
  wallet_verified boolean not null default false check (wallet_verified = false),
  review_status text not null default 'submitted'
    check (review_status in ('submitted', 'triaged', 'investigating', 'resolved', 'dismissed')),
  publication_status text not null default 'private'
    check (publication_status in ('private', 'published', 'archived')),
  reviewer_notes text check (reviewer_notes is null or char_length(reviewer_notes) <= 5000),
  reviewed_by uuid references auth.users(id),
  reviewed_at timestamptz,
  published_at timestamptz,
  created_at timestamptz not null default timezone('utc', now()),
  updated_at timestamptz not null default timezone('utc', now()),
  constraint risk_reports_publication_fields check (
    publication_status <> 'published' or published_at is not null
  ),
  constraint risk_reports_review_fields check (
    review_status not in ('resolved', 'dismissed')
    or (reviewed_by is not null and reviewed_at is not null)
  )
);

comment on table public.risk_reports is
  'Private risk intake. Public risk records must be copied into the sanitized risk_publications table after review.';

create trigger risk_reports_set_updated_at
before update on public.risk_reports
for each row execute function public.set_operations_updated_at();

create trigger risk_reports_protect_content
before update on public.risk_reports
for each row execute function public.protect_operations_columns(
  'review_status',
  'publication_status',
  'reviewer_notes',
  'reviewed_by',
  'reviewed_at',
  'published_at',
  'updated_at'
);

create trigger risk_reports_enforce_status_transition
before update on public.risk_reports
for each row execute function public.enforce_operations_state_transition(
  'review_status',
  'submitted->triaged',
  'submitted->dismissed',
  'triaged->investigating',
  'triaged->dismissed',
  'investigating->resolved',
  'investigating->dismissed'
);

create table public.risk_evidence (
  id uuid primary key default gen_random_uuid(),
  risk_report_id uuid not null references public.risk_reports(id),
  submitted_by uuid not null references auth.users(id),
  evidence_url text not null check (
    evidence_url ~ '^https://'
    and char_length(evidence_url) <= 2000
  ),
  content_sha256 text check (
    content_sha256 is null or content_sha256 ~ '^[0-9a-f]{64}$'
  ),
  summary text not null check (char_length(summary) between 10 and 2000),
  is_public boolean not null default false,
  reviewed_by uuid references auth.users(id),
  created_at timestamptz not null default timezone('utc', now())
);

comment on table public.risk_evidence is
  'Private evidence references and optional hashes. Public evidence must be separately sanitized.';

create trigger risk_evidence_protect_content
before update on public.risk_evidence
for each row execute function public.protect_operations_columns(
  'is_public',
  'reviewed_by'
);

create table public.risk_publications (
  id uuid primary key default gen_random_uuid(),
  report_reference text not null check (
    char_length(report_reference) between 6 and 80
    and report_reference !~ '[[:space:]]'
  ),
  supersedes_publication_id uuid unique references public.risk_publications(id),
  project_identifier text not null check (char_length(project_identifier) between 2 and 160),
  summary text not null check (char_length(summary) between 30 and 5000),
  reference_url text check (
    reference_url is null
    or (
      reference_url ~ '^https://'
      and char_length(reference_url) <= 2000
    )
  ),
  public_status text not null
    check (public_status in ('published', 'resolved', 'dismissed')),
  publication_basis text not null check (char_length(publication_basis) between 10 and 1000),
  published_at timestamptz not null default timezone('utc', now())
);

comment on table public.risk_publications is
  'Immutable sanitized public risk records. Corrections append a superseding record.';

create trigger risk_publications_immutable
before update or delete on public.risk_publications
for each row execute function public.reject_immutable_operations_mutation();

create table public.relief_applications (
  id uuid primary key default gen_random_uuid(),
  submitted_by uuid not null references auth.users(id),
  incident_summary text not null check (char_length(incident_summary) between 50 and 8000),
  requested_amount_usdc numeric(20, 6) not null
    check (requested_amount_usdc > 0 and requested_amount_usdc <= 1000000000),
  evidence_url text not null check (
    evidence_url ~ '^https://'
    and char_length(evidence_url) <= 2000
  ),
  wallet_address text not null check (char_length(wallet_address) between 32 and 44),
  wallet_verified boolean not null default false check (wallet_verified = false),
  status text not null default 'submitted'
    check (status in (
      'submitted',
      'triaged',
      'evidence_requested',
      'under_review',
      'approved',
      'rejected',
      'cancelled',
      'paid'
    )),
  reviewer_notes text check (reviewer_notes is null or char_length(reviewer_notes) <= 5000),
  reviewed_by uuid references auth.users(id),
  reviewed_at timestamptz,
  payment_receipt_id uuid,
  created_at timestamptz not null default timezone('utc', now()),
  updated_at timestamptz not null default timezone('utc', now()),
  constraint relief_applications_review_fields check (
    status not in ('approved', 'rejected', 'cancelled', 'paid')
    or (reviewed_by is not null and reviewed_at is not null)
  ),
  constraint relief_applications_paid_receipt check (
    (status = 'paid') = (payment_receipt_id is not null)
  )
);

comment on table public.relief_applications is
  'Private claimant intake. Approved is a review outcome only; it never authorizes or sends a payout.';

create trigger relief_applications_set_updated_at
before update on public.relief_applications
for each row execute function public.set_operations_updated_at();

create trigger relief_applications_protect_content
before update on public.relief_applications
for each row execute function public.protect_operations_columns(
  'status',
  'reviewer_notes',
  'reviewed_by',
  'reviewed_at',
  'payment_receipt_id',
  'updated_at'
);

create trigger relief_applications_enforce_status_transition
before update on public.relief_applications
for each row execute function public.enforce_operations_state_transition(
  'status',
  'submitted->triaged',
  'submitted->rejected',
  'submitted->cancelled',
  'triaged->evidence_requested',
  'triaged->under_review',
  'triaged->rejected',
  'triaged->cancelled',
  'evidence_requested->under_review',
  'evidence_requested->rejected',
  'evidence_requested->cancelled',
  'under_review->approved',
  'under_review->rejected',
  'under_review->cancelled',
  'approved->paid',
  'approved->cancelled'
);

create table public.relief_public_updates (
  id uuid primary key default gen_random_uuid(),
  case_reference text not null check (
    char_length(case_reference) between 6 and 80
    and case_reference !~ '[[:space:]]'
  ),
  supersedes_update_id uuid unique references public.relief_public_updates(id),
  title text not null check (char_length(title) between 4 and 160),
  summary text not null check (char_length(summary) between 20 and 3000),
  outcome text not null check (
    outcome in ('reviewing', 'approved', 'rejected', 'paid', 'cancelled')
  ),
  publication_basis text not null check (char_length(publication_basis) between 10 and 1000),
  published_at timestamptz not null default timezone('utc', now())
);

comment on table public.relief_public_updates is
  'Immutable sanitized public relief updates. Corrections append a superseding update.';

create trigger relief_public_updates_immutable
before update or delete on public.relief_public_updates
for each row execute function public.reject_immutable_operations_mutation();

create table public.governance_proposals (
  id uuid primary key default gen_random_uuid(),
  title text not null check (char_length(title) between 4 and 160),
  summary text not null check (char_length(summary) between 20 and 5000),
  proposal_kind text not null check (
    proposal_kind in (
      'task_acceptance',
      'risk_finding',
      'relief_recommendation',
      'builders_spend',
      'buyback_policy',
      'staking_policy',
      'protocol_parameter',
      'other'
    )
  ),
  public_source_reference text check (
    public_source_reference is null
    or char_length(public_source_reference) between 6 and 160
  ),
  execution_required boolean not null default false,
  execution_manifest_url text check (
    execution_manifest_url is null
    or (
      execution_manifest_url ~ '^https://'
      and char_length(execution_manifest_url) <= 2000
    )
  ),
  status text not null default 'draft'
    check (status in ('draft', 'discussion', 'voting', 'decided', 'cancelled')),
  publication_status text not null default 'draft'
    check (publication_status in ('draft', 'published', 'archived')),
  published_at timestamptz,
  created_at timestamptz not null default timezone('utc', now()),
  updated_at timestamptz not null default timezone('utc', now()),
  constraint governance_proposals_publication_fields check (
    publication_status <> 'published' or published_at is not null
  ),
  constraint governance_execution_manifest_boundary check (
    not execution_required or execution_manifest_url is not null
  ),
  constraint governance_active_requires_publication check (
    status not in ('discussion', 'voting', 'decided')
    or publication_status = 'published'
  )
);

comment on table public.governance_proposals is
  'Off-chain governance publication records. They do not execute programs or move treasury assets.';

create trigger governance_proposals_set_updated_at
before update on public.governance_proposals
for each row execute function public.set_operations_updated_at();

create trigger governance_proposals_protect_published
before update on public.governance_proposals
for each row execute function public.protect_published_operations_record();

create trigger governance_proposals_enforce_status_transition
before update on public.governance_proposals
for each row execute function public.enforce_operations_state_transition(
  'status',
  'draft->discussion',
  'draft->cancelled',
  'discussion->voting',
  'discussion->cancelled',
  'voting->decided',
  'voting->cancelled'
);

create table public.governance_discussions (
  id uuid primary key default gen_random_uuid(),
  proposal_id uuid references public.governance_proposals(id),
  submitted_by uuid not null references auth.users(id),
  topic text not null check (char_length(topic) between 4 and 160),
  body text not null check (char_length(body) between 20 and 5000),
  wallet_address text check (
    wallet_address is null or char_length(wallet_address) between 32 and 44
  ),
  wallet_verified boolean not null default false check (wallet_verified = false),
  moderation_status text not null default 'pending'
    check (moderation_status in ('pending', 'published', 'rejected', 'archived')),
  moderated_by uuid references auth.users(id),
  created_at timestamptz not null default timezone('utc', now()),
  updated_at timestamptz not null default timezone('utc', now()),
  constraint governance_discussions_moderation_fields check (
    moderation_status = 'pending' or moderated_by is not null
  )
);

comment on table public.governance_discussions is
  'Private moderated community discussion intake. Wallet text is self-asserted until separately verified.';

create trigger governance_discussions_set_updated_at
before update on public.governance_discussions
for each row execute function public.set_operations_updated_at();

create trigger governance_discussions_protect_content
before update on public.governance_discussions
for each row execute function public.protect_operations_columns(
  'moderation_status',
  'moderated_by',
  'updated_at'
);

create trigger governance_discussions_enforce_status_transition
before update on public.governance_discussions
for each row execute function public.enforce_operations_state_transition(
  'moderation_status',
  'pending->published',
  'pending->rejected',
  'published->archived',
  'rejected->archived'
);

create table public.governance_discussion_publications (
  id uuid primary key default gen_random_uuid(),
  discussion_reference text not null unique check (
    char_length(discussion_reference) between 6 and 80
    and discussion_reference !~ '[[:space:]]'
  ),
  supersedes_publication_id uuid unique references public.governance_discussion_publications(id),
  proposal_id uuid references public.governance_proposals(id),
  topic text not null check (char_length(topic) between 4 and 160),
  body text not null check (char_length(body) between 20 and 5000),
  wallet_address text check (
    wallet_address is null or char_length(wallet_address) between 32 and 44
  ),
  wallet_verified boolean not null default false check (wallet_verified = false),
  publication_basis text not null check (char_length(publication_basis) between 10 and 1000),
  published_at timestamptz not null default timezone('utc', now())
);

comment on table public.governance_discussion_publications is
  'Immutable sanitized public discussion records with no auth user ID or private discussion foreign key.';

create trigger governance_discussion_publications_immutable
before update or delete on public.governance_discussion_publications
for each row execute function public.reject_immutable_operations_mutation();

create table public.governance_decisions (
  id uuid primary key default gen_random_uuid(),
  proposal_id uuid not null unique references public.governance_proposals(id),
  decision text not null check (decision in ('approved', 'rejected', 'cancelled')),
  rationale text not null check (char_length(rationale) between 20 and 5000),
  decision_hash text not null unique check (decision_hash ~ '^[0-9a-f]{64}$'),
  execution_required boolean not null,
  execution_reference text check (
    execution_reference is null or char_length(execution_reference) between 6 and 2000
  ),
  publication_status text not null default 'published'
    check (publication_status = 'published'),
  decided_at timestamptz not null default timezone('utc', now()),
  constraint governance_decision_execution_boundary check (
    not execution_required or execution_reference is not null
  )
);

comment on table public.governance_decisions is
  'Immutable public decision receipts. Approval is not a transaction and cannot move treasury funds.';

create trigger governance_decisions_immutable
before update or delete on public.governance_decisions
for each row execute function public.reject_immutable_operations_mutation();

create table public.treasury_execution_intents (
  id uuid primary key default gen_random_uuid(),
  governance_decision_id uuid not null unique references public.governance_decisions(id),
  pool text not null check (pool in ('relief', 'buyback', 'builders', 'staking')),
  relief_application_id uuid unique references public.relief_applications(id),
  network text not null check (network in ('devnet', 'mainnet-beta')),
  asset_symbol text not null default 'USDC' check (asset_symbol = 'USDC'),
  asset_decimals smallint not null default 6 check (asset_decimals = 6),
  asset_mint text not null check (char_length(asset_mint) between 32 and 44),
  destination_wallet text not null check (char_length(destination_wallet) between 32 and 44),
  amount_base_units numeric(30, 0) not null check (amount_base_units > 0),
  recipient_verification_reference text not null check (
    char_length(recipient_verification_reference) between 12 and 2000
  ),
  manifest_sha256 text not null unique check (manifest_sha256 ~ '^[0-9a-f]{64}$'),
  status text not null default 'prepared'
    check (status in ('prepared', 'submitted', 'confirmed', 'cancelled', 'failed')),
  prepared_by uuid not null references auth.users(id),
  submitted_signature text unique,
  created_at timestamptz not null default timezone('utc', now()),
  updated_at timestamptz not null default timezone('utc', now()),
  constraint treasury_execution_intent_signature_state check (
    (status = 'prepared' and submitted_signature is null)
    or (status in ('submitted', 'confirmed') and submitted_signature is not null)
    or (status in ('cancelled', 'failed'))
  ),
  constraint treasury_execution_intent_relief_binding check (
    (pool = 'relief') = (relief_application_id is not null)
  )
);

comment on table public.treasury_execution_intents is
  'Private executor work queue. It stores a deterministic manifest only and contains no transaction sender or signing key.';

create trigger treasury_execution_intents_set_updated_at
before update on public.treasury_execution_intents
for each row execute function public.set_operations_updated_at();

create trigger treasury_execution_intents_protect_manifest
before update on public.treasury_execution_intents
for each row execute function public.protect_operations_columns(
  'status',
  'submitted_signature',
  'updated_at'
);

create trigger treasury_execution_intents_enforce_status_transition
before update on public.treasury_execution_intents
for each row execute function public.enforce_operations_state_transition(
  'status',
  'prepared->submitted',
  'prepared->cancelled',
  'prepared->failed',
  'submitted->confirmed',
  'submitted->cancelled',
  'submitted->failed'
);

create trigger treasury_execution_intents_lock_signature
before update on public.treasury_execution_intents
for each row execute function public.lock_operations_submission_signature();

create table public.treasury_execution_receipts (
  id uuid primary key default gen_random_uuid(),
  execution_intent_id uuid not null unique references public.treasury_execution_intents(id),
  governance_decision_id uuid not null unique references public.governance_decisions(id),
  chain text not null default 'solana' check (chain = 'solana'),
  network text not null check (network in ('devnet', 'mainnet-beta')),
  transaction_signature text not null unique check (
    char_length(transaction_signature) between 64 and 100
  ),
  manifest_sha256 text not null check (manifest_sha256 ~ '^[0-9a-f]{64}$'),
  receipt_sha256 text not null unique check (receipt_sha256 ~ '^[0-9a-f]{64}$'),
  confirmed_at timestamptz not null,
  recorded_at timestamptz not null default timezone('utc', now())
);

comment on table public.treasury_execution_receipts is
  'Immutable public evidence of a separately signed and confirmed chain transaction.';

create trigger treasury_execution_receipts_immutable
before update or delete on public.treasury_execution_receipts
for each row execute function public.reject_immutable_operations_mutation();

alter table public.relief_applications
add constraint relief_applications_payment_receipt_fk
foreign key (payment_receipt_id)
references public.treasury_execution_receipts(id);

create or replace function public.validate_relief_payment_receipt()
returns trigger
language plpgsql
security definer
set search_path = ''
as $$
begin
  if new.status = 'paid' and not exists (
    select 1
    from public.treasury_execution_receipts receipt
    join public.treasury_execution_intents intent
      on intent.id = receipt.execution_intent_id
    where receipt.id = new.payment_receipt_id
      and intent.relief_application_id = new.id
      and intent.pool = 'relief'
      and intent.destination_wallet = new.wallet_address
      and intent.status = 'confirmed'
  ) then
    raise exception 'paid relief application requires a matching confirmed relief execution receipt';
  end if;

  return new;
end;
$$;

revoke all on function public.validate_relief_payment_receipt() from public;

create trigger relief_applications_validate_payment_receipt
before update on public.relief_applications
for each row execute function public.validate_relief_payment_receipt();

create index task_submissions_submitted_by_created_at_idx
on public.task_submissions (submitted_by, created_at desc);

create index risk_reports_submitted_by_created_at_idx
on public.risk_reports (submitted_by, created_at desc);

create index risk_evidence_report_idx
on public.risk_evidence (risk_report_id, created_at desc);

create index relief_applications_submitted_by_created_at_idx
on public.relief_applications (submitted_by, created_at desc);

create index governance_discussions_submitted_by_created_at_idx
on public.governance_discussions (submitted_by, created_at desc);

create index community_tasks_publication_idx
on public.community_tasks (publication_status, status, published_at desc);

create index risk_publications_published_at_idx
on public.risk_publications (published_at desc);

create index risk_publications_reference_idx
on public.risk_publications (report_reference, published_at desc);

create index relief_public_updates_published_at_idx
on public.relief_public_updates (published_at desc);

create index relief_public_updates_reference_idx
on public.relief_public_updates (case_reference, published_at desc);

create index governance_discussion_publications_published_at_idx
on public.governance_discussion_publications (published_at desc);

create index governance_decisions_decided_at_idx
on public.governance_decisions (decided_at desc);

alter table public.community_tasks enable row level security;
alter table public.task_submissions enable row level security;
alter table public.risk_reports enable row level security;
alter table public.risk_evidence enable row level security;
alter table public.risk_publications enable row level security;
alter table public.relief_applications enable row level security;
alter table public.relief_public_updates enable row level security;
alter table public.governance_proposals enable row level security;
alter table public.governance_discussions enable row level security;
alter table public.governance_discussion_publications enable row level security;
alter table public.governance_decisions enable row level security;
alter table public.treasury_execution_intents enable row level security;
alter table public.treasury_execution_receipts enable row level security;

create policy community_tasks_public_read
on public.community_tasks
for select
to anon, authenticated
using (publication_status = 'published');

create policy community_tasks_operator_manage
on public.community_tasks
for all
to authenticated
using (public.has_operations_role(array['operator', 'governance_admin']))
with check (public.has_operations_role(array['operator', 'governance_admin']));

create policy task_submissions_owner_read
on public.task_submissions
for select
to authenticated
using (submitted_by = auth.uid());

create policy task_submissions_reviewer_read
on public.task_submissions
for select
to authenticated
using (public.has_operations_role(array['reviewer', 'operator', 'governance_admin']));

create policy task_submissions_owner_insert
on public.task_submissions
for insert
to authenticated
with check (
  submitted_by = auth.uid()
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

create policy task_submissions_reviewer_update
on public.task_submissions
for update
to authenticated
using (public.has_operations_role(array['reviewer', 'operator', 'governance_admin']))
with check (public.has_operations_role(array['reviewer', 'operator', 'governance_admin']));

create policy risk_publications_public_read
on public.risk_publications
for select
to anon, authenticated
using (true);

create policy risk_publications_operator_insert
on public.risk_publications
for insert
to authenticated
with check (public.has_operations_role(array['reviewer', 'operator', 'governance_admin']));

create policy risk_reports_owner_read
on public.risk_reports
for select
to authenticated
using (submitted_by = auth.uid());

create policy risk_reports_reviewer_read
on public.risk_reports
for select
to authenticated
using (public.has_operations_role(array['reviewer', 'operator', 'governance_admin']));

create policy risk_reports_owner_insert
on public.risk_reports
for insert
to authenticated
with check (
  submitted_by = auth.uid()
  and review_status = 'submitted'
  and publication_status = 'private'
  and published_at is null
  and wallet_verified = false
  and reviewer_notes is null
  and reviewed_by is null
  and reviewed_at is null
);

create policy risk_reports_reviewer_update
on public.risk_reports
for update
to authenticated
using (public.has_operations_role(array['reviewer', 'operator', 'governance_admin']))
with check (public.has_operations_role(array['reviewer', 'operator', 'governance_admin']));

create policy risk_evidence_owner_read
on public.risk_evidence
for select
to authenticated
using (submitted_by = auth.uid());

create policy risk_evidence_reviewer_read
on public.risk_evidence
for select
to authenticated
using (public.has_operations_role(array['reviewer', 'operator', 'governance_admin']));

create policy risk_evidence_owner_insert
on public.risk_evidence
for insert
to authenticated
with check (
  submitted_by = auth.uid()
  and is_public = false
  and reviewed_by is null
  and exists (
    select 1
    from public.risk_reports report
    where report.id = risk_report_id
      and report.submitted_by = auth.uid()
  )
);

create policy risk_evidence_reviewer_update
on public.risk_evidence
for update
to authenticated
using (public.has_operations_role(array['reviewer', 'operator', 'governance_admin']))
with check (public.has_operations_role(array['reviewer', 'operator', 'governance_admin']));

create policy relief_applications_owner_read
on public.relief_applications
for select
to authenticated
using (submitted_by = auth.uid());

create policy relief_applications_reviewer_read
on public.relief_applications
for select
to authenticated
using (public.has_operations_role(array['relief_reviewer', 'operator', 'governance_admin']));

create policy relief_applications_owner_insert
on public.relief_applications
for insert
to authenticated
with check (
  submitted_by = auth.uid()
  and status = 'submitted'
  and wallet_verified = false
  and reviewer_notes is null
  and reviewed_by is null
  and reviewed_at is null
);

create policy relief_applications_reviewer_update
on public.relief_applications
for update
to authenticated
using (public.has_operations_role(array['relief_reviewer', 'operator', 'governance_admin']))
with check (public.has_operations_role(array['relief_reviewer', 'operator', 'governance_admin']));

create policy relief_public_updates_public_read
on public.relief_public_updates
for select
to anon, authenticated
using (true);

create policy relief_public_updates_operator_insert
on public.relief_public_updates
for insert
to authenticated
with check (public.has_operations_role(array['relief_reviewer', 'operator', 'governance_admin']));

create policy governance_proposals_public_read
on public.governance_proposals
for select
to anon, authenticated
using (publication_status = 'published');

create policy governance_proposals_operator_manage
on public.governance_proposals
for all
to authenticated
using (public.has_operations_role(array['operator', 'governance_admin']))
with check (public.has_operations_role(array['operator', 'governance_admin']));

create policy governance_discussion_publications_public_read
on public.governance_discussion_publications
for select
to anon, authenticated
using (true);

create policy governance_discussion_publications_operator_insert
on public.governance_discussion_publications
for insert
to authenticated
with check (public.has_operations_role(array['moderator', 'operator', 'governance_admin']));

create policy governance_discussions_owner_read
on public.governance_discussions
for select
to authenticated
using (submitted_by = auth.uid());

create policy governance_discussions_owner_insert
on public.governance_discussions
for insert
to authenticated
with check (
  submitted_by = auth.uid()
  and moderation_status = 'pending'
  and wallet_verified = false
  and moderated_by is null
);

create policy governance_discussions_operator_update
on public.governance_discussions
for update
to authenticated
using (public.has_operations_role(array['moderator', 'operator', 'governance_admin']))
with check (public.has_operations_role(array['moderator', 'operator', 'governance_admin']));

create policy governance_decisions_public_read
on public.governance_decisions
for select
to anon, authenticated
using (publication_status = 'published');

create policy governance_decisions_admin_insert
on public.governance_decisions
for insert
to authenticated
with check (
  public.has_operations_role(array['governance_admin'])
  and exists (
    select 1
    from public.governance_proposals proposal
    where proposal.id = governance_decisions.proposal_id
      and proposal.publication_status = 'published'
      and proposal.status = 'decided'
      and proposal.execution_required = governance_decisions.execution_required
      and (
        not governance_decisions.execution_required
        or proposal.execution_manifest_url = governance_decisions.execution_reference
      )
  )
);

create policy treasury_execution_intents_executor_read
on public.treasury_execution_intents
for select
to authenticated
using (public.has_operations_role(array['executor', 'governance_admin']));

create policy treasury_execution_intents_executor_insert
on public.treasury_execution_intents
for insert
to authenticated
with check (
  public.has_operations_role(array['executor', 'governance_admin'])
  and prepared_by = auth.uid()
  and exists (
    select 1
    from public.governance_decisions decision
    where decision.id = treasury_execution_intents.governance_decision_id
      and decision.decision = 'approved'
      and decision.execution_required
  )
  and (
    treasury_execution_intents.pool <> 'relief'
    or exists (
      select 1
      from public.relief_applications application
      where application.id = treasury_execution_intents.relief_application_id
        and application.status = 'approved'
        and application.payment_receipt_id is null
        and application.wallet_address = treasury_execution_intents.destination_wallet
        and treasury_execution_intents.amount_base_units
          <= application.requested_amount_usdc * 1000000
    )
  )
);

create policy treasury_execution_intents_executor_update
on public.treasury_execution_intents
for update
to authenticated
using (public.has_operations_role(array['executor', 'governance_admin']))
with check (public.has_operations_role(array['executor', 'governance_admin']));

create policy treasury_execution_receipts_public_read
on public.treasury_execution_receipts
for select
to anon, authenticated
using (true);

create policy treasury_execution_receipts_executor_insert
on public.treasury_execution_receipts
for insert
to authenticated
with check (
  public.has_operations_role(array['executor', 'governance_admin'])
  and exists (
    select 1
    from public.treasury_execution_intents intent
    where intent.id = treasury_execution_receipts.execution_intent_id
      and intent.governance_decision_id = treasury_execution_receipts.governance_decision_id
      and intent.manifest_sha256 = treasury_execution_receipts.manifest_sha256
      and intent.network = treasury_execution_receipts.network
      and intent.status = 'confirmed'
      and intent.submitted_signature = treasury_execution_receipts.transaction_signature
  )
);

revoke all on table public.community_tasks from anon, authenticated;
revoke all on table public.task_submissions from anon, authenticated;
revoke all on table public.risk_reports from anon, authenticated;
revoke all on table public.risk_evidence from anon, authenticated;
revoke all on table public.risk_publications from anon, authenticated;
revoke all on table public.relief_applications from anon, authenticated;
revoke all on table public.relief_public_updates from anon, authenticated;
revoke all on table public.governance_proposals from anon, authenticated;
revoke all on table public.governance_discussions from anon, authenticated;
revoke all on table public.governance_discussion_publications from anon, authenticated;
revoke all on table public.governance_decisions from anon, authenticated;
revoke all on table public.treasury_execution_intents from anon, authenticated;
revoke all on table public.treasury_execution_receipts from anon, authenticated;

grant select on table public.community_tasks to anon, authenticated;
grant select, insert, update on table public.task_submissions to authenticated;
grant select, insert, update on table public.risk_reports to authenticated;
grant select, insert, update on table public.risk_evidence to authenticated;
grant select on table public.risk_publications to anon;
grant select, insert on table public.risk_publications to authenticated;
grant select, insert, update on table public.relief_applications to authenticated;
grant select on table public.relief_public_updates to anon;
grant select, insert on table public.relief_public_updates to authenticated;
grant select on table public.governance_proposals to anon;
grant select, insert, update on table public.governance_proposals to authenticated;
grant select, insert, update on table public.governance_discussions to authenticated;
grant select on table public.governance_discussion_publications to anon;
grant select, insert on table public.governance_discussion_publications to authenticated;
grant select on table public.governance_decisions to anon;
grant select, insert on table public.governance_decisions to authenticated;
grant select, insert, update on table public.community_tasks to authenticated;
grant select, insert, update on table public.treasury_execution_intents to authenticated;
grant select on table public.treasury_execution_receipts to anon;
grant select, insert on table public.treasury_execution_receipts to authenticated;

commit;
