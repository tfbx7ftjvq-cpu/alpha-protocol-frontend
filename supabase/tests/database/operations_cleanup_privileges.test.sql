-- Run after all operations migrations have been applied.
--
-- These assertions verify the narrow database privilege needed by the
-- self-cleaning staging E2E. They do not create users, mutate protocol data,
-- call a network endpoint, or send a Solana transaction.

begin;

create schema if not exists extensions;
create extension if not exists pgtap with schema extensions;
set local search_path = public, extensions;

select plan(2);

select ok(
  has_table_privilege('service_role', 'public.task_submissions', 'SELECT')
    and has_table_privilege('service_role', 'public.task_submissions', 'DELETE')
    and has_table_privilege('service_role', 'public.governance_discussions', 'SELECT')
    and has_table_privilege('service_role', 'public.governance_discussions', 'DELETE')
    and has_table_privilege('service_role', 'public.community_tasks', 'SELECT')
    and has_table_privilege('service_role', 'public.community_tasks', 'DELETE'),
  'service_role has the exact row cleanup privileges required by staging E2E'
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

select * from finish();

rollback;
