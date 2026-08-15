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
  discussionOwnerId: string | null;
  runReference: string;
  riskReportIds: string[];
  riskRunReference: string;
  reliefApplicationIds: string[];
  reliefRunReference: string;
}

interface TaskWorkflowCleanupCounts {
  publicationsDeleted: number;
  eventsDeleted: number;
  submissionsDeleted: number;
  tasksDeleted: number;
}

interface RiskWorkflowCleanupCounts {
  publicationsDeleted: number;
  eventsDeleted: number;
  evidenceDeleted: number;
  reportsDeleted: number;
}

interface ReliefWorkflowCleanupCounts {
  publicUpdatesDeleted: number;
  eventsDeleted: number;
  applicationsDeleted: number;
}

interface ReliefPaymentStateCounts {
  applicationsMatched: number;
  treasuryIntentsFound: number;
  paymentReceiptsFound: number;
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
  const riskRunReference = `phase-2e-6b-4n-staging-e2e:${runId}`;
  const reliefRunReference = `phase-2e-6b-4o-staging-e2e:${runId}`;
  const createdUsers: string[] = [];
  const rows: CreatedRows = {
    taskId: null,
    submissionIds: [],
    publicationId: null,
    discussionId: null,
    discussionOwnerId: null,
    runReference,
    riskReportIds: [],
    riskRunReference,
    reliefApplicationIds: [],
    reliefRunReference,
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

    const moderator = await createActor(admin, config, runId, 'moderator', 'moderator');
    createdUsers.push(moderator.user.id);

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

    await grantOperationsRole(admin, ownerA.user.id, 'reviewer', `${runReference}:owner:reviewer`);
    const ownerReviewerAccess = await ownerA.client.rpc('get_my_operations_access_v1');
    assertNoError(ownerReviewerAccess.error, 'load wallet actor operations access');
    expect(
      Array.isArray(ownerReviewerAccess.data)
        && ownerReviewerAccess.data.length === 1
        && ownerReviewerAccess.data[0]?.role_name === 'reviewer'
        && ownerReviewerAccess.data[0]?.status === 'active',
      'wallet actor did not receive audited reviewer access',
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

    const publishRiskIdentifier = `Staging risk publish ${runId}`;
    const dismissRiskIdentifier = `Staging risk dismiss ${runId}`;
    const publishRiskUrl = `https://example.com/alpha-staging-risk-${runId}-publish`;
    const dismissRiskUrl = `https://example.com/alpha-staging-risk-${runId}-dismiss`;
    const additionalRiskEvidenceUrl =
      `https://example.com/alpha-staging-risk-${runId}-additional-evidence`;

    const publishRiskInsert = await ownerA.client
      .from('risk_reports')
      .insert({
        submitted_by: ownerA.user.id,
        project_identifier: publishRiskIdentifier,
        summary: `Private Phase 4N Staging risk report ${runId}; facts and unverified interpretation remain separated.`,
        reference_url: publishRiskUrl,
        wallet_address: requiredWallet(ownerA),
        wallet_verified: false,
        public_report_consent: true,
        public_reference_consent: true,
      })
      .select('id')
      .single();
    assertNoError(publishRiskInsert.error, 'publish-path risk report insert');
    const publishRiskId = requiredId(publishRiskInsert.data?.id, 'publish-path risk report');
    rows.riskReportIds.push(publishRiskId);

    const dismissRiskInsert = await ownerA.client
      .from('risk_reports')
      .insert({
        submitted_by: ownerA.user.id,
        project_identifier: dismissRiskIdentifier,
        summary: `Private Phase 4N Staging dismissal report ${runId}; this fixture must never create a public record.`,
        reference_url: dismissRiskUrl,
        wallet_address: requiredWallet(ownerA),
        wallet_verified: false,
        public_report_consent: false,
        public_reference_consent: false,
      })
      .select('id')
      .single();
    assertNoError(dismissRiskInsert.error, 'dismiss-path risk report insert');
    const dismissRiskId = requiredId(dismissRiskInsert.data?.id, 'dismiss-path risk report');
    rows.riskReportIds.push(dismissRiskId);
    assertions += 2;

    const evidenceInsert = await ownerA.client
      .from('risk_evidence')
      .insert({
        risk_report_id: publishRiskId,
        submitted_by: ownerA.user.id,
        evidence_url: additionalRiskEvidenceUrl,
        content_sha256: 'a'.repeat(64),
        summary: `Additional private Phase 4N Staging evidence ${runId}.`,
        is_public: false,
      })
      .select('id')
      .single();
    assertNoError(evidenceInsert.error, 'private risk evidence insert');
    assertions += 1;

    const anonPrivateRiskRead = await publicClient
      .from('risk_reports')
      .select('id')
      .in('id', rows.riskReportIds);
    expect(Boolean(anonPrivateRiskRead.error), 'anon unexpectedly read private risk reports');
    assertions += 1;

    const unauthorizedRiskReview = await emailOnlyOwner.client.rpc(
      'review_risk_report_v1',
      {
        p_risk_report_id: dismissRiskId,
        p_decision: 'dismissed',
        p_reviewer_notes: 'This no-role risk review must be denied.',
        p_public_summary: null,
        p_public_reference_url: null,
        p_publication_basis: null,
        p_audit_reference: `${riskRunReference}:unauthorized`,
      },
    );
    expect(Boolean(unauthorizedRiskReview.error), 'user without a role reviewed a risk report');
    assertions += 1;

    const selfRiskReview = await ownerA.client.rpc('review_risk_report_v1', {
      p_risk_report_id: dismissRiskId,
      p_decision: 'dismissed',
      p_reviewer_notes: 'This self-review must be denied.',
      p_public_summary: null,
      p_public_reference_url: null,
      p_publication_basis: null,
      p_audit_reference: `${riskRunReference}:self-review`,
    });
    expect(Boolean(selfRiskReview.error), 'risk reporter completed a self-review');
    assertions += 1;

    const directRiskRewrite = await operator.client
      .from('risk_reports')
      .update({ reviewer_notes: 'Direct mutation must be denied.' })
      .eq('id', publishRiskId)
      .select('id');
    expect(
      Boolean(directRiskRewrite.error) || (directRiskRewrite.data ?? []).length === 0,
      'operator bypassed the audited risk review RPC',
    );
    assertions += 1;

    const publicRiskSummary =
      `Sanitized Phase 4N Staging risk finding ${runId} with private reporter and evidence metadata removed.`;
    const publishRiskReview = await reviewer.client.rpc('review_risk_report_v1', {
      p_risk_report_id: publishRiskId,
      p_decision: 'published',
      p_reviewer_notes: 'Independent Staging reviewer examined the private report and evidence fixture.',
      p_public_summary: publicRiskSummary,
      p_public_reference_url: publishRiskUrl,
      p_publication_basis: 'Controlled Phase 4N Staging evidence review with reporter publication consent.',
      p_audit_reference: `${riskRunReference}:published`,
    });
    assertNoError(publishRiskReview.error, 'published risk review RPC');
    const publishRiskReceipt = readSingleRpcRow(publishRiskReview.data, 'published risk review');
    expect(
      publishRiskReceipt.risk_report_id === publishRiskId
        && publishRiskReceipt.review_status === 'resolved'
        && typeof publishRiskReceipt.publication_id === 'string',
      'published risk review returned an invalid receipt',
    );
    assertions += 1;

    const dismissRiskReview = await reviewer.client.rpc('review_risk_report_v1', {
      p_risk_report_id: dismissRiskId,
      p_decision: 'dismissed',
      p_reviewer_notes: 'Independent Staging reviewer dismissed this fixture without publication.',
      p_public_summary: null,
      p_public_reference_url: null,
      p_publication_basis: null,
      p_audit_reference: `${riskRunReference}:dismissed`,
    });
    assertNoError(dismissRiskReview.error, 'dismissed risk review RPC');
    const dismissRiskReceipt = readSingleRpcRow(dismissRiskReview.data, 'dismissed risk review');
    expect(
      dismissRiskReceipt.risk_report_id === dismissRiskId
        && dismissRiskReceipt.review_status === 'dismissed'
        && dismissRiskReceipt.publication_id === null,
      'dismissed risk review returned an invalid receipt',
    );
    assertions += 1;

    const riskPublicationRead = await publicClient
      .from('risk_publications')
      .select('*')
      .eq('report_reference', `${riskRunReference}:published`)
      .single();
    assertNoError(riskPublicationRead.error, 'anon sanitized risk publication read');
    const riskPublication = readRecord(riskPublicationRead.data, 'sanitized risk publication');
    expect(
      riskPublication.project_identifier === publishRiskIdentifier
        && riskPublication.summary === publicRiskSummary
        && riskPublication.reference_url === publishRiskUrl
        && !('submitted_by' in riskPublication)
        && !('wallet_address' in riskPublication)
        && !('reviewer_notes' in riskPublication),
      'sanitized risk publication leaked private fields',
    );
    assertions += 1;

    const dismissedRiskPublication = await publicClient
      .from('risk_publications')
      .select('id')
      .eq('report_reference', `${riskRunReference}:dismissed`);
    assertNoError(dismissedRiskPublication.error, 'dismissed risk publication absence read');
    expect((dismissedRiskPublication.data ?? []).length === 0, 'dismissed report created a publication');
    assertions += 1;

    const replayRiskReview = await reviewer.client.rpc('review_risk_report_v1', {
      p_risk_report_id: publishRiskId,
      p_decision: 'dismissed',
      p_reviewer_notes: 'Terminal risk review replay must be denied.',
      p_public_summary: null,
      p_public_reference_url: null,
      p_publication_basis: null,
      p_audit_reference: `${riskRunReference}:replay`,
    });
    expect(Boolean(replayRiskReview.error), 'terminal risk review was replayed');
    assertions += 1;

    const anonRiskEventsRead = await publicClient
      .from('operations_risk_workflow_events')
      .select('event_id')
      .in('risk_report_id', rows.riskReportIds);
    expect(Boolean(anonRiskEventsRead.error), 'anon unexpectedly read private risk audit events');
    assertions += 1;

    const staffRiskEventsRead = await reviewer.client
      .from('operations_risk_workflow_events')
      .select('action,event_reference')
      .in('risk_report_id', rows.riskReportIds);
    assertNoError(staffRiskEventsRead.error, 'staff risk audit event read');
    expect(
      hasExactRiskWorkflowActions(staffRiskEventsRead.data, riskRunReference),
      'risk audit did not contain the exact publish and dismiss events',
    );
    assertions += 1;

    const immutableRiskPublication = await reviewer.client
      .from('risk_publications')
      .update({ summary: `Rewritten risk publication ${runId}` })
      .eq('report_reference', `${riskRunReference}:published`)
      .select('id');
    expect(
      Boolean(immutableRiskPublication.error)
        || (immutableRiskPublication.data ?? []).length === 0,
      'reviewer rewrote an immutable risk publication',
    );
    assertions += 1;

    const approvedReliefUrl =
      `https://example.com/alpha-staging-relief-${runId}-approve`;
    const rejectedReliefUrl =
      `https://example.com/alpha-staging-relief-${runId}-reject`;

    const approvedReliefInsert = await ownerA.client
      .from('relief_applications')
      .insert({
        submitted_by: ownerA.user.id,
        incident_summary: `Private Phase 4O Staging relief application ${runId}. This fixture verifies consented public progress without authorizing or sending any payment.`,
        requested_amount_usdc: '17.250000',
        evidence_url: approvedReliefUrl,
        wallet_address: requiredWallet(ownerA),
        wallet_verified: false,
        public_update_consent: true,
      })
      .select('id')
      .single();
    assertNoError(approvedReliefInsert.error, 'approved relief application insert');
    const approvedReliefId = requiredId(
      approvedReliefInsert.data?.id,
      'approved relief application',
    );
    rows.reliefApplicationIds.push(approvedReliefId);

    const rejectedReliefInsert = await ownerA.client
      .from('relief_applications')
      .insert({
        submitted_by: ownerA.user.id,
        incident_summary: `Private Phase 4O Staging rejected relief application ${runId}. This fixture must remain private and must never create a public progress record.`,
        requested_amount_usdc: '9.500000',
        evidence_url: rejectedReliefUrl,
        wallet_address: requiredWallet(ownerA),
        wallet_verified: false,
        public_update_consent: false,
      })
      .select('id')
      .single();
    assertNoError(rejectedReliefInsert.error, 'rejected relief application insert');
    const rejectedReliefId = requiredId(
      rejectedReliefInsert.data?.id,
      'rejected relief application',
    );
    rows.reliefApplicationIds.push(rejectedReliefId);
    assertions += 2;

    const anonReliefRead = await publicClient
      .from('relief_applications')
      .select('id')
      .in('id', rows.reliefApplicationIds);
    expect(
      Boolean(anonReliefRead.error),
      'anon unexpectedly read private relief applications',
    );
    assertions += 1;

    const unauthorizedReliefReview = await emailOnlyOwner.client.rpc(
      'review_relief_application_v1',
      {
        p_relief_application_id: rejectedReliefId,
        p_decision: 'rejected',
        p_reviewer_notes: 'This no-role relief review must be denied.',
        p_public_title: null,
        p_public_summary: null,
        p_publication_basis: null,
        p_audit_reference: `${reliefRunReference}:unauthorized`,
      },
    );
    expect(
      Boolean(unauthorizedReliefReview.error),
      'user without a role reviewed a relief application',
    );
    assertions += 1;

    const selfReliefReview = await ownerA.client.rpc(
      'review_relief_application_v1',
      {
        p_relief_application_id: rejectedReliefId,
        p_decision: 'rejected',
        p_reviewer_notes: 'This self-review must be denied.',
        p_public_title: null,
        p_public_summary: null,
        p_publication_basis: null,
        p_audit_reference: `${reliefRunReference}:self-review`,
      },
    );
    expect(Boolean(selfReliefReview.error), 'relief claimant completed a self-review');
    assertions += 1;

    const directReliefRewrite = await operator.client
      .from('relief_applications')
      .update({ reviewer_notes: 'Direct mutation must be denied.' })
      .eq('id', approvedReliefId)
      .select('id');
    expect(
      Boolean(directReliefRewrite.error)
        || (directReliefRewrite.data ?? []).length === 0,
      'operator bypassed the audited relief review RPC',
    );
    assertions += 1;

    const publicReliefTitle = `Phase 4O relief progress ${runId}`;
    const publicReliefSummary =
      `A reviewed Staging relief application was approved as an eligibility outcome. This public progress record contains no claimant identity, wallet, private evidence, or payment promise.`;
    const approvedReliefReview = await operator.client.rpc(
      'review_relief_application_v1',
      {
        p_relief_application_id: approvedReliefId,
        p_decision: 'approved',
        p_reviewer_notes: 'Independent Staging operator reviewed the private application and recorded eligibility only.',
        p_public_title: publicReliefTitle,
        p_public_summary: publicReliefSummary,
        p_publication_basis: 'Claimant consented to a sanitized public progress update during controlled Phase 4O Staging review.',
        p_audit_reference: `${reliefRunReference}:approved`,
      },
    );
    assertNoError(approvedReliefReview.error, 'approved relief review RPC');
    const approvedReliefReceipt = readSingleRpcRow(
      approvedReliefReview.data,
      'approved relief review',
    );
    expect(
      approvedReliefReceipt.relief_application_id === approvedReliefId
        && approvedReliefReceipt.review_status === 'approved'
        && typeof approvedReliefReceipt.public_update_id === 'string',
      'approved relief review returned an invalid receipt',
    );
    assertions += 1;

    const rejectedReliefReview = await operator.client.rpc(
      'review_relief_application_v1',
      {
        p_relief_application_id: rejectedReliefId,
        p_decision: 'rejected',
        p_reviewer_notes: 'Independent Staging operator rejected this fixture without publication or payment.',
        p_public_title: null,
        p_public_summary: null,
        p_publication_basis: null,
        p_audit_reference: `${reliefRunReference}:rejected`,
      },
    );
    assertNoError(rejectedReliefReview.error, 'rejected relief review RPC');
    const rejectedReliefReceipt = readSingleRpcRow(
      rejectedReliefReview.data,
      'rejected relief review',
    );
    expect(
      rejectedReliefReceipt.relief_application_id === rejectedReliefId
        && rejectedReliefReceipt.review_status === 'rejected'
        && rejectedReliefReceipt.public_update_id === null,
      'rejected relief review returned an invalid receipt',
    );
    assertions += 1;

    const publicReliefRead = await publicClient
      .from('relief_public_updates')
      .select('*')
      .eq('case_reference', `${reliefRunReference}:approved`)
      .single();
    assertNoError(publicReliefRead.error, 'anon sanitized relief update read');
    const publicRelief = readRecord(publicReliefRead.data, 'sanitized relief update');
    expect(
      publicRelief.title === publicReliefTitle
        && publicRelief.summary === publicReliefSummary
        && publicRelief.outcome === 'approved'
        && !('submitted_by' in publicRelief)
        && !('wallet_address' in publicRelief)
        && !('requested_amount_usdc' in publicRelief)
        && !('reviewer_notes' in publicRelief),
      'sanitized relief update leaked private or payment-sensitive fields',
    );
    assertions += 1;

    const rejectedReliefPublicRead = await publicClient
      .from('relief_public_updates')
      .select('id')
      .eq('case_reference', `${reliefRunReference}:rejected`);
    assertNoError(rejectedReliefPublicRead.error, 'rejected relief update absence read');
    expect(
      (rejectedReliefPublicRead.data ?? []).length === 0,
      'rejected relief application created a public update',
    );
    assertions += 1;

    const replayReliefReview = await operator.client.rpc(
      'review_relief_application_v1',
      {
        p_relief_application_id: approvedReliefId,
        p_decision: 'rejected',
        p_reviewer_notes: 'Terminal relief review replay must be denied.',
        p_public_title: null,
        p_public_summary: null,
        p_publication_basis: null,
        p_audit_reference: `${reliefRunReference}:replay`,
      },
    );
    expect(Boolean(replayReliefReview.error), 'terminal relief review was replayed');
    assertions += 1;

    const anonReliefEventsRead = await publicClient
      .from('operations_relief_workflow_events')
      .select('event_id')
      .in('relief_application_id', rows.reliefApplicationIds);
    expect(
      Boolean(anonReliefEventsRead.error),
      'anon unexpectedly read private relief audit events',
    );
    assertions += 1;

    const staffReliefEventsRead = await operator.client
      .from('operations_relief_workflow_events')
      .select('action,event_reference,event_data')
      .in('relief_application_id', rows.reliefApplicationIds);
    assertNoError(staffReliefEventsRead.error, 'staff relief audit event read');
    expect(
      hasExactReliefWorkflowActions(staffReliefEventsRead.data, reliefRunReference),
      'relief audit did not prove approval is separate from payment',
    );
    assertions += 1;

    const immutableReliefUpdate = await operator.client
      .from('relief_public_updates')
      .update({ summary: `Rewritten relief update ${runId}` })
      .eq('case_reference', `${reliefRunReference}:approved`)
      .select('id');
    expect(
      Boolean(immutableReliefUpdate.error)
        || (immutableReliefUpdate.data ?? []).length === 0,
      'operator rewrote an immutable relief public update',
    );
    assertions += 1;

    const reliefPaymentState = await admin.rpc(
      'inspect_operations_relief_staging_e2e_payment_state_v1',
      {
        p_run_reference: reliefRunReference,
        p_relief_application_ids: rows.reliefApplicationIds,
      },
    );
    assertNoError(reliefPaymentState.error, 'relief payment-state inspection RPC');
    const reliefPaymentCounts = readReliefPaymentStateCounts(reliefPaymentState.data);
    const reviewedReliefRead = await ownerA.client
      .from('relief_applications')
      .select('id,status,payment_receipt_id')
      .in('id', rows.reliefApplicationIds);
    assertNoError(reviewedReliefRead.error, 'claimant reviewed relief application read');
    expect(
      reliefPaymentCounts?.applicationsMatched === 2
        && reliefPaymentCounts.treasuryIntentsFound === 0
        && reliefPaymentCounts.paymentReceiptsFound === 0
        && (reviewedReliefRead.data ?? []).length === 2
        && (reviewedReliefRead.data ?? []).every(
          (application) => application.payment_receipt_id === null,
        ),
      'relief review created a treasury intent or payment receipt',
    );
    assertions += 1;

    const discussionInsert = await ownerA.client.rpc('submit_governance_discussion_v1', {
      p_proposal_id: null,
      p_topic: `Staging moderation ${runId}`,
      p_body: 'This private staging discussion verifies independent moderator review and sanitized publication.',
      p_public_body_consent: true,
      p_public_wallet_consent: false,
      p_submission_reference: `${runReference}:discussion-submitted`,
    });
    assertNoError(discussionInsert.error, 'discussion insert');
    rows.discussionId = requiredId(readSingleRpcRow(discussionInsert.data, 'discussion submission').discussion_id, 'discussion');
    rows.discussionOwnerId = ownerA.user.id;
    assertions += 1;

    const operatorDiscussionRead = await operator.client
      .from('governance_discussions')
      .select('id,moderation_status')
      .eq('id', rows.discussionId);
    expect(
      (operatorDiscussionRead.data ?? []).length === 0,
      'operator read a moderator-only private discussion',
    );
    assertions += 1;

    const unauthorizedModeration = await emailOnlyOwner.client.rpc('review_governance_discussion_v1', {
      p_discussion_id: rows.discussionId, p_decision: 'rejected',
      p_reviewer_notes: 'No-role moderation must fail.', p_public_topic: null,
      p_public_body: null, p_publication_basis: null,
      p_audit_reference: `${runReference}:discussion-unauthorized`,
    });
    expect(
      Boolean(unauthorizedModeration.error)
        || !unauthorizedModeration.data,
      'unprivileged user moderated another user discussion',
    );
    assertions += 1;

    const authorizedModeration = await moderator.client.rpc('review_governance_discussion_v1', {
      p_discussion_id: rows.discussionId, p_decision: 'published',
      p_reviewer_notes: 'Independent moderator approved a separately sanitized public version.',
      p_public_topic: `Sanitized staging moderation ${runId}`,
      p_public_body: 'A sanitized governance discussion was independently reviewed; private source content remains isolated.',
      p_publication_basis: 'Controlled Staging consent and moderator review.',
      p_audit_reference: `${runReference}:discussion-published`,
    });
    assertNoError(authorizedModeration.error, 'operator discussion update');
    expect(
      readSingleRpcRow(authorizedModeration.data, 'discussion review').moderation_status === 'published',
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

async function grantOperationsRole(
  admin: SupabaseClient,
  userId: string,
  role: OperationsRole,
  grantReference: string,
): Promise<void> {
  const result = await admin.rpc('grant_operations_role_v1', {
    p_user_id: userId,
    p_role_name: role,
    p_grant_reference: grantReference,
  });
  assertNoError(result.error, 'grant operations role');
}

async function revokeOperationsRole(
  admin: SupabaseClient,
  userId: string,
  revokeReference: string,
): Promise<void> {
  const result = await admin.rpc('revoke_operations_role_v1', {
    p_user_id: userId,
    p_revoke_reference: revokeReference,
  });
  assertNoError(result.error, 'revoke operations role');
}

export async function cleanupOperationsRoles(
  admin: SupabaseClient,
  userIds: string[],
  runReference: string,
  errors: string[],
): Promise<void> {
  for (const userId of [...new Set(userIds)].reverse()) {
    try {
      const result = await admin.rpc('inspect_operations_role_v1', {
        p_user_id: userId,
      });
      if (result.error || !Array.isArray(result.data)) {
        errors.push('operations role inspection cleanup failed');
        continue;
      }
      if (result.data.length === 0) {
        continue;
      }
      if (result.data.length !== 1) {
        errors.push('operations role inspection cleanup returned multiple rows');
        continue;
      }

      const row = readUnknownRecord(result.data[0]);
      if (!row) {
        errors.push('operations role inspection cleanup failed');
        continue;
      }
      if (row.status === 'active') {
        try {
          await revokeOperationsRole(admin, userId, `${runReference}:cleanup:revoke:${userId}`);
        } catch {
          errors.push('operations role revoke cleanup failed');
        }
      }
    } catch {
      errors.push('operations role inspection cleanup failed');
    }
  }
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

    if (operationsRole) {
      await grantOperationsRole(
        admin,
        signInResult.data.user.id,
        operationsRole,
        `phase-2e-6e-staging-e2e:${runId}:${label}:grant`,
      );
    }

    return { user: signInResult.data.user, client, walletAddress: null };
  } catch (error) {
    await admin.auth.admin.deleteUser(createResult.data.user.id, true);
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
    await admin.auth.admin.deleteUser(signInResult.data.user.id, true);
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

  if (rows.discussionId && rows.discussionOwnerId) {
    const result = await admin.rpc('cleanup_governance_discussion_staging_e2e_v1', {
      p_run_reference: rows.runReference,
      p_owner_id: rows.discussionOwnerId,
      p_discussion_id: rows.discussionId,
    });
    if (result.error) {
      errors.push('governance_discussions cleanup failed');
    } else if (Number(readSingleRpcRow(result.data, 'discussion cleanup').discussions_deleted) !== 1) {
      errors.push('governance_discussions cleanup count mismatch');
    } else {
      rowsDeleted += 4;
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

  if (rows.riskReportIds.length === 2) {
    const riskCleanup = await admin.rpc(
      'cleanup_operations_risk_staging_e2e_v1',
      {
        p_run_reference: rows.riskRunReference,
        p_risk_report_ids: rows.riskReportIds,
      },
    );
    if (riskCleanup.error) {
      errors.push('risk workflow cleanup RPC failed');
    } else {
      const counts = readRiskWorkflowCleanupCounts(riskCleanup.data);
      if (!counts) {
        errors.push('risk workflow cleanup RPC returned invalid counts');
      } else if (
        requireCompleteWorkflow
        && (
          counts.publicationsDeleted !== 1
          || counts.eventsDeleted !== 2
          || counts.evidenceDeleted !== 1
          || counts.reportsDeleted !== 2
        )
      ) {
        errors.push('risk workflow cleanup RPC count mismatch');
      } else {
        rowsDeleted += counts.publicationsDeleted
          + counts.eventsDeleted
          + counts.evidenceDeleted
          + counts.reportsDeleted;
      }
    }
  } else if (rows.riskReportIds.length > 0) {
    errors.push('risk workflow cleanup identifiers are inconsistent');
  }

  if (rows.reliefApplicationIds.length === 2) {
    const reliefCleanup = await admin.rpc(
      'cleanup_operations_relief_staging_e2e_v1',
      {
        p_run_reference: rows.reliefRunReference,
        p_relief_application_ids: rows.reliefApplicationIds,
      },
    );
    if (reliefCleanup.error) {
      errors.push('relief workflow cleanup RPC failed');
    } else {
      const counts = readReliefWorkflowCleanupCounts(reliefCleanup.data);
      if (!counts) {
        errors.push('relief workflow cleanup RPC returned invalid counts');
      } else if (
        requireCompleteWorkflow
        && (
          counts.publicUpdatesDeleted !== 1
          || counts.eventsDeleted !== 2
          || counts.applicationsDeleted !== 2
        )
      ) {
        errors.push('relief workflow cleanup RPC count mismatch');
      } else {
        rowsDeleted += counts.publicUpdatesDeleted
          + counts.eventsDeleted
          + counts.applicationsDeleted;
      }
    }
  } else if (rows.reliefApplicationIds.length > 0) {
    errors.push('relief workflow cleanup identifiers are inconsistent');
  }

  await cleanupOperationsRoles(admin, userIds, rows.runReference, errors);

  for (const userId of [...userIds].reverse()) {
    try {
      const result = await admin.auth.admin.deleteUser(userId, true);
      if (result.error) {
        errors.push('test user cleanup failed');
      } else {
        usersDeleted += 1;
      }
    } catch {
      errors.push('test user cleanup failed');
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

export function readRiskWorkflowCleanupCounts(
  data: unknown,
): RiskWorkflowCleanupCounts | null {
  if (!Array.isArray(data) || data.length !== 1) {
    return null;
  }
  const row = readUnknownRecord(data[0]);
  if (!row) {
    return null;
  }
  const publicationsDeleted = readNonNegativeInteger(row.publications_deleted);
  const eventsDeleted = readNonNegativeInteger(row.events_deleted);
  const evidenceDeleted = readNonNegativeInteger(row.evidence_deleted);
  const reportsDeleted = readNonNegativeInteger(row.reports_deleted);
  if (
    publicationsDeleted === null
    || eventsDeleted === null
    || evidenceDeleted === null
    || reportsDeleted === null
  ) {
    return null;
  }
  return {
    publicationsDeleted,
    eventsDeleted,
    evidenceDeleted,
    reportsDeleted,
  };
}

export function readReliefWorkflowCleanupCounts(
  data: unknown,
): ReliefWorkflowCleanupCounts | null {
  if (!Array.isArray(data) || data.length !== 1) {
    return null;
  }
  const row = readUnknownRecord(data[0]);
  if (!row) {
    return null;
  }
  const publicUpdatesDeleted = readNonNegativeInteger(row.public_updates_deleted);
  const eventsDeleted = readNonNegativeInteger(row.events_deleted);
  const applicationsDeleted = readNonNegativeInteger(row.applications_deleted);
  if (
    publicUpdatesDeleted === null
    || eventsDeleted === null
    || applicationsDeleted === null
  ) {
    return null;
  }
  return { publicUpdatesDeleted, eventsDeleted, applicationsDeleted };
}

export function readReliefPaymentStateCounts(
  data: unknown,
): ReliefPaymentStateCounts | null {
  if (!Array.isArray(data) || data.length !== 1) {
    return null;
  }
  const row = readUnknownRecord(data[0]);
  if (!row) {
    return null;
  }
  const applicationsMatched = readNonNegativeInteger(row.applications_matched);
  const treasuryIntentsFound = readNonNegativeInteger(row.treasury_intents_found);
  const paymentReceiptsFound = readNonNegativeInteger(row.payment_receipts_found);
  if (
    applicationsMatched === null
    || treasuryIntentsFound === null
    || paymentReceiptsFound === null
  ) {
    return null;
  }
  return { applicationsMatched, treasuryIntentsFound, paymentReceiptsFound };
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

function hasExactRiskWorkflowActions(
  data: unknown,
  runReference: string,
): boolean {
  if (!Array.isArray(data) || data.length !== 2) {
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
    `report_dismissed:${runReference}:dismissed`,
    `report_published:${runReference}:published`,
  ].sort();
  return actual.length === expected.length
    && actual.every((value, index) => value === expected[index]);
}

function hasExactReliefWorkflowActions(
  data: unknown,
  runReference: string,
): boolean {
  if (!Array.isArray(data) || data.length !== 2) {
    return false;
  }
  const actual = data.flatMap((value) => {
    const record = readUnknownRecord(value);
    const eventData = readUnknownRecord(record?.event_data);
    return typeof record?.action === 'string'
      && typeof record.event_reference === 'string'
      && eventData?.payment_intent_created === false
      && eventData.payment_receipt_created === false
      && eventData.approval_is_payment === false
      ? [`${record.action}:${record.event_reference}`]
      : [];
  }).sort();
  const expected = [
    `application_approved:${runReference}:approved`,
    `application_rejected:${runReference}:rejected`,
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
