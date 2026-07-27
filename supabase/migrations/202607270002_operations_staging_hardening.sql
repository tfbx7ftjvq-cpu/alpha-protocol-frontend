-- Alpha Protocol Phase 2E-6B-4G
-- Staging findings applied on top of the V1 off-chain operations foundation.
--
-- This migration closes two authorization/integrity gaps found while preparing
-- real-role staging validation:
--   1. moderators need SELECT visibility before they can review private
--      governance discussion intake;
--   2. an already-published task or proposal must never be rewritten by first
--      downgrading publication_status.

begin;

create or replace function public.protect_published_operations_record()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  if old.publication_status = 'published'
    and new.publication_status is distinct from old.publication_status
  then
    raise exception 'published record on % cannot be unpublished', tg_table_name;
  end if;

  if old.publication_status = 'published'
    and (to_jsonb(new) - array['status', 'updated_at'])
      is distinct from
      (to_jsonb(old) - array['status', 'updated_at'])
  then
    raise exception 'published fields on % cannot be changed', tg_table_name;
  end if;

  return new;
end;
$$;

revoke all on function public.protect_published_operations_record() from public;

create policy governance_discussions_moderator_read
on public.governance_discussions
for select
to authenticated
using (public.has_operations_role(array['moderator', 'operator', 'governance_admin']));

commit;
