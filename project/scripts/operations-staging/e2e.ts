import {
  createPrivateKey,
  randomBytes,
  sign as signWithPrivateKey,
} from 'node:crypto';
import { Keypair } from '@solana/web3.js';
import {
  createClient,
  type SupabaseClient,
  type User,
} from '@supabase/supabase-js';
import { OPERATIONS_WALLET_SIGN_IN_STATEMENT } from '../../src/features/operations/auth.ts';
import {
  assertStagingSecretIsolation,
  isMainModule,
  loadOperationsStagingEnvironment,
  resolveOperationsStagingConfig,
  sanitizeStagingError,
  type OperationsStagingConfig,
} from './common.ts';
import { runOperationsStagingPreflight } from './preflight.ts';

type OperationsRole = 'operator' | 'reviewer' | 'moderator';

interface TestActor {
  user: User;
  client: SupabaseClient;
  walletAddress: string | null;
}

interface CreatedRows {
  taskId: string | null;
  submissionIds: string[];
  publicationId: string | null;
  discussionId: string | null;
  runReference: string;
}

interface TaskWorkflowCleanupCounts {
  publicationsDeleted: number;
  eventsDeleted: number;
  submissionsDeleted: number;
  tasksDeleted: number;
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
    || !config.web3Url
    || !config.e2eCaptchaToken
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
  const runReference = `phase-2e-6b-4m-staging-e2e:${runId}`;
  const createdUsers: string[] = [];
  const rows: CreatedRows = {
    taskId: null,
    submissionIds: [],
    publicationId: null,
    discussionId: null,
    runReference,
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

    const intakeGate = await operator.client.rpc('is_operations_wallet_intake_enabled');
    assertNoError(intakeGate.error, 'wallet intake server gate read');
    expect(
      intakeGate.data === true,
      'wallet intake server gate is disabled',
    );
    assertions += 1;

    const reviewer = await createActor(
      admin,
      config,
      runId,
      'reviewer',
      'reviewer',
    );
    createdUsers.push(reviewer.user.id);

    const ownerA = await createWalletActor(admin, config);
    createdUsers.push(ownerA.user.id);

    const switchedWalletAddress = Keypair.generate().publicKey.toBase58();

    const emailOnlyOwner = await createActor(admin, config, runId, 'email-owner');
    createdUsers.push(emailOnlyOwner.user.id);

    const taskTitle = `Staging task workflow ${runId}`;
    const taskSummary =
      `Temporary Phase 4M Staging task for audited publication and review E2E ${runId}.`;
    const taskRequirements =
      `Submit only the two reserved example.com fixtures for this controlled Staging E2E run ${runId}.`;
    const submissionDeadline = new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString();

    const unauthorizedTaskPublication = await emailOnlyOwner.client.rpc(
      'publish_community_task_v1',
      {
        p_title: taskTitle,
        p_summary: taskSummary,
        p_requirements: taskRequirements,
        p_reward_budget_usdc: 0,
        p_reward_source: 'none',
        p_submission_deadline: submissionDeadline,
        p_audit_reference: `${runReference}:unauthorized-publish`,
      },
    );
    expect(
      Boolean(unauthorizedTaskPublication.error),
      'authenticated user without an operations role published a task',
    );
    assertions += 1;

    const directTaskInsert = await operator.client
      .from('community_tasks')
      .insert({
        title: taskTitle,
        summary: taskSummary,
        requirements: taskRequirements,
        reward_budget_usdc: 0,
        reward_source: 'none',
        status: 'open',
        publication_status: 'published',
        published_at: new Date().toISOString(),
      })
      .select('id');
    expect(
      Boolean(directTaskInsert.error)
        || (directTaskInsert.data ?? []).length === 0,
      'operator bypassed the audited task publication RPC',
    );
    assertions += 1;

    const taskPublication = await operator.client.rpc(
      'publish_community_task_v1',
      {
        p_title: taskTitle,
        p_summary: taskSummary,
        p_requirements: taskRequirements,
        p_reward_budget_usdc: 0,
        p_reward_source: 'none',
        p_submission_deadline: submissionDeadline,
        p_audit_reference: `${runReference}:task:publish`,
      },
    );
    assertNoError(taskPublication.error, 'audited task publication RPC');
    rows.taskId = requiredId(taskPublication.data, 'task publication RPC');
    assertions += 1;

    const publicRead = await publicClient
      .from('community_tasks')
      .select('id,title,status,publication_status')
      .eq('id', rows.taskId)
      .single();
    assertNoError(publicRead.error, 'anon public task read');
    expect(
      publicRead.data?.id === rows.taskId
        && publicRead.data.title === taskTitle
        && publicRead.data.status === 'open'
        && publicRead.data.publication_status === 'published',
      'anon did not read the RPC-published task',
    );
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

    const acceptedSummary =
      `Accepted Phase 4M Staging submission ${runId} for sanitized publication and immutable audit verification.`;
    const acceptedUrl =
      `https://example.com/alpha-staging-task-${runId}-accepted`;
    const rejectedSummary =
      `Rejected Phase 4M Staging submission ${runId} for terminal-state and no-publication verification.`;
    const rejectedUrl =
      `https://example.com/alpha-staging-task-${runId}-rejected`;

    const acceptedSubmissionInsert = await ownerA.client
      .from('task_submissions')
      .insert({
        task_id: rows.taskId,
        submitted_by: ownerA.user.id,
        summary: acceptedSummary,
        deliverable_url: acceptedUrl,
        wallet_address: requiredWallet(ownerA),
        wallet_verified: false,
        status: 'submitted',
        public_result_consent: true,
        public_wallet_consent: false,
      })
      .select('id')
      .single();
    assertNoError(acceptedSubmissionInsert.error, 'accepted-path submission insert');
    const acceptedSubmissionId = requiredId(
      acceptedSubmissionInsert.data?.id,
      'accepted-path submission',
    );
    rows.submissionIds.push(acceptedSubmissionId);
    assertions += 1;

    const rejectedSubmissionInsert = await ownerA.client
      .from('task_submissions')
      .insert({
        task_id: rows.taskId,
        submitted_by: ownerA.user.id,
        summary: rejectedSummary,
        deliverable_url: rejectedUrl,
        wallet_address: requiredWallet(ownerA),
        wallet_verified: false,
        status: 'submitted',
        public_result_consent: false,
        public_wallet_consent: false,
      })
      .select('id')
      .single();
    assertNoError(rejectedSubmissionInsert.error, 'rejected-path submission insert');
    const rejectedSubmissionId = requiredId(
      rejectedSubmissionInsert.data?.id,
      'rejected-path submission',
    );
    rows.submissionIds.push(rejectedSubmissionId);
    assertions += 1;

    const switchedWalletInsert = await ownerA.client
      .from('task_submissions')
      .insert({
        task_id: rows.taskId,
        submitted_by: ownerA.user.id,
        summary: 'This row must be rejected because the submitted wallet differs from the authenticated wallet.',
        deliverable_url: 'https://example.com/alpha-staging-switched-wallet',
        wallet_address: switchedWalletAddress,
        wallet_verified: false,
        status: 'submitted',
        public_result_consent: false,
        public_wallet_consent: false,
      });
    expect(
      Boolean(switchedWalletInsert.error),
      'wallet-authenticated owner submitted a different wallet address',
    );
    assertions += 1;

    const emailOnlyInsert = await emailOnlyOwner.client
      .from('task_submissions')
      .insert({
        task_id: rows.taskId,
        submitted_by: emailOnlyOwner.user.id,
        summary: 'This row must be rejected because email auth does not prove wallet ownership.',
        deliverable_url: 'https://example.com/alpha-staging-email-only',
        wallet_address: requiredWallet(ownerA),
        wallet_verified: false,
        status: 'submitted',
        public_result_consent: false,
        public_wallet_consent: false,
      });
    expect(
      Boolean(emailOnlyInsert.error),
      'email-only owner bypassed the Solana Web3 identity requirement',
    );
    assertions += 1;

    const ownerRead = await ownerA.client
      .from('task_submissions')
      .select('id')
      .in('id', rows.submissionIds);
    assertNoError(ownerRead.error, 'owner task submission read');
    expect(
      (ownerRead.data ?? []).length === 2,
      'owner did not receive both private submissions',
    );
    assertions += 1;

    const crossUserRead = await emailOnlyOwner.client
      .from('task_submissions')
      .select('id')
      .in('id', rows.submissionIds);
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
        wallet_address: requiredWallet(ownerA),
        wallet_verified: true,
        status: 'submitted',
        public_result_consent: false,
        public_wallet_consent: false,
      });
    expect(
      Boolean(forgedWalletVerification.error),
      'owner forged wallet_verified=true',
    );
    assertions += 1;

    await assignOperationsRole(admin, ownerA, 'reviewer');
    expect(
      ownerA.user.app_metadata.operations_role === 'reviewer',
      'wallet actor did not receive the refreshed reviewer claim',
    );
    assertions += 1;

    const unauthorizedReview = await emailOnlyOwner.client.rpc(
      'review_task_submission_v1',
      {
        p_submission_id: rejectedSubmissionId,
        p_decision: 'rejected',
        p_reviewer_notes: 'This no-role review must be denied.',
        p_public_result_summary: null,
        p_public_deliverable_url: null,
        p_audit_reference: `${runReference}:unauthorized-review`,
      },
    );
    expect(
      Boolean(unauthorizedReview.error),
      'authenticated user without an operations role reviewed a submission',
    );
    assertions += 1;

    const selfReview = await ownerA.client.rpc('review_task_submission_v1', {
      p_submission_id: rejectedSubmissionId,
      p_decision: 'rejected',
      p_reviewer_notes: 'This self-review must be denied.',
      p_public_result_summary: null,
      p_public_deliverable_url: null,
      p_audit_reference: `${runReference}:self-review`,
    });
    expect(Boolean(selfReview.error), 'reviewer completed a self-review');
    assertions += 1;

    const publicResultSummary =
      `Sanitized accepted result for Phase 4M Staging workflow ${runId}; no payment or treasury action occurred.`;
    const acceptedReview = await reviewer.client.rpc(
      'review_task_submission_v1',
      {
        p_submission_id: acceptedSubmissionId,
        p_decision: 'accepted',
        p_reviewer_notes: 'Staging reviewer verified the reserved fixture and contributor consent.',
        p_public_result_summary: publicResultSummary,
        p_public_deliverable_url: acceptedUrl,
        p_audit_reference: `${runReference}:accepted`,
      },
    );
    assertNoError(acceptedReview.error, 'accepted task review RPC');
    const acceptedReceipt = readSingleRpcRow(acceptedReview.data, 'accepted review');
    expect(
      acceptedReceipt.submission_id === acceptedSubmissionId
        && acceptedReceipt.submission_status === 'accepted',
      'accepted review RPC returned an invalid receipt',
    );
    rows.publicationId = requiredId(
      acceptedReceipt.publication_id,
      'accepted review publication',
    );
    assertions += 1;

    const replayReview = await reviewer.client.rpc(
      'review_task_submission_v1',
      {
        p_submission_id: acceptedSubmissionId,
        p_decision: 'rejected',
        p_reviewer_notes: 'Terminal review replay must be denied.',
        p_public_result_summary: null,
        p_public_deliverable_url: null,
        p_audit_reference: `${runReference}:accepted-replay`,
      },
    );
    expect(Boolean(replayReview.error), 'terminal accepted review was replayed');
    assertions += 1;

    const rejectedReview = await reviewer.client.rpc(
      'review_task_submission_v1',
      {
        p_submission_id: rejectedSubmissionId,
        p_decision: 'rejected',
        p_reviewer_notes: 'Staging reviewer rejected the second reserved fixture.',
        p_public_result_summary: null,
        p_public_deliverable_url: null,
        p_audit_reference: `${runReference}:rejected`,
      },
    );
    assertNoError(rejectedReview.error, 'rejected task review RPC');
    const rejectedReceipt = readSingleRpcRow(rejectedReview.data, 'rejected review');
    expect(
      rejectedReceipt.submission_id === rejectedSubmissionId
        && rejectedReceipt.submission_status === 'rejected'
        && rejectedReceipt.publication_id === null,
      'rejected review RPC returned an invalid receipt',
    );
    assertions += 1;

    const rejectedPublication = await publicClient
      .from('task_submission_publications')
      .select('id')
      .eq('review_reference', `${runReference}:rejected`);
    assertNoError(rejectedPublication.error, 'rejected publication absence read');
    expect(
      (rejectedPublication.data ?? []).length === 0,
      'rejected submission created a public result',
    );
    assertions += 1;

    const publicPublication = await publicClient
      .from('task_submission_publications')
      .select('*')
      .eq('id', rows.publicationId)
      .single();
    assertNoError(publicPublication.error, 'anon sanitized result read');
    const publicResult = readRecord(publicPublication.data, 'sanitized result');
    expect(
      publicResult.result_summary === publicResultSummary
        && publicResult.deliverable_url === acceptedUrl
        && publicResult.wallet_address === null
        && !('submission_id' in publicResult)
        && !('submitted_by' in publicResult)
        && !('reviewed_by' in publicResult),
      'sanitized public result leaked private identifiers or ignored consent',
    );
    assertions += 1;

    const anonWorkflowRead = await publicClient
      .from('operations_task_workflow_events')
      .select('event_id')
      .like('event_reference', `${runReference}:%`);
    expect(
      Boolean(anonWorkflowRead.error),
      'anon unexpectedly read private task workflow events',
    );
    assertions += 1;

    const staffWorkflowRead = await reviewer.client
      .from('operations_task_workflow_events')
      .select('event_id,action,event_reference')
      .like('event_reference', `${runReference}:%`);
    assertNoError(staffWorkflowRead.error, 'staff workflow audit read');
    expect(
      hasExactWorkflowActions(staffWorkflowRead.data, runReference),
      'task workflow audit did not contain the exact four expected actions',
    );
    assertions += 1;

    const publicationRewrite = await reviewer.client
      .from('task_submission_publications')
      .update({ result_summary: `Rewritten ${runId}` })
      .eq('id', rows.publicationId)
      .select('id');
    expect(
      Boolean(publicationRewrite.error)
        || (publicationRewrite.data ?? []).length === 0,
      'staff rewrote an immutable sanitized publication',
    );
    assertions += 1;

    const serviceRolePublicationDelete = await admin
      .from('task_submission_publications')
      .delete()
      .eq('id', rows.publicationId)
      .select('id');
    expect(
      Boolean(serviceRolePublicationDelete.error)
        || (serviceRolePublicationDelete.data ?? []).length === 0,
      'service-role client bypassed the controlled workflow cleanup RPC',
    );
    assertions += 1;

    const directTaskRewrite = await operator.client
      .from('community_tasks')
      .update({ title: `Rewritten ${runId}` })
      .eq('id', rows.taskId)
      .select('id');
    expect(
      Boolean(directTaskRewrite.error)
        || (directTaskRewrite.data ?? []).length === 0,
      'operator bypassed the task workflow RPC with a direct update',
    );
    assertions += 1;

    const directSubmissionRewrite = await reviewer.client
      .from('task_submissions')
      .update({ reviewer_notes: 'Direct mutation must be denied.' })
      .eq('id', acceptedSubmissionId)
      .select('id');
    expect(
      Boolean(directSubmissionRewrite.error)
        || (directSubmissionRewrite.data ?? []).length === 0,
      'reviewer bypassed the task review RPC with a direct update',
    );
    assertions += 1;

    const discussionInsert = await ownerA.client
      .from('governance_discussions')
      .insert({
        submitted_by: ownerA.user.id,
        topic: `Staging moderation ${runId}`,
        body: 'This private staging discussion verifies moderator SELECT and UPDATE policies.',
        wallet_address: requiredWallet(ownerA),
        wallet_verified: false,
        moderation_status: 'pending',
      })
      .select('id')
      .single();
    assertNoError(discussionInsert.error, 'discussion insert');
    rows.discussionId = requiredId(discussionInsert.data?.id, 'discussion');
    assertions += 1;

    const operatorDiscussionRead = await operator.client
      .from('governance_discussions')
      .select('id,moderation_status')
      .eq('id', rows.discussionId)
      .single();
    assertNoError(operatorDiscussionRead.error, 'operator discussion read');
    expect(
      operatorDiscussionRead.data?.moderation_status === 'pending',
      'operator did not receive the pending discussion',
    );
    assertions += 1;

    const unauthorizedModeration = await emailOnlyOwner.client
      .from('governance_discussions')
      .update({
        moderation_status: 'rejected',
        moderated_by: emailOnlyOwner.user.id,
      })
      .eq('id', rows.discussionId)
      .select('id');
    expect(
      Boolean(unauthorizedModeration.error)
        || (unauthorizedModeration.data ?? []).length === 0,
      'unprivileged user moderated another user discussion',
    );
    assertions += 1;

    const authorizedModeration = await operator.client
      .from('governance_discussions')
      .update({
        moderation_status: 'published',
        moderated_by: operator.user.id,
      })
      .eq('id', rows.discussionId)
      .select('id,moderation_status')
      .single();
    assertNoError(authorizedModeration.error, 'operator discussion update');
    expect(
      authorizedModeration.data?.moderation_status === 'published',
      'operator moderation did not persist',
    );
    assertions += 1;
  } catch (error) {
    primaryError = error;
  }

  const cleanup = await cleanupStagingFixtures(
    admin,
    rows,
    createdUsers,
    primaryError === null,
  );
  if (primaryError) {
    if (cleanup.errors.length > 0) {
      throw new Error(
        `${errorMessage(primaryError)}; staging E2E cleanup incomplete: ${cleanup.errors.join('; ')}`,
      );
    }
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

async function assignOperationsRole(
  admin: SupabaseClient,
  actor: TestActor,
  role: OperationsRole,
): Promise<void> {
  const updateResult = await admin.auth.admin.updateUserById(actor.user.id, {
    app_metadata: {
      ...actor.user.app_metadata,
      operations_role: role,
    },
  });
  assertNoError(updateResult.error, 'assign operations role');
  if (!updateResult.data.user) {
    throw new Error('assign operations role returned no user');
  }

  const refreshResult = await actor.client.auth.refreshSession();
  assertNoError(refreshResult.error, 'refresh operations role session');
  if (!refreshResult.data.user) {
    throw new Error('refresh operations role session returned no user');
  }
  expect(
    refreshResult.data.user.id === actor.user.id,
    'refresh operations role session returned a different user',
  );
  expect(
    refreshResult.data.user.app_metadata.operations_role === role,
    'refreshed session is missing the assigned operations role',
  );
  actor.user = refreshResult.data.user;
}

async function createActor(
  admin: SupabaseClient,
  config: OperationsStagingConfig,
  runId: string,
  label: string,
  operationsRole?: OperationsRole,
): Promise<TestActor> {
  const email = `alpha-operations-${runId}-${label}@example.com`;
  const createResult = await admin.auth.admin.createUser({
    email,
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
    const linkResult = await admin.auth.admin.generateLink({
      type: 'magiclink',
      email,
    });
    assertNoError(linkResult.error, `generate ${label} test session link`);
    const hashedToken = linkResult.data.properties?.hashed_token;
    if (!hashedToken) {
      throw new Error(`generate ${label} test session link returned no token hash`);
    }

    const signInResult = await client.auth.verifyOtp({
      token_hash: hashedToken,
      type: 'magiclink',
    });
    assertNoError(signInResult.error, `sign in ${label} test user`);
    if (!signInResult.data.user) {
      throw new Error(`sign in ${label} test user returned no user`);
    }
    expect(
      signInResult.data.user.id === createResult.data.user.id,
      `sign in ${label} test user returned a different user`,
    );

    return { user: signInResult.data.user, client, walletAddress: null };
  } catch (error) {
    await admin.auth.admin.deleteUser(createResult.data.user.id);
    throw error;
  }
}

async function createWalletActor(
  admin: SupabaseClient,
  config: OperationsStagingConfig,
): Promise<TestActor> {
  if (!config.web3Url || !config.e2eCaptchaToken) {
    throw new Error(
      'wallet actor requires OPERATIONS_STAGING_WEB3_URL and a transient CAPTCHA token',
    );
  }

  const client = createClient(config.supabaseUrl, config.publicKey, {
    auth: {
      autoRefreshToken: false,
      detectSessionInUrl: false,
      persistSession: false,
    },
  });
  const keypair = Keypair.generate();
  const privateKey = createPrivateKey({
    key: Buffer.concat([
      Buffer.from('302e020100300506032b657004220420', 'hex'),
      Buffer.from(keypair.secretKey.slice(0, 32)),
    ]),
    format: 'der',
    type: 'pkcs8',
  });
  const wallet = {
    publicKey: keypair.publicKey,
    signMessage: async (message: Uint8Array): Promise<Uint8Array> => (
      new Uint8Array(signWithPrivateKey(null, Buffer.from(message), privateKey))
    ),
  };

  const signInResult = await client.auth.signInWithWeb3({
    chain: 'solana',
    statement: OPERATIONS_WALLET_SIGN_IN_STATEMENT,
    wallet,
    options: {
      url: config.web3Url,
      captchaToken: config.e2eCaptchaToken,
    },
  });
  assertNoError(signInResult.error, 'sign in ephemeral Solana wallet actor');
  if (!signInResult.data.user) {
    throw new Error('sign in ephemeral Solana wallet actor returned no user');
  }

  try {
    return {
      user: signInResult.data.user,
      client,
      walletAddress: keypair.publicKey.toBase58(),
    };
  } catch (error) {
    await admin.auth.admin.deleteUser(signInResult.data.user.id);
    throw error;
  }
}

function requiredWallet(actor: TestActor): string {
  if (!actor.walletAddress) {
    throw new Error('test actor does not have a verified Solana wallet');
  }
  return actor.walletAddress;
}

async function cleanupStagingFixtures(
  admin: SupabaseClient,
  rows: CreatedRows,
  userIds: string[],
  requireCompleteWorkflow: boolean,
): Promise<{
  errors: string[];
  rowsDeleted: number;
  usersDeleted: number;
}> {
  const errors: string[] = [];
  let rowsDeleted = 0;
  let usersDeleted = 0;

  if (rows.discussionId) {
    const result = await admin
      .from('governance_discussions')
      .delete()
      .eq('id', rows.discussionId)
      .select('id');
    if (result.error) {
      errors.push('governance_discussions cleanup failed');
    } else if (!isExactCleanupDeletion(result.data, rows.discussionId)) {
      errors.push('governance_discussions cleanup count mismatch');
    } else {
      rowsDeleted += 1;
    }
  }

  if (rows.taskId) {
    const workflowCleanup = await admin.rpc(
      'cleanup_operations_task_staging_e2e_v1',
      {
        p_run_reference: rows.runReference,
        p_task_id: rows.taskId,
        p_submission_ids: rows.submissionIds,
      },
    );
    if (workflowCleanup.error) {
      errors.push('task workflow cleanup RPC failed');
    } else {
      const counts = readTaskWorkflowCleanupCounts(workflowCleanup.data);
      if (!counts) {
        errors.push('task workflow cleanup RPC returned invalid counts');
      } else if (
        requireCompleteWorkflow
        && (
          counts.publicationsDeleted !== 1
          || counts.eventsDeleted !== 4
          || counts.submissionsDeleted !== 2
          || counts.tasksDeleted !== 1
        )
      ) {
        errors.push('task workflow cleanup RPC count mismatch');
      } else {
        rowsDeleted += counts.publicationsDeleted
          + counts.eventsDeleted
          + counts.submissionsDeleted
          + counts.tasksDeleted;
      }
    }
  } else if (rows.submissionIds.length > 0 || rows.publicationId) {
    errors.push('task workflow cleanup identifiers are inconsistent');
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

export function readTaskWorkflowCleanupCounts(
  data: unknown,
): TaskWorkflowCleanupCounts | null {
  if (!Array.isArray(data) || data.length !== 1) {
    return null;
  }

  const row = readUnknownRecord(data[0]);
  if (!row) {
    return null;
  }

  const publicationsDeleted = readNonNegativeInteger(row.publications_deleted);
  const eventsDeleted = readNonNegativeInteger(row.events_deleted);
  const submissionsDeleted = readNonNegativeInteger(row.submissions_deleted);
  const tasksDeleted = readNonNegativeInteger(row.tasks_deleted);
  if (
    publicationsDeleted === null
    || eventsDeleted === null
    || submissionsDeleted === null
    || tasksDeleted === null
  ) {
    return null;
  }

  return {
    publicationsDeleted,
    eventsDeleted,
    submissionsDeleted,
    tasksDeleted,
  };
}

export function isExactCleanupDeletion(data: unknown, id: string): boolean {
  if (!Array.isArray(data) || data.length !== 1) {
    return false;
  }

  const row = data[0];
  return (
    typeof row === 'object'
    && row !== null
    && 'id' in row
    && row.id === id
  );
}

function readSingleRpcRow(
  data: unknown,
  label: string,
): Record<string, unknown> {
  if (!Array.isArray(data) || data.length !== 1) {
    throw new Error(`${label} RPC did not return exactly one row`);
  }
  return readRecord(data[0], `${label} RPC`);
}

function readRecord(value: unknown, label: string): Record<string, unknown> {
  const record = readUnknownRecord(value);
  if (!record) {
    throw new Error(`${label} did not return an object`);
  }
  return record;
}

function readUnknownRecord(value: unknown): Record<string, unknown> | null {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function readNonNegativeInteger(value: unknown): number | null {
  return typeof value === 'number'
    && Number.isSafeInteger(value)
    && value >= 0
    ? value
    : null;
}

function hasExactWorkflowActions(
  data: unknown,
  runReference: string,
): boolean {
  if (!Array.isArray(data) || data.length !== 4) {
    return false;
  }

  const actual = data.flatMap((value) => {
    const record = readUnknownRecord(value);
    return typeof record?.action === 'string'
      && typeof record.event_reference === 'string'
      ? [`${record.action}:${record.event_reference}`]
      : [];
  }).sort();
  const expected = [
    `result_published:${runReference}:accepted:publication`,
    `submission_accepted:${runReference}:accepted:decision`,
    `submission_rejected:${runReference}:rejected:decision`,
    `task_published:${runReference}:task:publish`,
  ].sort();
  return actual.length === expected.length
    && actual.every((value, index) => value === expected[index]);
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
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
