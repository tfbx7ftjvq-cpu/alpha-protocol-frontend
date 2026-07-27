import { randomBytes } from 'node:crypto';
import {
  createClient,
  type SupabaseClient,
  type User,
} from '@supabase/supabase-js';
import {
  assertStagingSecretIsolation,
  isMainModule,
  loadOperationsStagingEnvironment,
  resolveOperationsStagingConfig,
  sanitizeStagingError,
  type OperationsStagingConfig,
} from './common.ts';
import { runOperationsStagingPreflight } from './preflight.ts';

type OperationsRole = 'operator' | 'moderator';

interface TestActor {
  user: User;
  client: SupabaseClient;
}

interface CreatedRows {
  taskId: string | null;
  submissionId: string | null;
  discussionId: string | null;
}

export interface OperationsStagingE2EResult {
  projectRef: string;
  assertions: number;
  usersDeleted: number;
  rowsDeleted: number;
}

export async function runOperationsStagingE2E(
  config: OperationsStagingConfig,
): Promise<OperationsStagingE2EResult> {
  if (
    config.mode !== 'e2e'
    || !config.serviceRoleKey
    || !config.confirmedForWrites
  ) {
    throw new Error('staging E2E 配置未满足写入确认与 service-role 要求');
  }

  await runOperationsStagingPreflight({ ...config, mode: 'preflight' });

  const admin = createClient(config.supabaseUrl, config.serviceRoleKey, {
    auth: {
      autoRefreshToken: false,
      detectSessionInUrl: false,
      persistSession: false,
    },
  });
  const publicClient = createClient(config.supabaseUrl, config.publicKey, {
    auth: {
      autoRefreshToken: false,
      detectSessionInUrl: false,
      persistSession: false,
    },
  });

  const runId = `${Date.now()}-${randomBytes(4).toString('hex')}`;
  const createdUsers: string[] = [];
  const rows: CreatedRows = {
    taskId: null,
    submissionId: null,
    discussionId: null,
  };
  let assertions = 0;
  let primaryError: unknown = null;

  try {
    const operator = await createActor(
      admin,
      config,
      runId,
      'operator',
      'operator',
    );
    createdUsers.push(operator.user.id);

    const moderator = await createActor(
      admin,
      config,
      runId,
      'moderator',
      'moderator',
    );
    createdUsers.push(moderator.user.id);

    const ownerA = await createActor(admin, config, runId, 'owner-a');
    createdUsers.push(ownerA.user.id);

    const ownerB = await createActor(admin, config, runId, 'owner-b');
    createdUsers.push(ownerB.user.id);

    const taskInsert = await operator.client
      .from('community_tasks')
      .insert({
        title: `Staging RLS task ${runId}`,
        summary: 'Temporary staging-only task used to verify public read and owner isolation.',
        requirements: 'Submit a staging-only HTTPS reference and never use production credentials.',
        reward_budget_usdc: 0,
        reward_source: 'none',
        status: 'open',
        publication_status: 'published',
        published_at: new Date().toISOString(),
      })
      .select('id')
      .single();
    assertNoError(taskInsert.error, 'operator task insert');
    rows.taskId = requiredId(taskInsert.data?.id, 'task');
    assertions += 1;

    const publicRead = await publicClient
      .from('community_tasks')
      .select('id')
      .eq('id', rows.taskId)
      .single();
    assertNoError(publicRead.error, 'anon public task read');
    expect(publicRead.data?.id === rows.taskId, 'anon did not read the published task');
    assertions += 1;

    const privateAnonRead = await publicClient
      .from('task_submissions')
      .select('id')
      .limit(1);
    expect(
      Boolean(privateAnonRead.error),
      'anon unexpectedly received SELECT access to task_submissions',
    );
    assertions += 1;

    const submissionInsert = await ownerA.client
      .from('task_submissions')
      .insert({
        task_id: rows.taskId,
        submitted_by: ownerA.user.id,
        summary: 'Staging-only contribution result used for owner isolation validation.',
        deliverable_url: 'https://example.com/alpha-staging-e2e',
        wallet_address: '11111111111111111111111111111111',
        wallet_verified: false,
        status: 'submitted',
      })
      .select('id')
      .single();
    assertNoError(submissionInsert.error, 'owner task submission insert');
    rows.submissionId = requiredId(submissionInsert.data?.id, 'submission');
    assertions += 1;

    const ownerRead = await ownerA.client
      .from('task_submissions')
      .select('id')
      .eq('id', rows.submissionId)
      .single();
    assertNoError(ownerRead.error, 'owner task submission read');
    assertions += 1;

    const crossUserRead = await ownerB.client
      .from('task_submissions')
      .select('id')
      .eq('id', rows.submissionId);
    assertNoError(crossUserRead.error, 'cross-user isolation query');
    expect(
      (crossUserRead.data ?? []).length === 0,
      'a second user read another owner private submission',
    );
    assertions += 1;

    const forgedWalletVerification = await ownerA.client
      .from('task_submissions')
      .insert({
        task_id: rows.taskId,
        submitted_by: ownerA.user.id,
        summary: 'This row must be rejected because intake cannot verify its own wallet.',
        deliverable_url: 'https://example.com/alpha-staging-forged-wallet',
        wallet_address: '11111111111111111111111111111111',
        wallet_verified: true,
        status: 'submitted',
      });
    expect(
      Boolean(forgedWalletVerification.error),
      'owner forged wallet_verified=true',
    );
    assertions += 1;

    const discussionInsert = await ownerA.client
      .from('governance_discussions')
      .insert({
        submitted_by: ownerA.user.id,
        topic: `Staging moderation ${runId}`,
        body: 'This private staging discussion verifies moderator SELECT and UPDATE policies.',
        wallet_verified: false,
        moderation_status: 'pending',
      })
      .select('id')
      .single();
    assertNoError(discussionInsert.error, 'discussion insert');
    rows.discussionId = requiredId(discussionInsert.data?.id, 'discussion');
    assertions += 1;

    const moderatorRead = await moderator.client
      .from('governance_discussions')
      .select('id,moderation_status')
      .eq('id', rows.discussionId)
      .single();
    assertNoError(moderatorRead.error, 'moderator discussion read');
    expect(
      moderatorRead.data?.moderation_status === 'pending',
      'moderator did not receive pending discussion',
    );
    assertions += 1;

    const unauthorizedModeration = await ownerB.client
      .from('governance_discussions')
      .update({
        moderation_status: 'rejected',
        moderated_by: ownerB.user.id,
      })
      .eq('id', rows.discussionId)
      .select('id');
    expect(
      Boolean(unauthorizedModeration.error)
        || (unauthorizedModeration.data ?? []).length === 0,
      'unprivileged user moderated another user discussion',
    );
    assertions += 1;

    const authorizedModeration = await moderator.client
      .from('governance_discussions')
      .update({
        moderation_status: 'published',
        moderated_by: moderator.user.id,
      })
      .eq('id', rows.discussionId)
      .select('id,moderation_status')
      .single();
    assertNoError(authorizedModeration.error, 'moderator discussion update');
    expect(
      authorizedModeration.data?.moderation_status === 'published',
      'moderator update did not persist',
    );
    assertions += 1;

    const closeTask = await operator.client
      .from('community_tasks')
      .update({ status: 'closed' })
      .eq('id', rows.taskId)
      .select('id,status')
      .single();
    assertNoError(closeTask.error, 'published task status transition');
    expect(closeTask.data?.status === 'closed', 'published task did not close');
    assertions += 1;

    const downgrade = await operator.client
      .from('community_tasks')
      .update({ publication_status: 'draft' })
      .eq('id', rows.taskId)
      .select('id');
    expect(
      Boolean(downgrade.error),
      'published task was downgraded to draft',
    );
    assertions += 1;

    const rewrite = await operator.client
      .from('community_tasks')
      .update({ title: `Rewritten ${runId}` })
      .eq('id', rows.taskId)
      .select('id');
    expect(
      Boolean(rewrite.error),
      'published task content was rewritten',
    );
    assertions += 1;
  } catch (error) {
    primaryError = error;
  }

  const cleanup = await cleanupStagingFixtures(admin, rows, createdUsers);
  if (primaryError) {
    throw primaryError;
  }
  if (cleanup.errors.length > 0) {
    throw new Error(`staging E2E cleanup incomplete: ${cleanup.errors.join('; ')}`);
  }

  return {
    projectRef: config.projectRef,
    assertions,
    usersDeleted: cleanup.usersDeleted,
    rowsDeleted: cleanup.rowsDeleted,
  };
}

