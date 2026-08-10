create or replace function public.is_base58_bytes_v1(value text, expected_bytes integer)
returns boolean
language plpgsql
immutable
strict
set search_path = public, pg_temp
as $$
declare
  alphabet constant text := '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
  numeric_value numeric := 0;
  leading_zero_bytes integer := 0;
  encoded_bytes integer := 0;
  character_value text;
begin
  if expected_bytes <= 0 or value = '' then return false; end if;
  for index_value in 1..char_length(value) loop
    character_value := substr(value, index_value, 1);
    if strpos(alphabet, character_value) = 0 then return false; end if;
    if character_value = '1' and numeric_value = 0 and encoded_bytes = 0 then
      leading_zero_bytes := leading_zero_bytes + 1;
    end if;
    numeric_value := numeric_value * 58 + strpos(alphabet, character_value) - 1;
  end loop;
  while numeric_value > 0 loop
    encoded_bytes := encoded_bytes + 1;
    numeric_value := trunc(numeric_value / 256);
  end loop;
  return leading_zero_bytes + encoded_bytes = expected_bytes;
end;
$$;
revoke all on function public.is_base58_bytes_v1(text, integer) from public;
