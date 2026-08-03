-- Run with `supabase test db` against a disposable local or staging-linked
-- database after all reviewed operations migrations have been applied.
--
-- pgTAP is diagnostic only. This file creates no protocol integration and
-- sends no network or Solana transaction.

begin;

create schema if not exists extensions;
create extension if not exists pgtap with schema extensions;
set local search_path = public, extensions;

select no_plan();

select has_table('public'::name, 'operations_intake_control'::name);
select has_table('public'::name, 'operations_intake_gate_events'::name);
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
select has_table('public'::name, 'task_submission_publications'::name);
select has_table('public'::name, 'operations_task_workflow_events'::name);

select is(
  (
    select count(*)::integer
    from pg_catalog.pg_tables
    where schemaname = 'public'
      and tablename = any(array[
        'operations_intake_control',
        'operations_intake_gate_events',
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
        'treasury_execution_receipts',
        'task_submission_publications',
        'operations_task_workflow_events'
      ])
  ),
  17,
  'all 17 operations tables exist, including task result and workflow audit separation'
);

select is(
  (
    select count(*)::integer
    from pg_catalog.pg_class relation
    join pg_catalog.pg_namespace namespace on namespace.oid = relation.relnamespace
    where namespace.nspname = 'public'
      and relation.relname = any(array[
        'operations_intake_control',
        'operations_intake_gate_events',
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
        'treasury_execution_receipts',
        'task_submission_publications',
        'operations_task_workflow_events'
      ])
      and relation.relrowsecurity
  ),
  17,
  'RLS is enabled on every operations table'
);

select is(
  (
    select count(*)::integer
    from pg_catalog.pg_policies
    where schemaname = 'public'
      and tablename = any(array[
        'operations_intake_control',
        'operations_intake_gate_events',
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
        'treasury_execution_receipts',
        'task_submission_publications',
        'operations_task_workflow_events'
      ])
  ),
  39,
  'the reviewed operations policy total includes separated task publication and audit reads'
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
        'operations_intake_control',
        'operations_intake_gate_events',
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
        'treasury_execution_receipts',
        'task_submission_publications',
        'operations_task_workflow_events'
      ])
  ),
  37,
  'the reviewed trigger total includes immutable task publications and task workflow events'
);

select ok(
  (
    select mode = 'disabled' and activation_reference is null
    from public.operations_intake_control
    where singleton
  )
    and has_function_privilege(
      'authenticated',
      'public.is_operations_wallet_intake_enabled()',
      'EXECUTE'
    )
    and not has_function_privilege(
      'anon',
      'public.is_operations_wallet_intake_enabled()',
      'EXECUTE'
    )
    and not has_table_privilege(
      'authenticated',
      'public.operations_intake_control',
      'SELECT'
    ),
  'wallet intake is database-disabled by default and its control row is not browser-readable'
);

select ok(
  has_function_privilege(
    'service_role',
    'public.set_operations_wallet_intake_mode(text,text)',
    'EXECUTE'
  )
    and not has_function_privilege(
      'authenticated',
      'public.set_operations_wallet_intake_mode(text,text)',
      'EXECUTE'
    )
    and has_table_privilege(
      'service_role',
      'public.operations_intake_control',
      'SELECT'
    )
    and not has_table_privilege(
      'service_role',
      'public.operations_intake_control',
      'UPDATE'
    )
    and has_table_privilege(
      'service_role',
      'public.operations_intake_gate_events',
      'SELECT'
    )
    and not has_table_privilege(
      'service_role',
      'public.operations_intake_gate_events',
      'INSERT,UPDATE,DELETE'
    )
    and not has_table_privilege(
      'authenticated',
      'public.operations_intake_gate_events',
      'SELECT'
    ),
  'only service-role tooling can inspect and invoke the audited gate transition RPC'
);

select ok(
  exists (
    select 1
    from pg_catalog.pg_trigger trigger
    join pg_catalog.pg_class relation on relation.oid = trigger.tgrelid
    join pg_catalog.pg_namespace namespace on namespace.oid = relation.relnamespace
    where not trigger.tgisinternal
      and namespace.nspname = 'public'
      and relation.relname = 'operations_intake_gate_events'
      and trigger.tgname = 'operations_intake_gate_events_immutable'
  ),
  'intake gate audit events are immutable after insertion'
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
  (
    select count(*) = 4
    from pg_catalog.pg_policies
    where schemaname = 'public'
      and policyname = any(array[
        'task_submissions_owner_insert',
        'risk_reports_owner_insert',
        'relief_applications_owner_insert',
        'governance_discussions_owner_insert'
      ])
      and with_check like '%current_verified_solana_wallet%'
  ),
  'all direct owner intake policies bind wallet_address to the verified Web3 identity'
);

select ok(
  has_function_privilege(
    'authenticated',
    'public.current_verified_solana_wallet()',
    'EXECUTE'
  )
    and not has_function_privilege(
      'anon',
      'public.current_verified_solana_wallet()',
      'EXECUTE'
    ),
  'only authenticated browser sessions can resolve their verified Solana wallet'
);