async function createActor(
  admin: SupabaseClient,
  config: OperationsStagingConfig,
  runId: string,
  label: string,
  operationsRole?: OperationsRole,
): Promise<TestActor> {
  const email = `alpha-operations-${runId}-${label}@example.com`;
  const password = `Aa1!${randomBytes(24).toString('base64url')}`;
  const createResult = await admin.auth.admin.createUser({
    email,
    password,
    email_confirm: true,
    app_metadata: operationsRole
      ? { operations_role: operationsRole }
      : {},
  });
  assertNoError(createResult.error, `create ${label} test user`);
  if (!createResult.data.user) {
    throw new Error(`create ${label} test user returned no user`);
  }

  try {
    const client = createClient(config.supabaseUrl, config.publicKey, {
      auth: {
        autoRefreshToken: false,
        detectSessionInUrl: false,
        persistSession: false,
      },
    });
    const signInResult = await client.auth.signInWithPassword({ email, password });
    assertNoError(signInResult.error, `sign in ${label} test user`);
    if (!signInResult.data.user) {
      throw new Error(`sign in ${label} test user returned no user`);
    }

    return { user: signInResult.data.user, client };
  } catch (error) {
    await admin.auth.admin.deleteUser(createResult.data.user.id);
    throw error;
  }
}

