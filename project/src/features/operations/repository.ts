import type { SupabaseClient, User } from '@supabase/supabase-js';
import {
  type MyOperationsSubmission,
  type MyOperationsAccess,
  type CommunityTaskPublicationInput,
  type OperationsStaffRole,
  type OperationsStaffWorkspace,
  type OperationsAccessStatus,
  OPERATIONS_PUBLIC_RECORD_LIMIT,
  OPERATIONS_STAFF_ROLES,
  type CommunityTask,
  type DiscussionInput,
  type GovernanceProposalInput,
  type GovernanceProposalReviewInput,
  type GovernanceDiscussionReviewInput,
  type GovernanceDecisionFinalizeInput,
  type GovernanceDecisionValue,
  type OperationsOverview,
  type PublicDiscussion,
  type PublicGovernanceProposal,
  type PublicGovernanceDecision,
  type PublicTreasuryExecutionRecord,
  type TreasuryExecutionPrepareInput,
  type TreasuryExecutionAuthorizeInput,
  type TreasuryExecutionCancelInput,
  type TreasuryExecutionReportInput,
  type TreasuryExecutionReconcileInput,
  type PublicReliefOutcome,
  type PublicReliefUpdate,
  type PublicRiskReport,
  type PublicTaskResult,
  type ReliefApplicationInput,
  type ReliefApplicationReviewInput,
  type RiskEvidenceInput,
  type RiskReportInput,
  type RiskReportReviewInput,
  type TaskSubmissionInput,
  type TaskSubmissionReviewInput,
  validateCommunityTaskPublication,
  validateDiscussion,
  validateGovernanceProposal,
  validateReliefApplication,
  validateReliefApplicationReview,
  validateRiskEvidence,
  validateRiskReport,
  validateRiskReportReview,
  validateTaskSubmission,
  validateTaskSubmissionReview,
  validateTreasuryExecutionPrepare,
  validateTreasuryExecutionAuthorize,
  validateTreasuryExecutionCancel,
  validateTreasuryExecutionReport,
  validateTreasuryExecutionReconcile,
} from './domain';
import { assertWalletSessionMatch } from './auth';
import {
  getOperationsSupabase,
  operationsBackendConfig,
} from '../../lib/operationsSupabase';

export class OperationsBackendError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'OperationsBackendError';
  }
}

