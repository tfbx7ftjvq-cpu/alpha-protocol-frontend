-- Run with `supabase test db` against a disposable local or staging-linked
-- database after both operations migrations have been applied.
--
-- pgTAP is diagnostic only. This file creates no protocol integration and
-- sends no network or Solana transaction.

begin;

create schema if not exists extensions;
create extension if not exists pgtap with schema extensions;
set local search_path = public, extensions;

select no_plan();

select has_table('public'::name, 'community_tasks'::name);
select has_table('public'::name, 'task_submissions'::name);
select has_table('public'::name, 'risk_reports'::name);
select has_table('public'::name, 'risk_evidence'::name);
select has_table('public'::name, 'risk_publications'::name);
select has_table('public'::name, 'relief_applications'::name);
select has_table('public'::name, 'relief_public_updates'::name);
select has_table('public'::name, 'governance_proposals'::name);
select has_table('public'::name, 'governance_discussions'::name);
select has_table('public'::name, 'governance_discussion_publications'::name);
select has_table('public'::name, 'governance_decisions'::name);
select has_table('public'::name, 'treasury_execution_intents'::name);
select has_table('public'::name, 'treasury_execution_receipts'::name);

select is(
  (
    select count(*)::integer
    from pg_catalog.pg_tables
    where schemaname = 'public'
      and tablename = any(array[
        'community_tasks',
        'task_submissions',
        'risk_reports',
        'risk_evidence',
        'risk_publications',
        'relief_applications',
        'relief_public_updates',
        'governance_proposals',
        'governance_discussions',
        'governance_discussion_publications',
        'governance_decisions',
        'treasury_execution_intents',
        'treasury_execution_receipts'
      ])
  ),
  13,
  'all 13 operations tables exist'
);

select is(
  (
    select count(*)::integer
    from pg_catalog.pg_class relation
    join pg_catalog.pg_namespace namespace on namespace.oid = relation.relnamespace
    where namespace.nspname = 'public'
      and relation.relname = any(array[
        'community_tasks',
        'task_submissions',
        'risk_reports',
        'risk_evidence',
        'risk_publications',
        'relief_applications',
        'relief_public_updates',
        'governance_proposals',
        'governance_discussions',
        'governance_discussion_publications',
        'governance_decisions',
        'treasury_execution_intents',
        'treasury_execution_receipts'
      ])
      and relation.relrowsecurity
  ),
  13,
  'RLS is enabled on every operations table'
);

select is(
  (
    select count(*)::integer
    from pg_catalog.pg_policies
    where schemaname = 'public'
      and tablename = any(array[
        'community_tasks',
        'task_submissions',
        'risk_reports',
        'risk_evidence',
        'risk_publications',
        'relief_applications',
        'relief_public_updates',
        'governance_proposals',
        'governance_discussions',
        'governance_discussion_publications',
        'governance_decisions',
        'treasury_execution_intents',
        'treasury_execution_receipts'
      ])
  ),
  37,
  'the reviewed operations policy total is unchanged'
);

select is(
  (
    select count(*)::integer
    from pg_catalog.pg_trigger trigger
    join pg_catalog.pg_class relation on relation.oid = trigger.tgrelid
    join pg_catalog.pg_namespace namespace on namespace.oid = relation.relnamespace
    where not trigger.tgisinternal
      and namespace.nspname = 'public'
      and relation.relname = any(array[
        'community_tasks',
        'task_submissions',
        'risk_reports',
        'risk_evidence',
        'risk_publications',
        'relief_applications',
        'relief_public_updates',
        'governance_proposals',
        'governance_discussions',
        'governance_discussion_publications',
        'governance_decisions',
        'treasury_execution_intents',
        'treasury_execution_receipts'
      ])
  ),
  29,
  'the reviewed non-internal trigger total is unchanged'
);

select ok(
  exists (
    select 1
    from pg_catalog.pg_policies
    where schemaname = 'public'
      and tablename = 'governance_discussions'
      and policyname = 'governance_discussions_moderator_read'
      and cmd = 'SELECT'
      and roles @> array['authenticated']::name[]
      and qual like '%moderator%'
  ),
  'moderators have an authenticated SELECT policy for private discussions'
);

select ok(
  not exists (
    select 1
    from information_schema.role_table_grants
    where table_schema = 'public'
      and table_name = any(array[
        'task_submissions',
        'risk_reports',
        'risk_evidence',
        'relief_applications',
        'governance_discussions',
        'treasury_execution_intents'
      ])
      and grantee = 'anon'
  ),
  'anon has no table grant on private intake'
);

select ok(
  (
    select pg_get_functiondef(procedure.oid)
    from pg_catalog.pg_proc procedure
    join pg_catalog.pg_namespace namespace
      on namespace.oid = procedure.pronamespace
    where namespace.nspname = 'public'
      and procedure.proname = 'protect_published_operations_record'
  ) ilike '%new.publication_status is distinct from old.publication_status%',
  'published protection explicitly rejects publication status downgrade'
);

select ok(
  (
    select pg_get_functiondef(procedure.oid)
    from pg_catalog.pg_proc procedure
    join pg_catalog.pg_namespace namespace
      on namespace.oid = procedure.pronamespace
    where namespace.nspname = 'public'
      and procedure.proname = 'protect_published_operations_record'
  ) like '%cannot be unpublished%',
  'published protection retains the reviewed failure boundary'
);

select ok(
  not exists (
    select 1
    from pg_catalog.pg_proc procedure
    join pg_catalog.pg_namespace namespace
      on namespace.oid = procedure.pronamespace
    where namespace.nspname = 'public'
      and (
        procedure.proname ilike '%send%transaction%'
        or procedure.proname ilike '%http%'
      )
  ),
  'operations schema contains no transaction sender or HTTP function'
);

select * from finish();

rollback;