async function cleanupStagingFixtures(
  admin: SupabaseClient,
  rows: CreatedRows,
  userIds: string[],
): Promise<{
  errors: string[];
  rowsDeleted: number;
  usersDeleted: number;
}> {
  const errors: string[] = [];
  let rowsDeleted = 0;
  let usersDeleted = 0;

  for (const [table, id] of [
    ['task_submissions', rows.submissionId],
    ['governance_discussions', rows.discussionId],
    ['community_tasks', rows.taskId],
  ] as const) {
    if (!id) {
      continue;
    }

    const result = await admin.from(table).delete().eq('id', id);
    if (result.error) {
      errors.push(`${table} cleanup failed`);
    } else {
      rowsDeleted += 1;
    }
  }

  for (const userId of [...userIds].reverse()) {
    const result = await admin.auth.admin.deleteUser(userId);
    if (result.error) {
      errors.push('test user cleanup failed');
    } else {
      usersDeleted += 1;
    }
  }

  return { errors, rowsDeleted, usersDeleted };
}

function assertNoError(
  error: { message: string } | null,
  label: string,
): asserts error is null {
  if (error) {
    throw new Error(`${label} failed: ${error.message}`);
  }
}

function requiredId(value: unknown, label: string): string {
  if (typeof value !== 'string' || !value) {
    throw new Error(`${label} insert returned no id`);
  }
  return value;
}

function expect(condition: boolean, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

async function main(): Promise<void> {
  const loaded = loadOperationsStagingEnvironment();
  assertStagingSecretIsolation(loaded.projectDirectory, loaded.envFile);
  const config = resolveOperationsStagingConfig(loaded.env, 'e2e');

  try {
    const result = await runOperationsStagingE2E(config);
    console.log('Operations staging RLS E2E passed.');
    console.log(`Project ref: ${result.projectRef}`);
    console.log(`Assertions: ${result.assertions}`);
    console.log(`Cleanup: ${result.rowsDeleted} rows, ${result.usersDeleted} users`);
    console.log('No Solana transaction or treasury action was performed.');
  } catch (error) {
    throw new Error(sanitizeStagingError(error, config));
  }
}

if (isMainModule(import.meta.url)) {
  main().catch((error: unknown) => {
    console.error(`Operations staging E2E failed: ${sanitizeStagingError(error)}`);
    process.exitCode = 1;
  });
}