export async function loadOperationsOverview(): Promise<OperationsOverview> {
  const client = requirePublicReadClient();

  const [
    taskResult,
    taskPublicationResult,
    riskResult,
    reliefResult,
    discussionResult,
    proposalResult,
    decisionResult,
    treasuryExecutionResult,
  ] = await Promise.all([
    client
      .from('community_tasks')
      .select('id,title,summary,requirements,reward_budget_usdc,reward_source,status,submission_deadline,published_at')
      .eq('publication_status', 'published')
      .in('status', ['open', 'under_review'])
      .order('published_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('task_submission_publications')
      .select('id,task_id,task_title,result_summary,deliverable_url,wallet_address,review_reference,accepted_at,published_at')
      .order('published_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('risk_publications')
      .select('id,project_identifier,summary,reference_url,public_status,published_at')
      .order('published_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('relief_public_updates')
      .select('id,case_reference,title,summary,outcome,published_at')
      .order('published_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('governance_discussion_publications')
      .select('id,topic,body,wallet_address,published_at')
      .order('published_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('governance_proposals')
      .select('id,title,summary,proposal_kind,public_source_reference,execution_required,execution_manifest_url,status,published_at')
      .eq('publication_status', 'published')
      .order('published_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('governance_decisions')
      .select('id,proposal_id,decision,rationale,decision_hash,execution_required,execution_reference,execution_manifest_sha256,finalization_reference,decided_at')
      .eq('publication_status', 'published')
      .order('decided_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('treasury_execution_public_registry')
      .select('intent_public_id,governance_decision_id,decision_hash,manifest_sha256,intent_hash,purpose_reference,asset_symbol,asset_decimals,asset_mint,destination_wallet_display,amount_base_units,network,public_status,external_execution_reference,prepared_at,authorized_at,reported_at,reconciled_at,reconciliation_reference')
      .order('updated_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
  ]);

  assertNoQueryError(taskResult.error, '绀惧尯浠诲姟');
  assertNoQueryError(taskPublicationResult.error, '鍏紑浠诲姟鎴愭灉');
  assertNoQueryError(riskResult.error, '椋庨櫓鎶ュ憡');
  assertNoQueryError(reliefResult.error, '鏁戝姪鍏紑杩涘害');
  assertNoQueryError(discussionResult.error, '娌荤悊璁ㄨ');
  assertNoQueryError(proposalResult.error, '娌荤悊鎻愭');
  assertNoQueryError(decisionResult.error, '娌荤悊鍐冲畾');
  assertNoQueryError(treasuryExecutionResult.error, '鍏紑鎵ц鐧昏');

  const proposals = new Map(
    (proposalResult.data ?? []).map((row) => [row.id, row.title]),
  );

  return {
    tasks: (taskResult.data ?? []).map(mapTask),
    taskResults: (taskPublicationResult.data ?? []).map(mapTaskResult),
    riskReports: (riskResult.data ?? []).map(mapRiskReport),
    reliefUpdates: (reliefResult.data ?? []).map(mapReliefUpdate),
    discussions: (discussionResult.data ?? []).map(mapDiscussion),
    governanceProposals: (proposalResult.data ?? []).map(mapProposal),
    governanceDecisions: (decisionResult.data ?? []).map((row) => mapDecision(
      row,
      proposals.get(row.proposal_id) ?? '鏈叕寮€鎻愭鏍囬',
    )),
    treasuryExecutions: (treasuryExecutionResult.data ?? []).map(mapTreasuryExecution),
  };
}

export async function submitTaskResult(input: TaskSubmissionInput): Promise<void> {
  const payload = validateTaskSubmission(input);
  const { client, user } = await requireIntakeSession(payload.walletAddress);
  const { error } = await client.from('task_submissions').insert({
    task_id: payload.taskId,
    submitted_by: user.id,
    summary: payload.summary,
    deliverable_url: payload.deliverableUrl,
    wallet_address: payload.walletAddress,
    wallet_verified: false,
    public_result_consent: payload.publicResultConsent,
    public_wallet_consent: payload.publicWalletConsent,
    status: 'submitted',
  });

  assertNoMutationError(error, '浠诲姟鎴愭灉');
}

export function resolveOperationsStaffRole(role: unknown): OperationsStaffRole | null {
  return typeof role === 'string'
    && (OPERATIONS_STAFF_ROLES as readonly string[]).includes(role)
    ? role as OperationsStaffRole
    : null;
}

export async function loadMyOperationsAccess(): Promise<MyOperationsAccess> {
  const client = getOperationsSupabase();
  if (!client) {
    throw new OperationsBackendError('operations backend is not configured');
  }

  return readMyOperationsAccess(client);
}

export async function loadOperationsStaffWorkspace(): Promise<OperationsStaffWorkspace> {
  const { client } = await requireStaffSession();
  const [submissionResult, taskResult, eventResult, riskResult, evidenceResult, riskEventResult, reliefResult, reliefEventResult, proposalSubmissionResult, discussionResult, governanceEventResult, treasuryIntentResult, treasuryEventResult] = await Promise.all([
    client
      .from('task_submissions')
      .select('id,task_id,submitted_by,summary,deliverable_url,wallet_address,public_result_consent,public_wallet_consent,status,reviewer_notes,created_at')
      .in('status', ['submitted', 'in_review'])
      .order('created_at', { ascending: true })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('community_tasks')
      .select('id,title')
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('operations_task_workflow_events')
      .select('event_id,entity_type,entity_reference,action,actor_role,event_reference,created_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('risk_reports')
      .select('id,submitted_by,project_identifier,summary,reference_url,wallet_address,public_report_consent,public_reference_consent,review_status,reviewer_notes,created_at')
      .in('review_status', ['submitted', 'triaged', 'investigating'])
      .eq('publication_status', 'private')
      .order('created_at', { ascending: true })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('risk_evidence')
      .select('risk_report_id')
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT * 12),
    client
      .from('operations_risk_workflow_events')
      .select('event_id,risk_report_id,action,actor_role,event_reference,created_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('relief_applications')
      .select('id,submitted_by,incident_summary,requested_amount_usdc,evidence_url,wallet_address,public_update_consent,status,reviewer_notes,created_at')
      .in('status', ['submitted', 'triaged', 'evidence_requested', 'under_review'])
      .order('created_at', { ascending: true })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('operations_relief_workflow_events')
      .select('event_id,relief_application_id,action,actor_role,event_reference,created_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('governance_proposal_submissions')
      .select('id,submitted_by,title,private_summary,proposal_kind,execution_required,execution_manifest_sha256,public_proposal_consent,review_status,created_at')
      .eq('review_status', 'pending')
      .order('created_at', { ascending: true })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('governance_discussions')
      .select('id,proposal_id,submitted_by,topic,body,wallet_address,public_body_consent,public_wallet_consent,moderation_status,created_at')
      .eq('moderation_status', 'pending')
      .order('created_at', { ascending: true })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('operations_governance_workflow_events')
      .select('event_id,action,actor_role,event_reference,created_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('treasury_execution_intents')
      .select('id,governance_decision_id,decision_hash,manifest_sha256,intent_hash,pool,network,asset_symbol,asset_mint,destination_wallet,amount_base_units,recipient_verification_reference,purpose_reference,status,prepared_by,authorized_by,reported_by,submitted_signature,created_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('operations_treasury_execution_workflow_events')
      .select('event_id,execution_intent_id,action,previous_status,new_status,actor_role,audit_reference,created_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
  ]);
  assertNoQueryError(submissionResult.error, 'staff task submissions');
  assertNoQueryError(taskResult.error, 'staff community tasks');
  assertNoQueryError(eventResult.error, 'staff task workflow events');
  assertNoQueryError(riskResult.error, 'staff risk reports');
  assertNoQueryError(evidenceResult.error, 'staff risk evidence');
  assertNoQueryError(riskEventResult.error, 'staff risk workflow events');
  assertNoQueryError(reliefResult.error, 'staff relief applications');
  assertNoQueryError(reliefEventResult.error, 'staff relief workflow events');
  assertNoQueryError(proposalSubmissionResult.error, 'staff governance proposal submissions');
  assertNoQueryError(discussionResult.error, 'staff governance discussions');
  assertNoQueryError(governanceEventResult.error, 'staff governance workflow events');
  assertNoQueryError(treasuryIntentResult.error, 'staff treasury execution intents');
  assertNoQueryError(treasuryEventResult.error, 'staff treasury execution workflow events');
  const taskTitles = new Map((taskResult.data ?? []).map((row) => [row.id, row.title]));
  const evidenceCounts = new Map<string, number>();
  for (const row of evidenceResult.data ?? []) {
    evidenceCounts.set(row.risk_report_id, (evidenceCounts.get(row.risk_report_id) ?? 0) + 1);
  }

  return {
    submissions: (submissionResult.data ?? []).map((row) => ({
      id: row.id,
      taskId: row.task_id,
      taskTitle: taskTitles.get(row.task_id) ?? 'Unknown task',
      summary: row.summary,
      deliverableUrl: row.deliverable_url,
      walletAddress: row.wallet_address,
      publicResultConsent: row.public_result_consent,
      publicWalletConsent: row.public_wallet_consent,
      status: row.status,
      submittedBy: row.submitted_by,
      reviewerNotes: row.reviewer_notes,
      createdAt: row.created_at,
    })),
    events: (eventResult.data ?? []).map((row) => ({
      eventId: String(row.event_id),
      entityType: row.entity_type,
      entityReference: row.entity_reference,
      action: row.action,
      actorRole: row.actor_role,
      eventReference: row.event_reference,
      createdAt: row.created_at,
    })),
    riskReports: (riskResult.data ?? []).map((row) => ({
      id: row.id,
      projectIdentifier: row.project_identifier,
      summary: row.summary,
      referenceUrl: row.reference_url,
      walletAddress: row.wallet_address,
      publicReportConsent: row.public_report_consent,
      publicReferenceConsent: row.public_reference_consent,
      reviewStatus: row.review_status,
      submittedBy: row.submitted_by,
      reviewerNotes: row.reviewer_notes,
      evidenceCount: evidenceCounts.get(row.id) ?? 0,
      createdAt: row.created_at,
    })),
    riskEvents: (riskEventResult.data ?? []).map((row) => ({
      eventId: String(row.event_id),
      riskReportId: row.risk_report_id,
      action: row.action,
      actorRole: row.actor_role,
      eventReference: row.event_reference,
      createdAt: row.created_at,
    })),
    reliefApplications: (reliefResult.data ?? []).map((row) => ({
      id: row.id,
      incidentSummary: row.incident_summary,
      requestedAmountUsdc: String(row.requested_amount_usdc),
      evidenceUrl: row.evidence_url,
      walletAddress: row.wallet_address,
      publicUpdateConsent: row.public_update_consent,
      status: row.status,
      submittedBy: row.submitted_by,
      reviewerNotes: row.reviewer_notes,
      createdAt: row.created_at,
    })),
    reliefEvents: (reliefEventResult.data ?? []).map((row) => ({
      eventId: String(row.event_id),
      reliefApplicationId: row.relief_application_id,
      action: row.action,
      actorRole: row.actor_role,
      eventReference: row.event_reference,
      createdAt: row.created_at,
    })),
    proposalSubmissions: (proposalSubmissionResult.data ?? []).map((row) => ({
      id: row.id,
      title: row.title,
      privateSummary: row.private_summary,
      proposalKind: row.proposal_kind,
      executionRequired: row.execution_required,
      executionManifestSha256: row.execution_manifest_sha256,
      publicProposalConsent: row.public_proposal_consent,
      submittedBy: row.submitted_by,
      reviewStatus: row.review_status,
      createdAt: row.created_at,
    })),
    discussions: (discussionResult.data ?? []).map((row) => ({
      id: row.id,
      proposalId: row.proposal_id,
      topic: row.topic,
      body: row.body,
      walletAddress: row.wallet_address,
      publicBodyConsent: row.public_body_consent,
      publicWalletConsent: row.public_wallet_consent,
      submittedBy: row.submitted_by,
      moderationStatus: row.moderation_status,
      createdAt: row.created_at,
    })),
    governanceEvents: (governanceEventResult.data ?? []).map((row) => ({
      eventId: String(row.event_id),
      action: row.action,
      actorRole: row.actor_role,
      eventReference: row.event_reference,
      createdAt: row.created_at,
    })),
    treasuryExecutionIntents: (treasuryIntentResult.data ?? []).map((row) => ({
      id: row.id,
      governanceDecisionId: row.governance_decision_id,
      decisionHash: row.decision_hash,
      manifestSha256: row.manifest_sha256,
      intentHash: row.intent_hash,
      pool: row.pool,
      network: row.network,
      assetSymbol: row.asset_symbol,
      assetMint: row.asset_mint,
      destinationWallet: row.destination_wallet,
      amountBaseUnits: String(row.amount_base_units),
      recipientVerificationReference: row.recipient_verification_reference,
      purposeReference: row.purpose_reference,
      status: row.status,
      preparedBy: row.prepared_by,
      authorizedBy: row.authorized_by,
      reportedBy: row.reported_by,
      submittedSignature: row.submitted_signature,
      createdAt: row.created_at,
    })),
    treasuryExecutionEvents: (treasuryEventResult.data ?? []).map((row) => ({
      eventId: String(row.event_id),
      executionIntentId: row.execution_intent_id,
      action: row.action,
      previousStatus: row.previous_status,
      newStatus: row.new_status,
      actorRole: row.actor_role,
      auditReference: row.audit_reference,
      createdAt: row.created_at,
    })),
  };
}

export async function publishCommunityTask(
  input: CommunityTaskPublicationInput,
): Promise<string> {
  const payload = validateCommunityTaskPublication(input);
  const { client, role } = await requireStaffSession();
  if (role === 'reviewer' || role === 'relief_reviewer') {
    throw new OperationsBackendError('reviewer 涓嶈兘鍙戝竷绀惧尯浠诲姟');
  }

  const result = await client.rpc('publish_community_task_v1', {
    p_title: payload.title,
    p_summary: payload.summary,
    p_requirements: payload.requirements,
    p_reward_budget_usdc: payload.rewardBudgetUsdc,
    p_reward_source: payload.rewardSource,
    p_submission_deadline: payload.submissionDeadline,
    p_audit_reference: payload.auditReference,
  });
  assertNoMutationError(result.error, '绀惧尯浠诲姟鍙戝竷');
  if (typeof result.data !== 'string') {
    throw new OperationsBackendError('绀惧尯浠诲姟鍙戝竷缁撴灉缂哄皯浠诲姟鏍囪瘑');
  }
  return result.data;
}

export async function reviewTaskSubmission(input: TaskSubmissionReviewInput): Promise<void> {
  const payload = validateTaskSubmissionReview(input);
  const { client } = await requireStaffSession();
  const result = await client.rpc('review_task_submission_v1', {
    p_submission_id: payload.submissionId,
    p_decision: payload.decision,
    p_reviewer_notes: payload.reviewerNotes,
    p_public_result_summary: payload.publicResultSummary,
    p_public_deliverable_url: payload.publicDeliverableUrl,
    p_audit_reference: payload.auditReference,
  });
  assertNoMutationError(result.error, '浠诲姟鎴愭灉瀹℃牳');
}

export async function submitRiskReport(input: RiskReportInput): Promise<void> {
  const payload = validateRiskReport(input);
  const { client, user } = await requireIntakeSession(payload.walletAddress);
  const { error } = await client.from('risk_reports').insert({
    submitted_by: user.id,
    project_identifier: payload.projectIdentifier,
    summary: payload.summary,
    reference_url: payload.referenceUrl,
    wallet_address: payload.walletAddress,
    wallet_verified: false,
    review_status: 'submitted',
    publication_status: 'private',
    public_report_consent: payload.publicReportConsent,
    public_reference_consent: payload.publicReferenceConsent,
  });

  assertNoMutationError(error, '椋庨櫓鎶ュ憡');
}

export async function submitRiskEvidence(input: RiskEvidenceInput): Promise<void> {
  const payload = validateRiskEvidence(input);
  const { client, user } = await requireIntakeSession(payload.walletAddress);
  const { error } = await client.from('risk_evidence').insert({
    risk_report_id: payload.riskReportId,
    submitted_by: user.id,
    evidence_url: payload.evidenceUrl,
    content_sha256: payload.contentSha256,
    summary: payload.summary,
    is_public: false,
  });
  assertNoMutationError(error, '椋庨櫓璇佹嵁');
}

export async function reviewRiskReport(input: RiskReportReviewInput): Promise<void> {
  const payload = validateRiskReportReview(input);
  const { client } = await requireStaffSession();
  const result = await client.rpc('review_risk_report_v1', {
    p_risk_report_id: payload.riskReportId,
    p_decision: payload.decision,
    p_reviewer_notes: payload.reviewerNotes,
    p_public_summary: payload.publicSummary,
    p_public_reference_url: payload.publicReferenceUrl,
    p_publication_basis: payload.publicationBasis,
    p_audit_reference: payload.auditReference,
  });
  assertNoMutationError(result.error, '椋庨櫓鎶ュ憡瀹℃牳');
}

export async function submitReliefApplication(input: ReliefApplicationInput): Promise<void> {
  const payload = validateReliefApplication(input);
  const { client, user } = await requireIntakeSession(payload.walletAddress);
  const { error } = await client.from('relief_applications').insert({
    submitted_by: user.id,
    incident_summary: payload.incidentSummary,
    requested_amount_usdc: payload.requestedAmountUsdc,
    evidence_url: payload.evidenceUrl,
    wallet_address: payload.walletAddress,
    wallet_verified: false,
    status: 'submitted',
    public_update_consent: payload.publicUpdateConsent,
  });

  assertNoMutationError(error, '鏁戝姪鐢宠');
}

export async function reviewReliefApplication(
  input: ReliefApplicationReviewInput,
): Promise<void> {
  const payload = validateReliefApplicationReview(input);
  const { client } = await requireStaffSession();
  const result = await client.rpc('review_relief_application_v1', {
    p_relief_application_id: payload.reliefApplicationId,
    p_decision: payload.decision,
    p_reviewer_notes: payload.reviewerNotes,
    p_public_title: payload.publicTitle,
    p_public_summary: payload.publicSummary,
    p_publication_basis: payload.publicationBasis,
    p_audit_reference: payload.auditReference,
  });
  assertNoMutationError(result.error, '鏁戝姪鐢宠瀹℃牳');
}

export async function submitDiscussion(input: DiscussionInput): Promise<void> {
  const payload = validateDiscussion(input);
  const { client } = await requireIntakeSession(payload.walletAddress);
  const { error } = await client.rpc('submit_governance_discussion_v1', {
    p_proposal_id: payload.proposalId,
    p_topic: payload.topic,
    p_body: payload.body,
    p_public_body_consent: payload.publicBodyConsent,
    p_public_wallet_consent: payload.publicWalletConsent,
    p_submission_reference: payload.submissionReference,
  });

  assertNoMutationError(error, '娌荤悊璁ㄨ');
}

export async function submitGovernanceProposal(input: GovernanceProposalInput): Promise<void> {
  const payload = validateGovernanceProposal(input);
  const { client } = await requireIntakeSession(payload.walletAddress);
  const { error } = await client.rpc('submit_governance_proposal_v1', {
    p_title: payload.title,
    p_private_summary: payload.privateSummary,
    p_proposal_kind: payload.proposalKind,
    p_execution_required: payload.executionRequired,
    p_private_execution_manifest: payload.privateExecutionManifest,
    p_execution_manifest_sha256: payload.executionManifestSha256 || null,
    p_public_proposal_consent: payload.publicProposalConsent,
    p_submission_reference: payload.submissionReference,
  });
  assertNoMutationError(error, '娌荤悊鎻愭');
}

export async function reviewGovernanceProposal(input: GovernanceProposalReviewInput): Promise<void> {
  const { client } = await requireStaffSession();
  const { error } = await client.rpc('publish_governance_proposal_v1', {
    p_proposal_submission_id: input.proposalSubmissionId,
    p_decision: input.decision,
    p_reviewer_notes: input.reviewerNotes.trim(),
    p_public_title: input.publicTitle.trim() || null,
    p_public_summary: input.publicSummary.trim() || null,
    p_public_source_reference: input.publicSourceReference.trim() || null,
    p_execution_manifest_url: input.executionManifestUrl.trim() || null,
    p_execution_manifest_sha256: input.executionManifestSha256.trim() || null,
    p_audit_reference: input.auditReference.trim(),
  });
  assertNoMutationError(error, '娌荤悊鎻愭瀹℃牳');
}

export async function reviewGovernanceDiscussion(input: GovernanceDiscussionReviewInput): Promise<void> {
  const { client } = await requireStaffSession();
  const { error } = await client.rpc('review_governance_discussion_v1', {
    p_discussion_id: input.discussionId,
    p_decision: input.decision,
    p_reviewer_notes: input.reviewerNotes.trim(),
    p_public_topic: input.publicTopic.trim() || null,
    p_public_body: input.publicBody.trim() || null,
    p_publication_basis: input.publicationBasis.trim() || null,
    p_audit_reference: input.auditReference.trim(),
  });
  assertNoMutationError(error, '娌荤悊璁ㄨ瀹℃牳');
}

export async function finalizeGovernanceDecision(input: GovernanceDecisionFinalizeInput): Promise<void> {
  const { client } = await requireStaffSession();
  const { error } = await client.rpc('finalize_governance_decision_v1', {
    p_proposal_id: input.proposalId,
    p_decision: input.decision,
    p_rationale: input.rationale.trim(),
    p_execution_manifest_sha256: input.executionManifestSha256.trim() || null,
    p_finalization_reference: input.finalizationReference.trim(),
  });
  assertNoMutationError(error, '娌荤悊鍐冲畾缁堝眬纭');
}

export async function prepareTreasuryExecutionIntent(input: TreasuryExecutionPrepareInput): Promise<void> {
  const payload = validateTreasuryExecutionPrepare(input);
  const { client } = await requireStaffSession();
  const { error } = await client.rpc('prepare_treasury_execution_intent_v1', {
    p_governance_decision_id: payload.governanceDecisionId,
    p_pool: payload.pool,
    p_relief_application_id: payload.reliefApplicationId || null,
    p_network: payload.network,
    p_asset_symbol: payload.assetSymbol,
    p_asset_decimals: payload.assetDecimals,
    p_asset_mint: payload.assetMint,
    p_destination_wallet: payload.destinationWallet,
    p_amount_base_units: payload.amountBaseUnits,
    p_recipient_verification_reference: payload.recipientVerificationReference,
    p_purpose_reference: payload.purposeReference,
    p_private_note: payload.privateNote,
    p_audit_reference: payload.auditReference,
  });
  assertNoMutationError(error, '鎵ц intent 鍑嗗');
}

export async function authorizeTreasuryExecutionIntent(input: TreasuryExecutionAuthorizeInput): Promise<void> {
  const payload = validateTreasuryExecutionAuthorize(input);
  const { client } = await requireStaffSession();
  const { error } = await client.rpc('authorize_treasury_execution_intent_v1', {
    p_execution_intent_id: payload.executionIntentId,
    p_authorization_reference: payload.authorizationReference,
    p_private_note: payload.privateNote,
    p_audit_reference: payload.auditReference,
  });
  assertNoMutationError(error, '鎵ц intent 鎺堟潈');
}

export async function cancelTreasuryExecutionIntent(input: TreasuryExecutionCancelInput): Promise<void> {
  const payload = validateTreasuryExecutionCancel(input);
  const { client } = await requireStaffSession();
  const { error } = await client.rpc('cancel_treasury_execution_intent_v1', {
    p_execution_intent_id: payload.executionIntentId,
    p_cancellation_reference: payload.cancellationReference,
    p_private_note: payload.privateNote,
    p_audit_reference: payload.auditReference,
  });
  assertNoMutationError(error, '鎵ц intent 鍙栨秷');
}

export async function reportTreasuryExecutionReceipt(input: TreasuryExecutionReportInput): Promise<void> {
  const payload = validateTreasuryExecutionReport(input);
  const { client } = await requireStaffSession();
  const { error } = await client.rpc('report_treasury_execution_receipt_v1', {
    p_execution_intent_id: payload.executionIntentId,
    p_transaction_signature: payload.transactionSignature,
    p_confirmed_at: payload.confirmedAt,
    p_private_note: payload.privateNote,
    p_audit_reference: payload.auditReference,
  });
  assertNoMutationError(error, '澶栭儴鎵ц鍥炴墽鐧昏');
}

export async function reconcileTreasuryExecution(input: TreasuryExecutionReconcileInput): Promise<void> {
  const payload = validateTreasuryExecutionReconcile(input);
  const { client } = await requireStaffSession();
  const { error } = await client.rpc('reconcile_treasury_execution_v1', {
    p_execution_intent_id: payload.executionIntentId,
    p_outcome: payload.outcome,
    p_reconciliation_reference: payload.reconciliationReference,
    p_private_note: payload.privateNote,
    p_audit_reference: payload.auditReference,
  });
  assertNoMutationError(error, '鎵ц瀵硅处');
}

export async function loadMyOperationsSubmissions(
  connectedWallet: string,
): Promise<MyOperationsSubmission[]> {
  const { client } = await requireIntakeSession(connectedWallet);
  const [taskResult, riskResult, reliefResult, proposalResult, discussionResult] = await Promise.all([
    client
      .from('task_submissions')
      .select('id,summary,status,reviewer_notes,created_at,updated_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('risk_reports')
      .select('id,project_identifier,review_status,reviewer_notes,created_at,updated_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('relief_applications')
      .select('id,incident_summary,status,reviewer_notes,created_at,updated_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('governance_proposal_submissions')
      .select('id,title,review_status,reviewer_notes,created_at,reviewed_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
    client
      .from('governance_discussions')
      .select('id,topic,moderation_status,reviewer_notes,created_at,updated_at')
      .order('created_at', { ascending: false })
      .limit(OPERATIONS_PUBLIC_RECORD_LIMIT),
  ]);

  assertNoQueryError(taskResult.error, '鎴戠殑浠诲姟鎻愪氦');
  assertNoQueryError(riskResult.error, '鎴戠殑椋庨櫓鎶ュ憡');
  assertNoQueryError(reliefResult.error, '鎴戠殑鏁戝姪鐢宠');
  assertNoQueryError(proposalResult.error, '鎴戠殑娌荤悊鎻愭');
  assertNoQueryError(discussionResult.error, '鎴戠殑娌荤悊璁ㄨ');

  return [
    ...(taskResult.data ?? []).map((row) => ({
      id: row.id,
      kind: 'task' as const,
      title: summarizePrivateText(row.summary, '浠诲姟鎴愭灉'),
      status: row.status,
      reviewerNotes: row.reviewer_notes,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    })),
    ...(riskResult.data ?? []).map((row) => ({
      id: row.id,
      kind: 'risk' as const,
      title: row.project_identifier,
      status: row.review_status,
      reviewerNotes: row.reviewer_notes,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    })),
    ...(reliefResult.data ?? []).map((row) => ({
      id: row.id,
      kind: 'relief' as const,
      title: summarizePrivateText(row.incident_summary, '鏁戝姪鐢宠'),
      status: row.status,
      reviewerNotes: row.reviewer_notes,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    })),
    ...(proposalResult.data ?? []).map((row) => ({
      id: row.id,
      kind: 'proposal' as const,
      title: row.title,
      status: row.review_status,
      reviewerNotes: row.reviewer_notes,
      createdAt: row.created_at,
      updatedAt: row.reviewed_at ?? row.created_at,
    })),
    ...(discussionResult.data ?? []).map((row) => ({
      id: row.id,
      kind: 'discussion' as const,
      title: row.topic,
      status: row.moderation_status,
      reviewerNotes: row.reviewer_notes,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    })),
  ]
    .sort((left, right) => Date.parse(right.createdAt) - Date.parse(left.createdAt))
    .slice(0, OPERATIONS_PUBLIC_RECORD_LIMIT);
}

async function requireIntakeSession(
  connectedWallet: string,
): Promise<{ client: SupabaseClient; user: User }> {
  if (!operationsBackendConfig.intakeEnabled) {
    throw new OperationsBackendError(
      operationsBackendConfig.reason ?? '绀惧尯鎻愪氦鍏ュ彛灏氭湭鍚敤',
    );
  }

  const client = getOperationsSupabase();
  if (!client) {
    throw new OperationsBackendError('杩愯惀鍚庣灏氭湭閰嶇疆');
  }

  const userResult = await client.auth.getUser();
  if (userResult.error || !userResult.data.user) {
    throw new OperationsBackendError('缂哄皯鏈夋晥鐨勯挶鍖呰璇佷細璇濓紝璇峰厛绛惧悕璁よ瘉');
  }

  try {
    assertWalletSessionMatch(userResult.data.user, connectedWallet);
  } catch (error) {
    throw new OperationsBackendError(
      error instanceof Error ? error.message : 'wallet-authenticated session mismatch',
    );
  }

  const [intakeResult, walletResult] = await Promise.all([
    client.rpc('is_operations_wallet_intake_enabled'),
    client.rpc('current_verified_solana_wallet'),
  ]);
  if (intakeResult.error || intakeResult.data !== true) {
    throw new OperationsBackendError('wallet intake gate is not enabled');
  }

  if (walletResult.error || walletResult.data !== connectedWallet) {
    throw new OperationsBackendError('鏁版嵁搴撴湭纭褰撳墠 Web3 閽卞寘韬唤锛屾彁浜ゅ凡涓');
  }

  return { client, user: userResult.data.user };
}
async function requireStaffSession(): Promise<{
  client: SupabaseClient;
  user: User;
  role: OperationsStaffRole;
}> {
  const client = getOperationsSupabase();
  if (!client) {
    throw new OperationsBackendError('operations backend is not configured');
  }

  const userResult = await client.auth.getUser();
  const user = userResult.data.user;
  if (userResult.error || !user) {
    throw new OperationsBackendError('current session has no operations staff access');
  }

  const access = await readMyOperationsAccess(client);
  if (access.status !== 'active' || !access.role) {
    throw new OperationsBackendError('current session has no operations staff access');
  }

  return { client, user, role: access.role };
}


function requirePublicReadClient(): SupabaseClient {
  if (!operationsBackendConfig.publicReadEnabled) {
    throw new OperationsBackendError(
      operationsBackendConfig.reason ?? '杩愯惀鍚庣灏氭湭閰嶇疆',
    );
  }

  const client = getOperationsSupabase();
  if (!client) {
    throw new OperationsBackendError('杩愯惀鍚庣灏氭湭閰嶇疆');
  }

  return client;
}

function assertNoQueryError(error: { message: string } | null, label: string): void {
  if (error) {
    throw new OperationsBackendError(`${label} could not be loaded`);
  }
}

async function readMyOperationsAccess(
  client: SupabaseClient,
): Promise<MyOperationsAccess> {
  const result = await client.rpc('get_my_operations_access_v1');
  if (result.error) {
    throw new OperationsBackendError('operations access status could not be loaded');
  }

  if (!Array.isArray(result.data) || result.data.length === 0) {
    return { role: null, status: null, expiresAt: null };
  }
  if (result.data.length !== 1) {
    throw new OperationsBackendError('operations access status returned an ambiguous result');
  }

  const row = result.data[0];
  if (typeof row !== 'object' || row === null || Array.isArray(row)) {
    throw new OperationsBackendError('operations access status returned an invalid record');
  }

  const record = row as Record<string, unknown>;
  const role = resolveOperationsStaffRole(record.role_name);
  const status = resolveOperationsAccessStatus(record.status);
  const expiresAt = record.expires_at === null
    ? null
    : typeof record.expires_at === 'string'
      ? record.expires_at
      : null;

  if (status === null || (record.role_name !== null && role === null)) {
    throw new OperationsBackendError('operations access status returned invalid values');
  }

  return { role, status, expiresAt };
}

function resolveOperationsAccessStatus(value: unknown): OperationsAccessStatus | null {
  return value === 'active' || value === 'revoked' || value === 'expired'
    ? value
    : null;
}

function assertNoMutationError(error: { message: string } | null, label: string): void {
  if (error) {
    throw new OperationsBackendError(`${label} was not submitted successfully`);
  }
}

function summarizePrivateText(value: string, fallback: string): string {
  const normalized = value.trim().replace(/\s+/g, ' ');
  if (!normalized) {
    return fallback;
  }

  return normalized.length > 80 ? `${normalized.slice(0, 77)}...` : normalized;
}

function mapTask(row: {
  id: string;
  title: string;
  summary: string;
  requirements: string;
  reward_budget_usdc: string | number | null;
  reward_source: string | null;
  status: CommunityTask['status'];
  submission_deadline: string | null;
  published_at: string;
}): CommunityTask {
  return {
    id: row.id,
    title: row.title,
    summary: row.summary,
    requirements: row.requirements,
    rewardBudgetUsdc: row.reward_budget_usdc === null ? null : String(row.reward_budget_usdc),
    rewardSource: row.reward_source,
    status: row.status,
    submissionDeadline: row.submission_deadline,
    publishedAt: row.published_at,
  };
}

function mapTaskResult(row: {
  id: string;
  task_id: string;
  task_title: string;
  result_summary: string;
  deliverable_url: string;
  wallet_address: string | null;
  review_reference: string;
  accepted_at: string;
  published_at: string;
}): PublicTaskResult {
  return {
    id: row.id,
    taskId: row.task_id,
    taskTitle: row.task_title,
    resultSummary: row.result_summary,
    deliverableUrl: row.deliverable_url,
    walletAddress: row.wallet_address,
    reviewReference: row.review_reference,
    acceptedAt: row.accepted_at,
    publishedAt: row.published_at,
  };
}

function mapRiskReport(row: {
  id: string;
  project_identifier: string;
  summary: string;
  reference_url: string | null;
  public_status: PublicRiskReport['publicStatus'];
  published_at: string;
}): PublicRiskReport {
  return {
    id: row.id,
    projectIdentifier: row.project_identifier,
    summary: row.summary,
    referenceUrl: row.reference_url,
    publicStatus: row.public_status,
    publishedAt: row.published_at,
  };
}

function mapReliefUpdate(row: {
  id: string;
  case_reference: string;
  title: string;
  summary: string;
  outcome: PublicReliefOutcome;
  published_at: string;
}): PublicReliefUpdate {
  return {
    id: row.id,
    caseReference: row.case_reference,
    title: row.title,
    summary: row.summary,
    outcome: row.outcome,
    publishedAt: row.published_at,
  };
}

function mapDiscussion(row: {
  id: string;
  topic: string;
  body: string;
  wallet_address: string | null;
  published_at: string;
}): PublicDiscussion {
  return {
    id: row.id,
    topic: row.topic,
    body: row.body,
    walletAddress: row.wallet_address,
    publishedAt: row.published_at,
  };
}

function mapProposal(row: {
  id: string;
  title: string;
  summary: string;
  proposal_kind: PublicGovernanceProposal['proposalKind'];
  public_source_reference: string | null;
  execution_required: boolean;
  execution_manifest_url: string | null;
  status: string;
  published_at: string;
}): PublicGovernanceProposal {
  return {
    id: row.id,
    title: row.title,
    summary: row.summary,
    proposalKind: row.proposal_kind,
    publicSourceReference: row.public_source_reference,
    executionRequired: row.execution_required,
    executionManifestUrl: row.execution_manifest_url,
    status: row.status,
    publishedAt: row.published_at,
  };
}

function mapDecision(
  row: {
    id: string;
    proposal_id: string;
    decision: GovernanceDecisionValue;
    rationale: string;
    execution_required: boolean;
    execution_reference: string | null;
    execution_manifest_sha256: string | null;
    decision_hash: string;
    finalization_reference: string;
    decided_at: string;
  },
  proposalTitle: string,
): PublicGovernanceDecision {
  return {
    id: row.id,
    proposalId: row.proposal_id,
    proposalTitle,
    decision: row.decision,
    rationale: row.rationale,
    executionRequired: row.execution_required,
    executionReference: row.execution_reference,
    executionManifestSha256: row.execution_manifest_sha256,
    decisionHash: row.decision_hash,
    finalizationReference: row.finalization_reference,
    decidedAt: row.decided_at,
  };
}

function mapTreasuryExecution(row: {
  intent_public_id: string;
  governance_decision_id: string;
  decision_hash: string;
  manifest_sha256: string;
  intent_hash: string;
  purpose_reference: string;
  asset_symbol: 'USDC';
  asset_decimals: 6;
  asset_mint: string;
  destination_wallet_display: string;
  amount_base_units: string | number;
  network: string;
  public_status: PublicTreasuryExecutionRecord['publicStatus'];
  external_execution_reference: string | null;
  prepared_at: string;
  authorized_at: string | null;
  reported_at: string | null;
  reconciled_at: string | null;
  reconciliation_reference: string | null;
}): PublicTreasuryExecutionRecord {
  return {
    intentPublicId: row.intent_public_id,
    governanceDecisionId: row.governance_decision_id,
    decisionHash: row.decision_hash,
    manifestSha256: row.manifest_sha256,
    intentHash: row.intent_hash,
    purposeReference: row.purpose_reference,
    assetSymbol: row.asset_symbol,
    assetDecimals: row.asset_decimals,
    assetMint: row.asset_mint,
    destinationWalletDisplay: row.destination_wallet_display,
    amountBaseUnits: String(row.amount_base_units),
    network: row.network,
    publicStatus: row.public_status,
    externalExecutionReference: row.external_execution_reference,
    preparedAt: row.prepared_at,
    authorizedAt: row.authorized_at,
    reportedAt: row.reported_at,
    reconciledAt: row.reconciled_at,
    reconciliationReference: row.reconciliation_reference,
  };
}
