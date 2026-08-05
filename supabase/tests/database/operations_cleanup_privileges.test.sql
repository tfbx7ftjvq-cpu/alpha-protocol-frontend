-- Run after all operations migrations have been applied.
--
-- These assertions verify the narrow database privilege needed by the
-- self-cleaning staging E2E. They do not create users, mutate protocol data,
-- call a network endpoint, or send a Solana transaction.

begin;

create schema if not exists extensions;
create extension if not exists pgtap with schema extensions;
set local search_path = public, extensions;

select plan(4);

select ok(
  has_table_privilege('service_role', 'public.task_submissions', 'SELECT')
    and has_table_privilege('service_role', 'public.governance_discussions', 'SELECT')
    and has_table_privilege('service_role', 'public.governance_discussions', 'DELETE')
    and has_table_privilege('service_role', 'public.community_tasks', 'SELECT')
    and not has_table_privilege('service_role', 'public.task_submissions', 'DELETE')
    and not has_table_privilege('service_role', 'public.community_tasks', 'DELETE'),
  'service_role keeps only the legacy discussion delete and task workflow read scope'
);

select ok(
  not has_table_privilege('anon', 'public.task_submissions', 'DELETE')
    and not has_table_privilege('anon', 'public.governance_discussions', 'DELETE')
    and not has_table_privilege('anon', 'public.community_tasks', 'DELETE')
    and not has_table_privilege('authenticated', 'public.task_submissions', 'DELETE')
    and not has_table_privilege('authenticated', 'public.governance_discussions', 'DELETE')
    and not has_table_privilege('authenticated', 'public.community_tasks', 'DELETE'),
  'anon and authenticated cannot use the staging E2E cleanup privilege'
);

select ok(
  has_function_privilege(
    'service_role',
    'public.cleanup_operations_task_staging_e2e_v1(text,uuid,uuid[])',
    'EXECUTE'
  )
    and not has_function_privilege(
      'authenticated',
      'public.cleanup_operations_task_staging_e2e_v1(text,uuid,uuid[])',
      'EXECUTE'
    )
    and not has_function_privilege(
      'anon',
      'public.cleanup_operations_task_staging_e2e_v1(text,uuid,uuid[])',
      'EXECUTE'
    ),
  'Phase 4M task workflow cleanup is exposed only through the service-role RPC'
);

select ok(
  has_function_privilege(
    'service_role',
    'public.cleanup_operations_relief_staging_e2e_v1(text,uuid[])',
    'EXECUTE'
  )
    and not has_function_privilege(
      'authenticated',
      'public.cleanup_operations_relief_staging_e2e_v1(text,uuid[])',
      'EXECUTE'
    )
    and not has_function_privilege(
      'anon',
      'public.cleanup_operations_relief_staging_e2e_v1(text,uuid[])',
      'EXECUTE'
    ),
  'Phase 4O relief workflow cleanup is exposed only through the service-role RPC'
);

select * from finish();

rollback;
