begin;

-- The staging E2E uses a service-role client only to remove the exact rows it
-- created. BYPASSRLS does not replace PostgreSQL table privileges, so cleanup
-- also needs SELECT for the id filter/returned id and DELETE for removal.
--
-- Keep browser-facing roles unable to delete these records.
revoke delete on table
  public.task_submissions,
  public.governance_discussions,
  public.community_tasks
from anon, authenticated;

grant usage on schema public to service_role;

grant select, delete on table
  public.task_submissions,
  public.governance_discussions,
  public.community_tasks
to service_role;

commit;
