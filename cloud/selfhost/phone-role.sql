-- THE SELF-HOSTED CLOUD'S ROLE IN THE DATABASE (docs/cloud/SELF-HOSTED.md).
--
-- The server carries one person's phone replies with the computer shut
-- (cloud/gateway/phone.mjs) as this role, not as the service role. Its token
-- names that person (`sub`), so the phone tables' own rules give it their
-- rows and nobody else's. It may do what the carry does, which is one thing
-- more than a signed-in person: say who claimed a command.
--
-- A signed token cannot be taken back on its own, so each one carries an id
-- (`jti`) and works only while that id is in phone_carry_tokens: deleting the
-- row stops it at once.
--
-- Run as a superuser (supabase_admin): Storage's own role has to take this
-- one too. Written to be run twice without harm.
do $$
begin
  if not exists (select 1 from pg_roles where rolname = 'phone_carry') then
    create role phone_carry nologin;
  end if;
end
$$;
grant phone_carry to authenticator;

create table if not exists public.phone_carry_tokens (
  id text primary key check (id ~ '^[0-9a-f]{32}$'),
  made_at timestamptz not null default now()
);
alter table public.phone_carry_tokens enable row level security;
revoke all on public.phone_carry_tokens from public, anon, authenticated, phone_carry;
grant all on public.phone_carry_tokens to service_role;

create or replace function public.phone_carry_token_lives() returns boolean
language sql stable security definer set search_path = public as $$
  select exists (select 1 from public.phone_carry_tokens where id = auth.jwt() ->> 'jti')
$$;
revoke all on function public.phone_carry_token_lives() from public, anon, authenticated;
grant execute on function public.phone_carry_token_lives() to phone_carry;

grant usage on schema public to phone_carry;
grant select on public.phone_commands to phone_carry;
grant update (claimed_at, claimed_by, taken_at, outcome) on public.phone_commands to phone_carry;
grant select, insert, update on public.phone_threads to phone_carry;
grant select, delete on public.push_devices to phone_carry;

drop policy if exists phone_carry_token_lives on public.phone_commands;
create policy phone_carry_token_lives on public.phone_commands as restrictive for all to phone_carry
  using (public.phone_carry_token_lives()) with check (public.phone_carry_token_lives());
drop policy if exists phone_carry_token_lives on public.phone_threads;
create policy phone_carry_token_lives on public.phone_threads as restrictive for all to phone_carry
  using (public.phone_carry_token_lives()) with check (public.phone_carry_token_lives());
drop policy if exists phone_carry_token_lives on public.push_devices;
create policy phone_carry_token_lives on public.push_devices as restrictive for all to phone_carry
  using (public.phone_carry_token_lives()) with check (public.phone_carry_token_lives());

-- The links a sandbox fetches the phone's files by.
do $$
begin
  if not exists (select 1 from information_schema.tables where table_schema = 'storage' and table_name = 'objects') then
    return;
  end if;
  grant phone_carry to supabase_storage_admin;
  grant usage on schema storage to phone_carry;
  grant select on storage.objects to phone_carry;
  drop policy if exists phone_files_carried on storage.objects;
  create policy phone_files_carried on storage.objects for select to phone_carry
    using (bucket_id = 'phone-files' and (storage.foldername(name))[1] = auth.uid()::text and public.phone_carry_token_lives());
end
$$;
