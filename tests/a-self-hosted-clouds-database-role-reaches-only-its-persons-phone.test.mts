// A SELF-HOSTED CLOUD'S DATABASE ROLE REACHES ONLY ITS PERSON'S PHONE.
//
// The managed cloud carries a phone's reply with the computer shut as the
// service role, which can read and write every table in the database. A
// self-hosted server sits on a machine that runs agents, so it gets a role of
// its own (cloud/selfhost/phone-role.sql) whose token names one person. Run
// here after every migration in PGlite, in `public` and in a schema of its
// own as a self-hosted Supabase shared by several apps keeps it, and tried
// door by door: each thing the carry does (cloud/gateway/phone.mjs
// supabaseRest) is allowed for that person's rows, and nothing else is. A
// signed token cannot be taken back, so it works only while its id is listed
// in the database; taken off the list, it is shut out at once.
import { it, expect, beforeAll, afterAll, describe } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { PGlite } from '@electric-sql/pglite';
import { emptyDatabase } from './an-empty-test-database.mts';
import { spawnSync } from 'node:child_process';
import { inSchema } from '../scripts/lib/migrations-in-schema.mjs';
import { migrationScript } from '../scripts/apply-migrations-self-hosted.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const migrationsDir = path.join(root, 'cloud/supabase/migrations');
const migrations = fs.readdirSync(migrationsDir).filter((f) => f.endsWith('.sql')).sort()
  .map((f) => fs.readFileSync(path.join(migrationsDir, f), 'utf8'));
const ROLE_SQL = fs.readFileSync(path.join(root, 'cloud/selfhost/phone-role.sql'), 'utf8');

const SUPABASE_STAND_IN = `
create schema auth;
create table auth.users (id uuid primary key, email text, email_confirmed_at timestamptz, raw_user_meta_data jsonb default '{}'::jsonb);
create function auth.uid() returns uuid language sql stable as $$
  select nullif(current_setting('request.jwt.claim.sub', true), '')::uuid
$$;
create function auth.jwt() returns jsonb language sql stable as $$
  select coalesce(nullif(current_setting('request.jwt.claims', true), ''), '{}')::jsonb
$$;
create role authenticated;
create role anon;
create role service_role;
create role authenticator;
create role supabase_storage_admin;
create publication supabase_realtime;
create schema storage;
create table storage.buckets (id text primary key, name text not null, public boolean default false, file_size_limit bigint);
create table storage.objects (id uuid primary key default gen_random_uuid(), bucket_id text references storage.buckets(id), name text not null, owner uuid, unique (bucket_id, name));
create function storage.foldername(name text) returns text[] language sql immutable as $$
  select (string_to_array(name, '/'))[1:array_length(string_to_array(name, '/'), 1) - 1]
$$;
alter table storage.objects enable row level security;
grant usage on schema storage to authenticated, anon;
grant select, insert, update, delete on storage.objects to authenticated, anon;
insert into storage.buckets (id, name) values ('other-files', 'other-files');
`;

const OWNER = '11111111-1111-1111-1111-111111111111';
const STRANGER = '22222222-2222-2222-2222-222222222222';
const DEV = 'a1b2c3d4e5f6';
const SHA = 'a'.repeat(64);
const LIVE = '0123456789abcdef0123456789abcdef';
const TAKEN_BACK = 'fedcba9876543210fedcba9876543210';

