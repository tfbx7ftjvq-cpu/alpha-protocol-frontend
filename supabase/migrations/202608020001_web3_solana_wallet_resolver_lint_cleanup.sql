-- Alpha Protocol Phase 2E-6B-4K operational hardening.
-- Remove the redundant PL/pgSQL integer-loop declaration reported by the
-- linked Supabase schema linter after migration 202607310001 was applied.
--
-- PostgreSQL creates the integer FOR-loop variable automatically. Declaring
-- the same name in the function DECLARE block caused shadowed/unused-variable
-- warnings but did not change resolver behavior. This migration preserves the
-- reviewed fail-closed identity checks and does not activate intake.

begin;

create or replace function public.current_verified_solana_wallet()
returns text
language plpgsql
stable
security definer
set search_path = ''
as $$
declare
  alphabet constant text :=
    '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
  identity_prefix constant text := 'web3:solana:';
  web3_identity_count integer;
  provider_identifier text;
  identity_subject text;
  wallet_address text;
  numeric_value numeric := 0;
  digit integer;
  leading_zero_bytes integer := 0;
  non_zero_bytes integer := 0;
begin
  select
    count(*)::integer,
    min(identity.provider_id),
    min(identity.identity_data ->> 'sub')
  into
    web3_identity_count,
    provider_identifier,
    identity_subject
  from auth.identities identity
  where identity.user_id = (select auth.uid())
    and identity.provider = 'web3';

  if web3_identity_count <> 1 then
    return null;
  end if;

  if provider_identifier is null
    or identity_subject is null
    or provider_identifier is distinct from identity_subject
  then
    return null;
  end if;

  if left(provider_identifier, char_length(identity_prefix)) <> identity_prefix then
    return null;
  end if;

  wallet_address := substr(
    provider_identifier,
    char_length(identity_prefix) + 1
  );

  if char_length(wallet_address) not between 32 and 44
    or wallet_address !~ '^[1-9A-HJ-NP-Za-km-z]+$'
  then
    return null;
  end if;

  for character_index in 1..char_length(wallet_address) loop
    digit := strpos(alphabet, substr(wallet_address, character_index, 1)) - 1;
    if digit < 0 then
      return null;
    end if;
    numeric_value := numeric_value * 58 + digit;
  end loop;

  while leading_zero_bytes < char_length(wallet_address)
    and substr(wallet_address, leading_zero_bytes + 1, 1) = '1'
  loop
    leading_zero_bytes := leading_zero_bytes + 1;
  end loop;

  while numeric_value > 0 loop
    numeric_value := trunc(numeric_value / 256);
    non_zero_bytes := non_zero_bytes + 1;
  end loop;

  if leading_zero_bytes + non_zero_bytes <> 32 then
    return null;
  end if;

  return wallet_address;
end;
$$;

comment on function public.current_verified_solana_wallet() is
  'Returns exactly one Solana wallet from matching Supabase Web3 provider_id and identity_data.sub values; otherwise NULL.';

revoke all on function public.current_verified_solana_wallet() from public;
revoke all on function public.current_verified_solana_wallet() from anon;
grant execute on function public.current_verified_solana_wallet() to authenticated;

commit;