select ok(
  (
    select
      pg_get_functiondef(procedure.oid) ilike '%identity.provider_id%'
      and pg_get_functiondef(procedure.oid)
        ilike '%identity.identity_data ->> ''sub''%'
      and pg_get_functiondef(procedure.oid)
        ilike '%provider_identifier is distinct from identity_subject%'
      and pg_get_functiondef(procedure.oid) ilike '%web3:solana:%'
      and pg_get_functiondef(procedure.oid)
        ilike '%leading_zero_bytes + non_zero_bytes <> 32%'
      and pg_get_functiondef(procedure.oid)
        not ilike '%identity_data ->> ''chain''%'
      and pg_get_functiondef(procedure.oid)
        not ilike '%character_index integer%'
    from pg_catalog.pg_proc procedure
    join pg_catalog.pg_namespace namespace
      on namespace.oid = procedure.pronamespace
    where namespace.nspname = 'public'
      and procedure.proname = 'current_verified_solana_wallet'
  ),
  'wallet resolver uses the observed identity contract without a shadowed loop declaration'
);

select ok(
  (
    select count(*) = 4
    from pg_catalog.pg_trigger trigger
    join pg_catalog.pg_class relation on relation.oid = trigger.tgrelid
    join pg_catalog.pg_namespace namespace on namespace.oid = relation.relnamespace
    where not trigger.tgisinternal
      and namespace.nspname = 'public'
      and trigger.tgname = any(array[
        'task_submissions_rate_limit',
        'risk_reports_rate_limit',
        'relief_applications_rate_limit',
        'governance_discussions_rate_limit'
      ])
  ),
  'all four direct intake tables enforce the database submission throttle'
);

select ok(
  not exists (
    select 1
    from information_schema.role_table_grants
    where table_schema = 'public'
      and table_name = any(array[
        'task_submissions',
        'operations_intake_gate_events',
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
  has_table_privilege(
    'anon',
    'public.task_submission_publications',
    'SELECT'
  )
    and has_table_privilege(
      'authenticated',
      'public.task_submission_publications',
      'SELECT'
    )
    and not exists (
      select 1
      from information_schema.columns
      where table_schema = 'public'
        and table_name = 'task_submission_publications'
        and column_name = any(array[
          'submission_id',
          'submitted_by',
          'reviewed_by',
          'actor_id'
        ])
    ),
  'public task results are readable and exclude private submission or Auth identifiers'
);

select ok(
  not has_table_privilege(
    'anon',
    'public.operations_task_workflow_events',
    'SELECT'
  )
    and has_table_privilege(
      'authenticated',
      'public.operations_task_workflow_events',
      'SELECT'
    )
    and exists (
      select 1
      from pg_catalog.pg_trigger trigger
      join pg_catalog.pg_class relation on relation.oid = trigger.tgrelid
      join pg_catalog.pg_namespace namespace on namespace.oid = relation.relnamespace
      where not trigger.tgisinternal
        and namespace.nspname = 'public'
        and relation.relname = 'operations_task_workflow_events'
        and trigger.tgname = 'operations_task_workflow_events_immutable'
    ),
  'task workflow audit is private to staff policy scope and immutable'
);

select ok(
  has_function_privilege(
    'authenticated',
    'public.publish_community_task_v1(text,text,text,numeric,text,timestamp with time zone,text)',
    'EXECUTE'
  )
    and not has_function_privilege(
      'anon',
      'public.publish_community_task_v1(text,text,text,numeric,text,timestamp with time zone,text)',
      'EXECUTE'
    )
    and has_function_privilege(
      'authenticated',
      'public.review_task_submission_v1(uuid,text,text,text,text,text)',
      'EXECUTE'
    )
    and not has_function_privilege(
      'anon',
      'public.review_task_submission_v1(uuid,text,text,text,text,text)',
      'EXECUTE'
    )
    and not has_table_privilege(
      'authenticated',
      'public.community_tasks',
      'INSERT,UPDATE,DELETE'
    )
    and not has_table_privilege(
      'authenticated',
      'public.task_submissions',
      'UPDATE,DELETE'
    ),
  'authenticated staff must use the audited task workflow RPCs instead of direct writes'
);

select ok(
  (
    select pg_get_functiondef(procedure.oid)
    from pg_catalog.pg_proc procedure
    join pg_catalog.pg_namespace namespace
      on namespace.oid = procedure.pronamespace
    where namespace.nspname = 'public'
      and procedure.proname = 'publish_community_task_v1'
  ) ilike '%v_actor_role is null%'
    and (
      select pg_get_functiondef(procedure.oid)
      from pg_catalog.pg_proc procedure
      join pg_catalog.pg_namespace namespace
        on namespace.oid = procedure.pronamespace
      where namespace.nspname = 'public'
        and procedure.proname = 'review_task_submission_v1'
    ) ilike '%v_actor_role is null%'
    and (
      select pg_get_functiondef(procedure.oid)
      from pg_catalog.pg_proc procedure
      join pg_catalog.pg_namespace namespace
        on namespace.oid = procedure.pronamespace
      where namespace.nspname = 'public'
        and procedure.proname = 'review_task_submission_v1'
    ) ilike '%public_result_consent%'
    and (
      select pg_get_functiondef(procedure.oid)
      from pg_catalog.pg_proc procedure
      join pg_catalog.pg_namespace namespace
        on namespace.oid = procedure.pronamespace
      where namespace.nspname = 'public'
        and procedure.proname = 'review_task_submission_v1'
    ) ilike '%reviewers cannot review their own task submission%',
  'task workflow RPCs explicitly reject NULL roles, require publication consent, and deny self-review'
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