describe.each(['public', 'mercury'])('in schema %s', (schema) => {
  const s = schema;
  let db: PGlite;

  async function as(person: string, sql: string, params: unknown[] = [], role = 'phone_carry', jti = LIVE) {
    const claims = JSON.stringify({ sub: person, role, jti });
    await db.exec(`reset role; select set_config('request.jwt.claim.sub', '${person}', false); select set_config('request.jwt.claims', '${claims}', false); set role ${role};`);
    try { return await db.query(sql, params); } finally { await db.exec('reset role;'); }
  }
  const rows = async (person: string, sql: string, params?: unknown[], role?: string, jti?: string) => (await as(person, sql, params, role, jti)).rows as Record<string, unknown>[];
  const refused = async (person: string, sql: string, params?: unknown[], role?: string, jti?: string) => {
    try { await as(person, sql, params, role, jti); return false; } catch { return true; }
  };
  const command = async (person: string) => (await rows(person,
    `insert into ${s}.phone_commands (device, project, item, kind, payload) values ($1, 'alpha', 'w-1', 'reply', '{"text":"yes"}') returning id`,
    [DEV], 'authenticated'))[0].id as string;

  beforeAll(async () => {
    db = new PGlite(await emptyDatabase());
    await db.exec(SUPABASE_STAND_IN);
    for (const m of migrations) await db.exec(inSchema(m, schema));
    await db.exec(inSchema(ROLE_SQL, schema));
    // Written to be run twice without harm, as every migration is.
    await db.exec(inSchema(ROLE_SQL, schema));
    await db.query(`insert into ${s}.phone_carry_tokens (id) values ($1)`, [LIVE]);
    for (const [id, email] of [[OWNER, 'owner@example.com'], [STRANGER, 'stranger@example.com']]) {
      await db.query(`insert into auth.users (id, email, email_confirmed_at) values ($1, $2, now())`, [id, email]);
      await as(id, `insert into ${s}.phone_projects (device, project, name) values ($1, 'alpha', 'Alpha')`, [DEV], 'authenticated');
      await as(id, `insert into ${s}.phone_threads (device, project, item, snapshot) values ($1, 'alpha', 'w-1', '{"v":3}')`, [DEV], 'authenticated');
      await as(id, `insert into ${s}.push_devices (token, platform) values ($1, 'ios')`, [`ExponentPushToken[${id.slice(0, 12)}]`], 'authenticated');
      await db.query(`insert into storage.objects (bucket_id, name, owner) values ('phone-files', $1, $2), ('other-files', $3, $2)`,
        [`${id}/${DEV}/${SHA}.png`, id, `${id}/x.png`]);
    }
  }, 120_000);
  afterAll(() => db.close());

  it('is taken by the API and by Storage, as the roles a token may name are', async () => {
    const member = async (of: string) => (await db.query(`select pg_has_role($1, 'phone_carry', 'member') as m`, [of])).rows[0] as { m: boolean };
    expect((await member('authenticator')).m).toBe(true);
    expect((await member('supabase_storage_admin')).m).toBe(true);
  });

  it('claims its person\'s reply for a cloud run, gives the claim back, and marks it taken', async () => {
    const id = await command(OWNER);
    const waiting = await rows(OWNER, `select id from ${s}.phone_commands where kind = 'reply' and taken_at is null and claimed_at is null`);
    expect(waiting.map((r) => r.id)).toContain(id);
    const claimed = await rows(OWNER, `update ${s}.phone_commands set claimed_at = now(), claimed_by = 'cloud:r-1' where id = $1 and claimed_at is null returning id`, [id]);
    expect(claimed).toHaveLength(1);
    await as(OWNER, `update ${s}.phone_commands set claimed_at = null, claimed_by = null where id = $1 and claimed_by = 'cloud:r-1'`, [id]);
    await as(OWNER, `update ${s}.phone_commands set claimed_at = now(), claimed_by = 'cloud:r-2' where id = $1`, [id]);
    await as(OWNER, `update ${s}.phone_commands set taken_at = now(), outcome = 'Answered' where id = $1`, [id]);
    const [done] = await rows(OWNER, `select claimed_by, outcome from ${s}.phone_commands where id = $1`, [id]);
    expect(done).toEqual({ claimed_by: 'cloud:r-2', outcome: 'Answered' });
  });

  it('cannot see, claim or mark another person\'s reply', async () => {
    const theirs = await command(STRANGER);
    expect(await rows(OWNER, `select id from ${s}.phone_commands where id = $1`, [theirs])).toEqual([]);
    expect(await rows(OWNER, `update ${s}.phone_commands set claimed_at = now(), claimed_by = 'cloud:r-3' where id = $1 returning id`, [theirs])).toEqual([]);
    const [still] = (await db.query(`select claimed_at from ${s}.phone_commands where id = $1`, [theirs])).rows as { claimed_at: unknown }[];
    expect(still.claimed_at).toBeNull();
  });

  it('cannot write a command, rewrite one, or delete one', async () => {
    const id = await command(OWNER);
    expect(await refused(OWNER, `insert into ${s}.phone_commands (device, project, item, kind, payload) values ($1, 'alpha', 'w-1', 'reply', '{}')`, [DEV])).toBe(true);
    expect(await refused(OWNER, `update ${s}.phone_commands set payload = '{"text":"no"}' where id = $1`, [id])).toBe(true);
    expect(await refused(OWNER, `delete from ${s}.phone_commands where id = $1`, [id])).toBe(true);
  });

  it('reads and writes its person\'s phone threads, and starts one there, but nobody else\'s', async () => {
    const mine = await rows(OWNER, `select owner from ${s}.phone_threads`);
    expect(mine.map((r) => r.owner)).toEqual([OWNER]);
    expect(await rows(OWNER, `update ${s}.phone_threads set snapshot = '{"v":4}' where owner = $1 returning item`, [OWNER])).toHaveLength(1);
    expect(await rows(OWNER, `update ${s}.phone_threads set snapshot = '{"v":4}' where owner = $1 returning item`, [STRANGER])).toEqual([]);
    await as(OWNER, `insert into ${s}.phone_threads (owner, device, project, item, snapshot) values ($1, $2, 'alpha', 'w-2', '{"v":3}')`, [OWNER, DEV]);
    expect(await refused(OWNER, `insert into ${s}.phone_threads (owner, device, project, item, snapshot) values ($1, $2, 'alpha', 'w-9', '{"v":3}')`, [STRANGER, DEV])).toBe(true);
    expect(await refused(OWNER, `delete from ${s}.phone_threads where item = 'w-2'`)).toBe(true);
  });

  it('reads and drops its person\'s alert devices, and adds none', async () => {
    expect((await rows(OWNER, `select owner from ${s}.push_devices`)).map((r) => r.owner)).toEqual([OWNER]);
    expect(await refused(OWNER, `insert into ${s}.push_devices (owner, token, platform) values ($1, 'ExponentPushToken[aaaaaaaaaaaa]', 'ios')`, [OWNER])).toBe(true);
    await as(OWNER, `delete from ${s}.push_devices where owner = $1`, [OWNER]);
    expect((await db.query(`select owner from ${s}.push_devices`)).rows).toEqual([{ owner: STRANGER }]);
  });

  it('sees its person\'s files from the phone, and no one else\'s, and no other bucket', async () => {
    const seen = await rows(OWNER, `select bucket_id, name from storage.objects order by name`);
    expect(seen).toEqual([{ bucket_id: 'phone-files', name: `${OWNER}/${DEV}/${SHA}.png` }]);
    expect(await refused(OWNER, `delete from storage.objects where bucket_id = 'phone-files'`)).toBe(true);
  });

  it('shuts out a token whose id is not listed, and one taken off the list, at once', async () => {
    const id = await command(OWNER);
    const nobody = async (jti: string) => {
      expect(await rows(OWNER, `select id from ${s}.phone_commands`, [], 'phone_carry', jti)).toEqual([]);
      expect(await rows(OWNER, `select item from ${s}.phone_threads`, [], 'phone_carry', jti)).toEqual([]);
      expect(await rows(OWNER, `select token from ${s}.push_devices`, [], 'phone_carry', jti)).toEqual([]);
      expect(await rows(OWNER, `select name from storage.objects`, [], 'phone_carry', jti)).toEqual([]);
      expect(await rows(OWNER, `update ${s}.phone_commands set claimed_at = now(), claimed_by = 'cloud:r-9' where id = $1 returning id`, [id], 'phone_carry', jti)).toEqual([]);
      expect(await refused(OWNER, `insert into ${s}.phone_threads (owner, device, project, item, snapshot) values ($1, $2, 'alpha', 'w-8', '{"v":3}')`, [OWNER, DEV], 'phone_carry', jti)).toBe(true);
    };
    await nobody(TAKEN_BACK);
    await db.query(`insert into ${s}.phone_carry_tokens (id) values ($1)`, [TAKEN_BACK]);
    expect(await rows(OWNER, `select id from ${s}.phone_commands where id = $1`, [id], 'phone_carry', TAKEN_BACK)).toHaveLength(1);
    await db.query(`delete from ${s}.phone_carry_tokens where id = $1`, [TAKEN_BACK]);
    await nobody(TAKEN_BACK);
  });

  it('keeps the list of tokens from everyone the API serves, the role itself too', async () => {
    for (const role of ['phone_carry', 'authenticated', 'anon']) {
      expect(await refused(OWNER, `select id from ${s}.phone_carry_tokens`, [], role), role).toBe(true);
      expect(await refused(OWNER, `insert into ${s}.phone_carry_tokens (id) values ('${'b'.repeat(32)}')`, [], role), role).toBe(true);
    }
  });

  it('opens no other table', async () => {
    for (const table of ['phone_projects', 'phone_settings', 'cloud_plans', 'team_members', 'people']) {
      expect(await refused(OWNER, `select 1 from ${s}.${table} limit 1`), table).toBe(true);
    }
  });
});

describe('the script a server at home is set up with', () => {
  const script = path.join(root, 'scripts/apply-migrations-self-hosted.mjs');
  const file = path.join(root, 'cloud/selfhost/phone-role.sql');
  const run = (...args: string[]) => spawnSync(process.execPath, [script, ...args], { encoding: 'utf8', env: { PATH: process.env.PATH } });

  it('prints the role for the schema it is told, without a database', () => {
    const out = run('--schema', 'mercury', '--print', file);
    expect(out.status).toBe(0);
    expect(out.stdout).toBe(migrationScript([file], 'mercury'));
    expect(out.stdout).toContain('on mercury.phone_commands to phone_carry');
  });

  it('refuses a schema that is not a plain name, or none', () => {
    expect(run('--schema', 'x; drop table people', '--print', file).status).not.toBe(0);
    expect(run('--print', file, '--schema').status).not.toBe(0);
  });
});
