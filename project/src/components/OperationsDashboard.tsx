import { useEffect, useMemo, useRef, useState, type FormEvent, type ReactNode } from 'react';
import {
  AlertTriangle,
  BadgeCheck,
  BookOpenCheck,
  CheckCircle2,
  ClipboardCheck,
  FileWarning,
  HandHeart,
  Landmark,
  Link2,
  Loader2,
  LockKeyhole,
  LogIn,
  LogOut,
  MessageSquareText,
  RefreshCw,
  Send,
  ShieldCheck,
  UserRoundCheck,
  Users,
  WalletCards,
} from 'lucide-react';
import TurnstileChallenge from './TurnstileChallenge';
import type {
  CommunityTaskPublicationInput,
  TaskSubmissionReviewInput,
} from '../features/operations/domain';
import {
  submitDiscussion,
  submitReliefApplication,
  submitRiskReport,
  submitTaskResult,
} from '../features/operations/repository';
import { resolveOperationsWalletAccess } from '../features/operations/walletAccess';
import type { OperationsWalletAuthState } from '../hooks/useOperationsWalletAuth';
import { useMyOperationsSubmissions } from '../hooks/useMyOperationsSubmissions';
import { useOperations } from '../hooks/useOperations';
import { useOperationsStaff } from '../hooks/useOperationsStaff';
import { useOperationsWalletAuth } from '../hooks/useOperationsWalletAuth';
import { operationsBackendConfig } from '../lib/operationsSupabase';

type OperationsTab = 'tasks' | 'risk' | 'relief' | 'discussion' | 'mine' | 'decisions' | 'staff';
type NoticeTone = 'success' | 'error';

interface SubmissionNotice {
  tone: NoticeTone;
  message: string;
}

const TABS: {
  key: OperationsTab;
  label: string;
  icon: typeof ClipboardCheck;
}[] = [
  { key: 'tasks', label: '社区任务', icon: ClipboardCheck },
  { key: 'risk', label: '风险与证据', icon: FileWarning },
  { key: 'relief', label: '救助申请', icon: HandHeart },
  { key: 'discussion', label: '治理讨论', icon: MessageSquareText },
  { key: 'mine', label: '我的提交', icon: UserRoundCheck },
  { key: 'decisions', label: '公开决定', icon: BookOpenCheck },
];

const STAFF_TAB = { key: 'staff' as const, label: '运营审核', icon: ShieldCheck };

const fieldClassName = [
  'w-full rounded border border-zinc-800 bg-zinc-950 px-3 py-2.5',
  'text-xs text-zinc-200 outline-none transition-colors',
  'placeholder:text-zinc-700 focus:border-cyan-400/50',
  'disabled:cursor-not-allowed disabled:opacity-50',
].join(' ');

export default function OperationsDashboard() {
  const [activeTab, setActiveTab] = useState<OperationsTab>('tasks');
  const operations = useOperations();
  const walletAuth = useOperationsWalletAuth();
  const walletAccess = resolveOperationsWalletAccess(
    walletAuth.status,
    walletAuth.authenticatedWallet,
    walletAuth.connectedWallet,
    walletAuth.intakeGateStatus,
  );
  const mySubmissions = useMyOperationsSubmissions(
    walletAccess.sessionVerified ? walletAuth.authenticatedWallet : null,
  );
  const staff = useOperationsStaff(walletAuth.operationsRole);
  const intakeReady = walletAccess.intakeEnabled;

  const counts = useMemo(() => ({
    tasks: operations.overview.tasks.length,
    risk: operations.overview.riskReports.length,
    relief: operations.overview.reliefUpdates.length,
    discussion: operations.overview.discussions.length,
    mine: mySubmissions.submissions.length,
    decisions: operations.overview.governanceDecisions.length,
    staff: staff.workspace.submissions.length,
  }), [mySubmissions.submissions.length, operations.overview, staff.workspace.submissions.length]);
  const visibleTabs = walletAuth.operationsRole ? [...TABS, STAFF_TAB] : TABS;

  return (
    <div className="select-text space-y-6">
      <section className="overflow-hidden rounded-xl border border-cyan-400/20 bg-cyan-400/[0.035]">
        <div className="border-b border-cyan-400/15 px-5 py-5 sm:px-6">
          <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <div className="flex flex-wrap items-center gap-2">
                <StatusBadge label="OFF-CHAIN OPERATIONS" tone="cyan" />
                <StatusBadge label="NO TREASURY AUTHORITY" tone="red" />
                <StatusBadge
                  label={operationsBackendConfig.publicReadEnabled ? 'PUBLIC READ CONFIGURED' : 'BACKEND UNCONFIGURED'}
                  tone={operationsBackendConfig.publicReadEnabled ? 'emerald' : 'yellow'}
                />
                <StatusBadge
                  label={walletAccess.sessionVerified ? 'WALLET SESSION VERIFIED' : 'WALLET SESSION REQUIRED'}
                  tone={walletAccess.sessionVerified ? 'emerald' : 'zinc'}
                />
                <StatusBadge
                  label={walletAuth.intakeGateStatus === 'enabled' ? 'INTAKE ENABLED' : 'INTAKE LOCKED'}
                  tone={walletAuth.intakeGateStatus === 'enabled' ? 'yellow' : 'zinc'}
                />
              </div>
              <h2 className="mt-4 text-2xl font-black text-zinc-100">社区运营、申请与公开决定</h2>
              <p className="mt-2 max-w-3xl text-xs leading-relaxed text-zinc-400">
                任务、举报、证据、救助申请和治理讨论保存在受 RLS 保护的运营数据库。
                链上程序继续只负责国库、分账和最终资金安全；数据库记录不能签名，也不能直接转账。
              </p>
            </div>
            <button
              type="button"
              onClick={() => void operations.refresh()}
              disabled={operations.status === 'loading' || !operationsBackendConfig.publicReadEnabled}
              className="inline-flex items-center justify-center gap-2 rounded border border-cyan-400/25 bg-cyan-400/5 px-3 py-2 text-[10px] font-black text-cyan-300 transition-colors hover:bg-cyan-400/10 disabled:cursor-not-allowed disabled:opacity-40"
            >
              <RefreshCw className={`h-3.5 w-3.5 ${operations.status === 'loading' ? 'animate-spin' : ''}`} />
              刷新公开记录
            </button>
          </div>
        </div>

        <div className="grid gap-px bg-zinc-800/80 sm:grid-cols-2 lg:grid-cols-4">
          <BoundaryStep
            step="STEP 1"
            title="社区提交"
            body="任务成果、风险证据、救助申请、治理讨论。"
            icon={Users}
            tone="cyan"
          />
          <BoundaryStep
            step="STEP 2"
            title="人工审核 / DAO 决定"
            body="净化敏感信息，公开依据与结果，保留审计记录。"
            icon={ShieldCheck}
            tone="emerald"
          />
          <BoundaryStep
            step="STEP 3"
            title="独立执行清单"
            body="资金决定必须绑定确定性 manifest 和唯一决定记录。"
            icon={ClipboardCheck}
            tone="yellow"
          />
          <BoundaryStep
            step="STEP 4"
            title="链上 / 多签付款"
            body="由独立签名流程执行，确认后再登记不可变 receipt。"
            icon={Landmark}
            tone="violet"
          />
        </div>
      </section>

      <BackendState
        status={operations.status}
        error={operations.error}
        lastLoadedAt={operations.lastLoadedAt}
      />

      <WalletAuthBoundary auth={walletAuth} />

      <section className="rounded-xl border border-zinc-800 bg-zinc-950/70">
        <div className="flex overflow-x-auto border-b border-zinc-800">
          {visibleTabs.map((tab) => {
            const Icon = tab.icon;
            const active = activeTab === tab.key;
            return (
              <button
                key={tab.key}
                type="button"
                onClick={() => setActiveTab(tab.key)}
                className={`flex min-w-max items-center gap-2 border-b-2 px-4 py-3 text-[11px] font-black transition-colors ${
                  active
                    ? 'border-cyan-400 bg-cyan-400/5 text-cyan-300'
                    : 'border-transparent text-zinc-600 hover:text-zinc-300'
                }`}
              >
                <Icon className="h-3.5 w-3.5" />
                {tab.label}
                {operations.status === 'ready' && (
                  <span className="rounded bg-zinc-900 px-1.5 py-0.5 text-[9px] text-zinc-500">
                    {counts[tab.key]}
                  </span>
                )}
              </button>
            );
          })}
        </div>

        <div className="p-4 sm:p-6">
          {activeTab === 'tasks' && (
            <TasksPanel
              tasks={operations.overview.tasks}
              taskResults={operations.overview.taskResults}
              loaded={operations.status === 'ready'}
              authenticatedWallet={intakeReady ? walletAuth.authenticatedWallet : null}
              onSubmitted={mySubmissions.refresh}
            />
          )}
          {activeTab === 'risk' && (
            <RiskPanel
              reports={operations.overview.riskReports}
              loaded={operations.status === 'ready'}
              authenticatedWallet={intakeReady ? walletAuth.authenticatedWallet : null}
              onSubmitted={mySubmissions.refresh}
            />
          )}
          {activeTab === 'relief' && (
            <ReliefPanel
              updates={operations.overview.reliefUpdates}
              loaded={operations.status === 'ready'}
              authenticatedWallet={intakeReady ? walletAuth.authenticatedWallet : null}
              onSubmitted={mySubmissions.refresh}
            />
          )}
          {activeTab === 'discussion' && (
            <DiscussionPanel
              discussions={operations.overview.discussions}
              loaded={operations.status === 'ready'}
              authenticatedWallet={intakeReady ? walletAuth.authenticatedWallet : null}
              onSubmitted={mySubmissions.refresh}
            />
          )}
          {activeTab === 'mine' && (
            <MySubmissionsPanel state={mySubmissions} auth={walletAuth} />
          )}
          {activeTab === 'decisions' && (
            <DecisionsPanel
              decisions={operations.overview.governanceDecisions}
              loaded={operations.status === 'ready'}
            />
          )}
          {activeTab === 'staff' && walletAuth.operationsRole && (
            <StaffOperationsPanel
              role={walletAuth.operationsRole}
              state={staff}
              onPublicDataChanged={operations.refresh}
            />
          )}
        </div>
      </section>

      <section className="grid gap-4 lg:grid-cols-2">
        <div className="rounded-xl border border-emerald-400/20 bg-emerald-400/5 p-5">
          <h3 className="flex items-center gap-2 text-sm font-black text-emerald-200">
            <BadgeCheck className="h-4 w-4" />
            数据库可以做什么
          </h3>
          <ul className="mt-4 space-y-2 text-[11px] leading-relaxed text-zinc-400">
            <li>• 接收与追踪运营申请，执行字段长度、状态与权限约束。</li>
            <li>• 将私有原始记录与净化后的公开记录分表保存。</li>
            <li>• 记录治理决定、执行 manifest 与已确认交易 receipt。</li>
            <li>• 允许公众读取明确标记为公开的数据。</li>
          </ul>
        </div>
        <div className="rounded-xl border border-red-400/20 bg-red-400/5 p-5">
          <h3 className="flex items-center gap-2 text-sm font-black text-red-200">
            <LockKeyhole className="h-4 w-4" />
            数据库绝不能做什么
          </h3>
          <ul className="mt-4 space-y-2 text-[11px] leading-relaxed text-zinc-400">
            <li>• 不保存任何 Solana 私钥、seed phrase、upgrade authority key。</li>
            <li>• 不根据 accepted / approved 状态自动发放 USDC。</li>
            <li>• 不把匿名会话或连接钱包当成签名验证。</li>
            <li>• 不绕过国库池规则、多签、Security Layer 或人工复核。</li>
          </ul>
        </div>
      </section>
    </div>
  );
}

function TasksPanel({
  tasks,
  taskResults,
  loaded,
  authenticatedWallet,
  onSubmitted,
}: {
  tasks: ReturnType<typeof useOperations>['overview']['tasks'];
  taskResults: ReturnType<typeof useOperations>['overview']['taskResults'];
  loaded: boolean;
  authenticatedWallet: string | null;
  onSubmitted: () => Promise<void>;
}) {
  const [form, setForm] = useState({
    taskId: '',
    summary: '',
    deliverableUrl: '',
    walletAddress: authenticatedWallet ?? '',
    publicResultConsent: false,
    publicWalletConsent: false,
  });
  const submission = useSubmission();

  usePrefillWallet(authenticatedWallet, (walletAddress) => {
    setForm((current) => ({ ...current, walletAddress }));
  });

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    const succeeded = await submission.run(
      () => submitTaskResult(form),
      '任务成果已进入私有审核队列；这不是付款承诺。',
      () => setForm((current) => ({
        ...current,
        summary: '',
        deliverableUrl: '',
        publicResultConsent: false,
        publicWalletConsent: false,
      })),
    );
    if (succeeded) {
      await onSubmitted();
    }
  }

  return (
    <TwoColumnPanel>
      <div className="space-y-6">
        <PublicList
          title="公开社区任务"
          icon={ClipboardCheck}
          loaded={loaded}
          emptyText="当前没有已发布的开放任务。未使用示例任务填充。"
        >
          {tasks.map((task) => (
            <article key={task.id} className="rounded border border-zinc-800 bg-zinc-950 p-4">
            <div className="flex items-start justify-between gap-3">
              <div>
                <h4 className="text-sm font-black text-zinc-200">{task.title}</h4>
                <p className="mt-2 text-[11px] leading-relaxed text-zinc-500">{task.summary}</p>
              </div>
              <StatusBadge label={task.status.replace('_', ' ')} tone="cyan" />
            </div>
            <div className="mt-3 grid gap-2 text-[10px] text-zinc-600 sm:grid-cols-2">
              <span>预算上限：{task.rewardBudgetUsdc === null ? '未承诺' : `${task.rewardBudgetUsdc} USDC`}</span>
              <span>截止：{formatDate(task.submissionDeadline)}</span>
            </div>
            <details className="mt-3 rounded border border-zinc-900 px-3 py-2 text-[10px] text-zinc-500">
              <summary className="cursor-pointer font-black text-zinc-400">查看验收要求</summary>
              <p className="mt-2 whitespace-pre-wrap leading-relaxed">{task.requirements}</p>
            </details>
            </article>
          ))}
        </PublicList>
        <PublicList
          title="经同意脱敏公开的任务成果"
          icon={BadgeCheck}
          loaded={loaded}
          emptyText="当前没有经审核并获得公开同意的任务成果。"
        >
          {taskResults.map((result) => (
            <article key={result.id} className="rounded border border-emerald-400/15 bg-emerald-400/[0.025] p-4">
              <h4 className="text-sm font-black text-zinc-200">{result.taskTitle}</h4>
              <p className="mt-2 whitespace-pre-wrap text-[11px] leading-relaxed text-zinc-500">
                {result.resultSummary}
              </p>
              <div className="mt-3 flex flex-wrap items-center justify-between gap-2 text-[10px] text-zinc-600">
                <SafeLink href={result.deliverableUrl} label="公开成果" />
                <span>{result.walletAddress ? `贡献钱包：${shorten(result.walletAddress)}` : '贡献钱包未公开'}</span>
              </div>
              <p className="mt-2 text-[9px] text-zinc-700">
                审核引用：{result.reviewReference} · accepted 不代表已付款
              </p>
            </article>
          ))}
        </PublicList>
      </div>

      <SubmissionForm
        title="提交任务成果"
        description="提交后仅进入审核。accepted 不代表已付款；付款需另行形成治理决定与执行记录。"
        busy={submission.busy}
        notice={submission.notice}
        onSubmit={handleSubmit}
        disabled={!authenticatedWallet || tasks.length === 0}
      >
        <FormField label="任务">
          <select
            value={form.taskId}
            onChange={(event) => setForm({ ...form, taskId: event.target.value })}
            className={fieldClassName}
            required
          >
            <option value="">选择已发布任务</option>
            {tasks.filter((task) => task.status === 'open').map((task) => (
              <option key={task.id} value={task.id}>{task.title}</option>
            ))}
          </select>
        </FormField>
        <FormField label="成果说明（20–5000 字）">
          <textarea
            value={form.summary}
            onChange={(event) => setForm({ ...form, summary: event.target.value })}
            className={`${fieldClassName} min-h-32 resize-y`}
            placeholder="说明完成内容、验证方法和已知限制"
            required
          />
        </FormField>
        <FormField label="成果 HTTPS 链接">
          <input
            type="url"
            value={form.deliverableUrl}
            onChange={(event) => setForm({ ...form, deliverableUrl: event.target.value })}
            className={fieldClassName}
            placeholder="https://..."
            required
          />
        </FormField>
        <WalletField
          value={form.walletAddress}
          label="已认证提交钱包（付款前仍需独立复核）"
        />
        <label className="flex items-start gap-2 text-[10px] leading-relaxed text-zinc-500">
          <input
            type="checkbox"
            checked={form.publicResultConsent}
            onChange={(event) => setForm({
              ...form,
              publicResultConsent: event.target.checked,
              publicWalletConsent: event.target.checked ? form.publicWalletConsent : false,
            })}
            className="mt-0.5"
          />
          我同意：仅在成果被接受后，审核者可公开脱敏摘要与成果 HTTPS 链接；原始私有提交不会公开。
        </label>
        <label className="flex items-start gap-2 text-[10px] leading-relaxed text-zinc-500">
          <input
            type="checkbox"
            checked={form.publicWalletConsent}
            disabled={!form.publicResultConsent}
            onChange={(event) => setForm({ ...form, publicWalletConsent: event.target.checked })}
            className="mt-0.5"
          />
          我另行同意在脱敏公开成果中展示当前钱包地址（可选；不影响审核）。
        </label>
      </SubmissionForm>
    </TwoColumnPanel>
  );
}

function RiskPanel({
  reports,
  loaded,
  authenticatedWallet,
  onSubmitted,
}: {
  reports: ReturnType<typeof useOperations>['overview']['riskReports'];
  loaded: boolean;
  authenticatedWallet: string | null;
  onSubmitted: () => Promise<void>;
}) {
  const [form, setForm] = useState({
    projectIdentifier: '',
    summary: '',
    referenceUrl: '',
    walletAddress: authenticatedWallet ?? '',
  });
  const submission = useSubmission();

  usePrefillWallet(authenticatedWallet, (walletAddress) => {
    setForm((current) => ({ ...current, walletAddress }));
  });

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    const succeeded = await submission.run(
      () => submitRiskReport(form),
      '风险报告已私密提交；公开前必须人工核验并净化敏感信息。',
      () => setForm((current) => ({
        ...current,
        projectIdentifier: '',
        summary: '',
        referenceUrl: '',
      })),
    );
    if (succeeded) {
      await onSubmitted();
    }
  }

  return (
    <TwoColumnPanel>
      <PublicList
        title="已核验并公开的风险记录"
        icon={FileWarning}
        loaded={loaded}
        emptyText="当前没有经审核公开的风险记录。原始举报不会自动公开。"
      >
        {reports.map((report) => (
          <article key={report.id} className="rounded border border-red-400/15 bg-red-400/[0.025] p-4">
            <div className="flex items-start justify-between gap-3">
              <h4 className="text-sm font-black text-zinc-200">{report.projectIdentifier}</h4>
              <StatusBadge label={report.publicStatus} tone={report.publicStatus === 'dismissed' ? 'zinc' : 'red'} />
            </div>
            <p className="mt-3 whitespace-pre-wrap text-[11px] leading-relaxed text-zinc-500">{report.summary}</p>
            <div className="mt-3 flex items-center justify-between gap-3 text-[10px] text-zinc-600">
              <span>{formatDate(report.publishedAt)}</span>
              {report.referenceUrl && <SafeLink href={report.referenceUrl} label="公开依据" />}
            </div>
          </article>
        ))}
      </PublicList>

      <SubmissionForm
        title="私密提交风险报告"
        description="举报只是线索，不等于诈骗定性。公开记录必须经过证据复核和明确治理依据。"
        busy={submission.busy}
        notice={submission.notice}
        onSubmit={handleSubmit}
        disabled={!authenticatedWallet}
      >
        <FormField label="项目名称 / Mint / Program / 交易标识">
          <input
            value={form.projectIdentifier}
            onChange={(event) => setForm({ ...form, projectIdentifier: event.target.value })}
            className={fieldClassName}
            required
          />
        </FormField>
        <FormField label="风险说明（30–5000 字）">
          <textarea
            value={form.summary}
            onChange={(event) => setForm({ ...form, summary: event.target.value })}
            className={`${fieldClassName} min-h-36 resize-y`}
            placeholder="区分事实、推断与尚待验证的部分"
            required
          />
        </FormField>
        <FormField label="证据 HTTPS 链接">
          <input
            type="url"
            value={form.referenceUrl}
            onChange={(event) => setForm({ ...form, referenceUrl: event.target.value })}
            className={fieldClassName}
            placeholder="https://..."
            required
          />
        </FormField>
        <WalletField
          value={form.walletAddress}
          label="已认证提交钱包"
        />
      </SubmissionForm>
    </TwoColumnPanel>
  );
}

function ReliefPanel({
  updates,
  loaded,
  authenticatedWallet,
  onSubmitted,
}: {
  updates: ReturnType<typeof useOperations>['overview']['reliefUpdates'];
  loaded: boolean;
  authenticatedWallet: string | null;
  onSubmitted: () => Promise<void>;
}) {
  const [form, setForm] = useState({
    incidentSummary: '',
    requestedAmountUsdc: '',
    evidenceUrl: '',
    walletAddress: authenticatedWallet ?? '',
  });
  const submission = useSubmission();

  usePrefillWallet(authenticatedWallet, (walletAddress) => {
    setForm((current) => ({ ...current, walletAddress }));
  });

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    const succeeded = await submission.run(
      () => submitReliefApplication(form),
      '救助申请已私密提交；申请、批准与实际赔付是三个独立阶段。',
      () => setForm((current) => ({
        ...current,
        incidentSummary: '',
        requestedAmountUsdc: '',
        evidenceUrl: '',
      })),
    );
    if (succeeded) {
      await onSubmitted();
    }
  }

  return (
    <TwoColumnPanel>
      <PublicList
        title="匿名化救助进度"
        icon={HandHeart}
        loaded={loaded}
        emptyText="当前没有已发布的匿名化救助进度。私有申请不会在这里直接显示。"
      >
        {updates.map((update) => (
          <article key={update.id} className="rounded border border-emerald-400/15 bg-emerald-400/[0.025] p-4">
            <div className="flex items-start justify-between gap-3">
              <div>
                <span className="text-[9px] font-black uppercase tracking-widest text-zinc-600">
                  {update.caseReference}
                </span>
                <h4 className="mt-1 text-sm font-black text-zinc-200">{update.title}</h4>
              </div>
              <StatusBadge label={update.outcome} tone="emerald" />
            </div>
            <p className="mt-3 whitespace-pre-wrap text-[11px] leading-relaxed text-zinc-500">{update.summary}</p>
            <p className="mt-3 text-[10px] text-zinc-600">{formatDate(update.publishedAt)}</p>
          </article>
        ))}
      </PublicList>

      <SubmissionForm
        title="私密提交救助申请"
        description="不得提交 seed phrase 或私钥。批准仍需验证身份、冻结收款地址并完成独立资金执行。"
        busy={submission.busy}
        notice={submission.notice}
        onSubmit={handleSubmit}
        disabled={!authenticatedWallet}
      >
        <FormField label="事件说明（50–8000 字）">
          <textarea
            value={form.incidentSummary}
            onChange={(event) => setForm({ ...form, incidentSummary: event.target.value })}
            className={`${fieldClassName} min-h-40 resize-y`}
            placeholder="时间、链上交易、损失原因、已采取措施"
            required
          />
        </FormField>
        <FormField label="申请金额（USDC，最多 6 位小数）">
          <input
            inputMode="decimal"
            value={form.requestedAmountUsdc}
            onChange={(event) => setForm({ ...form, requestedAmountUsdc: event.target.value })}
            className={fieldClassName}
            placeholder="0.00"
            required
          />
        </FormField>
        <FormField label="证据 HTTPS 链接">
          <input
            type="url"
            value={form.evidenceUrl}
            onChange={(event) => setForm({ ...form, evidenceUrl: event.target.value })}
            className={fieldClassName}
            placeholder="https://..."
            required
          />
        </FormField>
        <WalletField
          value={form.walletAddress}
          label="已认证申请钱包（赔付前仍需冻结与复核）"
        />
      </SubmissionForm>
    </TwoColumnPanel>
  );
}

function DiscussionPanel({
  discussions,
  loaded,
  authenticatedWallet,
  onSubmitted,
}: {
  discussions: ReturnType<typeof useOperations>['overview']['discussions'];
  loaded: boolean;
  authenticatedWallet: string | null;
  onSubmitted: () => Promise<void>;
}) {
  const [form, setForm] = useState({
    topic: '',
    body: '',
    walletAddress: authenticatedWallet ?? '',
  });
  const submission = useSubmission();

  usePrefillWallet(authenticatedWallet, (walletAddress) => {
    setForm((current) => ({ ...current, walletAddress }));
  });

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    const succeeded = await submission.run(
      () => submitDiscussion(form),
      '讨论已进入审核队列；通过后才会出现在公开页面。',
      () => setForm((current) => ({ ...current, topic: '', body: '' })),
    );
    if (succeeded) {
      await onSubmitted();
    }
  }

  return (
    <TwoColumnPanel>
      <PublicList
        title="已审核公开讨论"
        icon={MessageSquareText}
        loaded={loaded}
        emptyText="当前没有已审核公开的讨论。待审核内容不会提前显示。"
      >
        {discussions.map((discussion) => (
          <article key={discussion.id} className="rounded border border-cyan-400/15 bg-cyan-400/[0.025] p-4">
            <h4 className="text-sm font-black text-zinc-200">{discussion.topic}</h4>
            <p className="mt-3 whitespace-pre-wrap text-[11px] leading-relaxed text-zinc-500">{discussion.body}</p>
            <div className="mt-3 flex items-center justify-between gap-3 text-[10px] text-zinc-600">
              <span>{discussion.walletAddress ? shorten(discussion.walletAddress) : '未提供钱包'}</span>
              <span>{formatDate(discussion.publishedAt)}</span>
            </div>
          </article>
        ))}
      </PublicList>

      <SubmissionForm
        title="提交治理讨论"
        description="当前是讨论与意见收集层，不是链上投票。公开前执行内容审核。"
        busy={submission.busy}
        notice={submission.notice}
        onSubmit={handleSubmit}
        disabled={!authenticatedWallet}
      >
        <FormField label="讨论主题">
          <input
            value={form.topic}
            onChange={(event) => setForm({ ...form, topic: event.target.value })}
            className={fieldClassName}
            required
          />
        </FormField>
        <FormField label="讨论内容（20–5000 字）">
          <textarea
            value={form.body}
            onChange={(event) => setForm({ ...form, body: event.target.value })}
            className={`${fieldClassName} min-h-40 resize-y`}
            required
          />
        </FormField>
        <WalletField
          value={form.walletAddress}
          label="已认证发言钱包"
        />
      </SubmissionForm>
    </TwoColumnPanel>
  );
}

function WalletAuthBoundary({ auth }: { auth: OperationsWalletAuthState }) {
  const [turnstileToken, setTurnstileToken] = useState<string | null>(null);
  const [turnstileError, setTurnstileError] = useState<string | null>(null);
  const [turnstileResetKey, setTurnstileResetKey] = useState(0);
  const canSignIn = operationsBackendConfig.intakeEnabled
    && Boolean(auth.connectedWallet)
    && Boolean(turnstileToken)
    && !['checking', 'signing-in'].includes(auth.status);
  const access = resolveOperationsWalletAccess(
    auth.status,
    auth.authenticatedWallet,
    auth.connectedWallet,
    auth.intakeGateStatus,
  );
  const authenticated = access.sessionVerified;
  const showChallenge = operationsBackendConfig.intakeEnabled
    && !authenticated
    && Boolean(operationsBackendConfig.turnstileSiteKey);

  const handleSignIn = async () => {
    if (!turnstileToken) {
      setTurnstileError('请先完成 Turnstile 安全验证');
      return;
    }

    const singleUseToken = turnstileToken;
    setTurnstileToken(null);
    try {
      await auth.signIn(singleUseToken);
    } finally {
      setTurnstileResetKey((value) => value + 1);
    }
  };

  return (
    <section className={`rounded-xl border p-5 ${
      authenticated
        ? 'border-emerald-400/25 bg-emerald-400/5'
        : 'border-yellow-400/20 bg-yellow-400/[0.035]'
    }`}>
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h3 className={`flex items-center gap-2 text-sm font-black ${
            authenticated ? 'text-emerald-200' : 'text-yellow-200'
          }`}>
            {authenticated
              ? <UserRoundCheck className="h-4 w-4" />
              : <LockKeyhole className="h-4 w-4" />}
            {authenticated ? 'Solana Web3 钱包会话已验证' : '提交前需要钱包签名认证'}
          </h3>
          <div className="mt-3 grid gap-1 text-[10px] leading-relaxed text-zinc-500">
            <span>当前连接：{auth.connectedWallet ? shorten(auth.connectedWallet) : '未连接钱包'}</span>
            <span>认证会话：{auth.authenticatedWallet ? shorten(auth.authenticatedWallet) : '尚未建立'}</span>
            <span>数据库写入总闸门：{formatIntakeGateStatus(auth.intakeGateStatus)}</span>
            <span>签名只建立 Supabase 会话，不创建 Solana 交易、不授权代币、不移动资金。</span>
          </div>
          {authenticated && auth.intakeGateStatus === 'disabled' && (
            <p className="mt-3 text-[10px] leading-relaxed text-yellow-300">
              钱包会话已验证；数据库总闸门仍关闭。你可以查看自己的历史提交，但所有新提交继续锁定。
            </p>
          )}
          {auth.error && (
            <p className="mt-3 text-[10px] leading-relaxed text-red-300">{auth.error}</p>
          )}
          {auth.intakeGateError && (
            <p className="mt-2 text-[10px] leading-relaxed text-red-300">{auth.intakeGateError}</p>
          )}
          {turnstileError && (
            <p className="mt-2 text-[10px] leading-relaxed text-red-300">{turnstileError}</p>
          )}
        </div>
        <div className="flex w-full max-w-sm flex-col gap-3 sm:w-auto">
          {showChallenge && operationsBackendConfig.turnstileSiteKey && (
            <TurnstileChallenge
              siteKey={operationsBackendConfig.turnstileSiteKey}
              resetKey={turnstileResetKey}
              onToken={setTurnstileToken}
              onError={setTurnstileError}
            />
          )}
          <div className="flex flex-wrap justify-end gap-2">
            {auth.authenticatedWallet && (
              <button
                type="button"
                onClick={() => void auth.signOut()}
                className="inline-flex items-center gap-2 rounded border border-zinc-700 bg-zinc-900 px-3 py-2 text-[10px] font-black text-zinc-300 hover:border-zinc-600"
              >
                <LogOut className="h-3.5 w-3.5" />
                退出运营会话
              </button>
            )}
            {!authenticated && (
              <button
                type="button"
                onClick={() => void handleSignIn()}
                disabled={!canSignIn}
                className="inline-flex items-center gap-2 rounded border border-emerald-400/30 bg-emerald-400/10 px-3 py-2 text-[10px] font-black text-emerald-200 hover:bg-emerald-400/15 disabled:cursor-not-allowed disabled:opacity-40"
              >
                {auth.status === 'checking' || auth.status === 'signing-in'
                  ? <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  : <LogIn className="h-3.5 w-3.5" />}
                {auth.status === 'signing-in' ? '等待钱包签名…' : '签名认证钱包'}
              </button>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}

function MySubmissionsPanel({
  state,
  auth,
}: {
  state: ReturnType<typeof useMyOperationsSubmissions>;
  auth: OperationsWalletAuthState;
}) {
  const kindLabels = {
    task: '任务成果',
    risk: '风险报告',
    relief: '救助申请',
    discussion: '治理讨论',
  } as const;

  if (auth.status !== 'authenticated') {
    return (
      <EmptyState text="签名认证当前 Solana 钱包后，才能读取只属于该 Auth 用户的私有提交状态。" />
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-col gap-3 rounded border border-cyan-400/15 bg-cyan-400/[0.025] p-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h3 className="text-sm font-black text-zinc-200">我的私有提交状态</h3>
          <p className="mt-1 text-[10px] leading-relaxed text-zinc-600">
            这些记录由 RLS 按 Auth 用户隔离，不会自动公开，也不代表付款承诺。
          </p>
        </div>
        <button
          type="button"
          onClick={() => void state.refresh()}
          disabled={state.status === 'loading'}
          className="inline-flex items-center justify-center gap-2 rounded border border-cyan-400/25 bg-cyan-400/5 px-3 py-2 text-[10px] font-black text-cyan-300 disabled:opacity-40"
        >
          <RefreshCw className={`h-3.5 w-3.5 ${state.status === 'loading' ? 'animate-spin' : ''}`} />
          刷新我的提交
        </button>
      </div>

      {state.status === 'loading' && <LoadingRows />}
      {state.status === 'error' && (
        <div className="rounded border border-red-400/20 bg-red-400/5 p-4 text-[10px] text-red-200">
          {state.error}
        </div>
      )}
      {state.status === 'ready' && state.submissions.length === 0 && (
        <EmptyState text="当前认证钱包还没有提交记录。" />
      )}
      <div className="grid gap-3 lg:grid-cols-2">
        {state.submissions.map((submission) => (
          <article key={`${submission.kind}:${submission.id}`} className="rounded border border-zinc-800 bg-zinc-950 p-4">
            <div className="flex items-start justify-between gap-3">
              <div>
                <span className="text-[9px] font-black uppercase tracking-widest text-cyan-500">
                  {kindLabels[submission.kind]}
                </span>
                <h4 className="mt-2 text-xs font-black leading-relaxed text-zinc-200">
                  {submission.title}
                </h4>
              </div>
              <StatusBadge
                label={submission.status.replace(/_/g, ' ')}
                tone={['accepted', 'approved', 'published', 'resolved', 'paid'].includes(submission.status)
                  ? 'emerald'
                  : ['rejected', 'dismissed', 'cancelled'].includes(submission.status)
                    ? 'red'
                    : 'yellow'}
              />
            </div>
            {submission.reviewerNotes && (
              <div className="mt-3 rounded border border-zinc-900 bg-zinc-900/30 p-3 text-[10px] leading-relaxed text-zinc-500">
                审核备注：{submission.reviewerNotes}
              </div>
            )}
            <div className="mt-3 flex flex-wrap justify-between gap-2 text-[9px] text-zinc-700">
              <span>ID: {shorten(submission.id)}</span>
              <span>更新：{formatDate(submission.updatedAt)}</span>
            </div>
          </article>
        ))}
      </div>
    </div>
  );
}

function DecisionsPanel({
  decisions,
  loaded,
}: {
  decisions: ReturnType<typeof useOperations>['overview']['governanceDecisions'];
  loaded: boolean;
}) {
  return (
    <div className="space-y-4">
      <div className="rounded border border-yellow-400/20 bg-yellow-400/5 p-4 text-[11px] leading-relaxed text-yellow-100/70">
        <strong className="text-yellow-200">决定 ≠ 付款。</strong>
        {' '}只有独立执行 manifest、授权签名、链上确认和不可变 receipt 全部匹配后，才能将资金动作标记为完成。
      </div>
      {!loaded && <LoadingRows />}
      {loaded && decisions.length === 0 && (
        <EmptyState text="当前没有已发布的不可变治理决定。未生成模拟决定。" />
      )}
      <div className="grid gap-3 lg:grid-cols-2">
        {decisions.map((decision) => (
          <article key={decision.id} className="rounded border border-zinc-800 bg-zinc-950 p-4">
            <div className="flex items-start justify-between gap-3">
              <div>
                <span className="text-[9px] font-black uppercase tracking-widest text-zinc-600">
                  {shorten(decision.proposalId)}
                </span>
                <h4 className="mt-1 text-sm font-black text-zinc-200">{decision.proposalTitle}</h4>
              </div>
              <StatusBadge
                label={decision.decision}
                tone={decision.decision === 'approved' ? 'emerald' : 'red'}
              />
            </div>
            <p className="mt-3 whitespace-pre-wrap text-[11px] leading-relaxed text-zinc-500">{decision.rationale}</p>
            <div className="mt-4 grid gap-2 rounded border border-zinc-900 p-3 text-[10px] text-zinc-600">
              <span>需要资金执行：{decision.executionRequired ? '是' : '否'}</span>
              <span>执行引用：{decision.executionReference ?? '无'}</span>
              <span>决定时间：{formatDate(decision.decidedAt)}</span>
            </div>
          </article>
        ))}
      </div>
    </div>
  );
}

function StaffOperationsPanel({
  role,
  state,
  onPublicDataChanged,
}: {
  role: NonNullable<OperationsWalletAuthState['operationsRole']>;
  state: ReturnType<typeof useOperationsStaff>;
  onPublicDataChanged: () => Promise<void>;
}) {
  const [taskForm, setTaskForm] = useState<CommunityTaskPublicationInput>({
    title: '',
    summary: '',
    requirements: '',
    rewardBudgetUsdc: '',
    rewardSource: 'none',
    submissionDeadline: '',
    auditReference: '',
  });
  const [reviewForm, setReviewForm] = useState<TaskSubmissionReviewInput>({
    submissionId: '',
    decision: 'rejected',
    reviewerNotes: '',
    publicResultSummary: '',
    publicDeliverableUrl: '',
    auditReference: '',
  });
  const taskSubmission = useSubmission();
  const reviewSubmission = useSubmission();
  const selected = state.workspace.submissions.find((item) => item.id === reviewForm.submissionId);

  async function handlePublish(event: FormEvent) {
    event.preventDefault();
    const succeeded = await taskSubmission.run(
      async () => { await state.publishTask(taskForm); },
      '社区任务已通过受控 RPC 发布并写入不可变审计事件。',
      () => setTaskForm({
        title: '',
        summary: '',
        requirements: '',
        rewardBudgetUsdc: '',
        rewardSource: 'none',
        submissionDeadline: '',
        auditReference: '',
      }),
    );
    if (succeeded) {
      await onPublicDataChanged();
    }
  }

  async function handleReview(event: FormEvent) {
    event.preventDefault();
    const succeeded = await reviewSubmission.run(
      () => state.reviewSubmission(reviewForm),
      reviewForm.decision === 'accepted'
        ? '成果已接受；仅经贡献者同意的脱敏结果已公开。accepted 不代表已付款。'
        : '成果已拒绝并写入不可变审计事件；没有公开成果。',
      () => setReviewForm({
        submissionId: '',
        decision: 'rejected',
        reviewerNotes: '',
        publicResultSummary: '',
        publicDeliverableUrl: '',
        auditReference: '',
      }),
    );
    if (succeeded) {
      await onPublicDataChanged();
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-3 rounded border border-violet-400/20 bg-violet-400/5 p-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h3 className="text-sm font-black text-violet-200">受控运营工作台</h3>
          <p className="mt-1 text-[10px] text-zinc-500">
            当前角色：{role}。所有发布与审核走 security-definer RPC、行锁和不可变审计事件；不触发付款。
          </p>
        </div>
        <button
          type="button"
          onClick={() => void state.refresh()}
          disabled={state.loading}
          className="inline-flex items-center justify-center gap-2 rounded border border-violet-400/25 px-3 py-2 text-[10px] font-black text-violet-200 disabled:opacity-40"
        >
          <RefreshCw className={`h-3.5 w-3.5 ${state.loading ? 'animate-spin' : ''}`} />
          刷新审核队列
        </button>
      </div>

      {state.error && (
        <div className="rounded border border-red-400/20 bg-red-400/5 p-3 text-[10px] text-red-200">
          {state.error}
        </div>
      )}

      <div className="grid gap-6 xl:grid-cols-2">
        {role !== 'reviewer' && (
          <form onSubmit={handlePublish} className="space-y-3 rounded border border-zinc-800 bg-zinc-950 p-4">
            <h4 className="text-sm font-black text-zinc-200">发布社区任务</h4>
            <FormField label="任务标题（4–160 字）">
              <input className={fieldClassName} value={taskForm.title} onChange={(event) => setTaskForm({ ...taskForm, title: event.target.value })} required />
            </FormField>
            <FormField label="公开摘要（20–3000 字）">
              <textarea className={`${fieldClassName} min-h-24`} value={taskForm.summary} onChange={(event) => setTaskForm({ ...taskForm, summary: event.target.value })} required />
            </FormField>
            <FormField label="验收要求（20–5000 字）">
              <textarea className={`${fieldClassName} min-h-28`} value={taskForm.requirements} onChange={(event) => setTaskForm({ ...taskForm, requirements: event.target.value })} required />
            </FormField>
            <div className="grid gap-3 sm:grid-cols-2">
              <FormField label="预算上限 USDC（可空，0–1B）">
                <input className={fieldClassName} value={taskForm.rewardBudgetUsdc} onChange={(event) => setTaskForm({ ...taskForm, rewardBudgetUsdc: event.target.value })} inputMode="decimal" />
              </FormField>
              <FormField label="奖励来源">
                <select className={fieldClassName} value={taskForm.rewardSource} onChange={(event) => setTaskForm({ ...taskForm, rewardSource: event.target.value as CommunityTaskPublicationInput['rewardSource'] })}>
                  <option value="none">none / 尚无资金</option>
                  <option value="builders_pool">builders_pool</option>
                  <option value="grant">grant</option>
                  <option value="sponsor">sponsor</option>
                </select>
              </FormField>
            </div>
            <FormField label="提交截止时间（可空）">
              <input type="datetime-local" className={fieldClassName} value={taskForm.submissionDeadline} onChange={(event) => setTaskForm({ ...taskForm, submissionDeadline: event.target.value })} />
            </FormField>
            <FormField label="唯一审计引用（10–180 字）">
              <input className={fieldClassName} value={taskForm.auditReference} onChange={(event) => setTaskForm({ ...taskForm, auditReference: event.target.value })} required />
            </FormField>
            <button type="submit" disabled={state.submitting} className="w-full rounded border border-violet-400/30 bg-violet-400/10 px-4 py-2.5 text-xs font-black text-violet-200 disabled:opacity-40">
              发布任务并记录审计事件
            </button>
            {taskSubmission.notice && <InlineNotice notice={taskSubmission.notice} />}
          </form>
        )}

        <form onSubmit={handleReview} className="space-y-3 rounded border border-zinc-800 bg-zinc-950 p-4">
          <h4 className="text-sm font-black text-zinc-200">审核任务成果</h4>
          <FormField label="待审核提交">
            <select className={fieldClassName} value={reviewForm.submissionId} onChange={(event) => setReviewForm({ ...reviewForm, submissionId: event.target.value })} required>
              <option value="">选择私有提交</option>
              {state.workspace.submissions.map((submission) => (
                <option key={submission.id} value={submission.id}>{submission.taskTitle} · {shorten(submission.id)}</option>
              ))}
            </select>
          </FormField>
          {selected && (
            <div className="rounded border border-zinc-900 bg-zinc-900/30 p-3 text-[10px] leading-relaxed text-zinc-500">
              <p className="whitespace-pre-wrap">{selected.summary}</p>
              <div className="mt-2 flex flex-wrap gap-3">
                <SafeLink href={selected.deliverableUrl} label="私有提交成果" />
                <span>公开成果同意：{selected.publicResultConsent ? '是' : '否'}</span>
                <span>公开钱包同意：{selected.publicWalletConsent ? '是' : '否'}</span>
              </div>
            </div>
          )}
          <FormField label="审核决定">
            <select className={fieldClassName} value={reviewForm.decision} onChange={(event) => setReviewForm({ ...reviewForm, decision: event.target.value as 'accepted' | 'rejected' })}>
              <option value="rejected">rejected</option>
              <option value="accepted" disabled={!selected?.publicResultConsent}>accepted（需贡献者公开成果同意）</option>
            </select>
          </FormField>
          <FormField label="审核说明（必填）">
            <textarea className={`${fieldClassName} min-h-24`} value={reviewForm.reviewerNotes} onChange={(event) => setReviewForm({ ...reviewForm, reviewerNotes: event.target.value })} required />
          </FormField>
          {reviewForm.decision === 'accepted' && (
            <>
              <FormField label="脱敏公开成果摘要（20–3000 字）">
                <textarea className={`${fieldClassName} min-h-24`} value={reviewForm.publicResultSummary} onChange={(event) => setReviewForm({ ...reviewForm, publicResultSummary: event.target.value })} required />
              </FormField>
              <FormField label="安全 HTTPS 公开成果链接">
                <input type="url" className={fieldClassName} value={reviewForm.publicDeliverableUrl} onChange={(event) => setReviewForm({ ...reviewForm, publicDeliverableUrl: event.target.value })} required />
              </FormField>
            </>
          )}
          <FormField label="唯一审计引用（10–160 字）">
            <input className={fieldClassName} value={reviewForm.auditReference} onChange={(event) => setReviewForm({ ...reviewForm, auditReference: event.target.value })} required />
          </FormField>
          <button type="submit" disabled={state.submitting || !selected} className="w-full rounded border border-cyan-400/30 bg-cyan-400/10 px-4 py-2.5 text-xs font-black text-cyan-200 disabled:opacity-40">
            提交审核决定（不执行付款）
          </button>
          {reviewSubmission.notice && <InlineNotice notice={reviewSubmission.notice} />}
        </form>
      </div>

      <div className="rounded border border-zinc-800 bg-zinc-950 p-4">
        <h4 className="text-sm font-black text-zinc-200">不可变任务工作流事件</h4>
        {state.workspace.events.length === 0 ? (
          <p className="mt-3 text-[10px] text-zinc-600">暂无审计事件。</p>
        ) : (
          <div className="mt-3 grid gap-2 lg:grid-cols-2">
            {state.workspace.events.map((event) => (
              <div key={event.eventId} className="rounded border border-zinc-900 p-3 text-[10px] text-zinc-500">
                <div className="flex justify-between gap-2"><strong className="text-zinc-300">{event.action}</strong><span>{event.actorRole}</span></div>
                <p className="mt-2 break-all">{event.eventReference}</p>
                <p className="mt-1 text-zinc-700">{formatDate(event.createdAt)}</p>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function InlineNotice({ notice }: { notice: SubmissionNotice }) {
  return (
    <div className={`rounded border px-3 py-2 text-[10px] ${notice.tone === 'success' ? 'border-emerald-400/25 text-emerald-200' : 'border-red-400/25 text-red-200'}`}>
      {notice.message}
    </div>
  );
}

function BackendState({
  status,
  error,
  lastLoadedAt,
}: {
  status: ReturnType<typeof useOperations>['status'];
  error: string | null;
  lastLoadedAt: Date | null;
}) {
  if (status === 'unconfigured') {
    return (
      <section className="rounded-xl border border-yellow-400/25 bg-yellow-400/5 p-5">
        <div className="flex gap-3">
          <AlertTriangle className="mt-0.5 h-5 w-5 flex-none text-yellow-400" />
          <div>
            <h3 className="text-sm font-black text-yellow-200">运营后端尚未连接，所有提交入口已锁定</h3>
            <p className="mt-2 text-[11px] leading-relaxed text-zinc-400">{error}</p>
            <p className="mt-2 text-[11px] leading-relaxed text-zinc-500">
              先在独立 Supabase 项目执行迁移，再配置公开 URL 与 anon/publishable key。
              浏览器绝不能配置 service-role 或 secret key。
            </p>
          </div>
        </div>
      </section>
    );
  }

  if (status === 'error') {
    return (
      <section className="rounded-xl border border-red-400/25 bg-red-400/5 p-5">
        <div className="flex gap-3">
          <AlertTriangle className="mt-0.5 h-5 w-5 flex-none text-red-400" />
          <div>
            <h3 className="text-sm font-black text-red-200">公开数据读取失败</h3>
            <p className="mt-2 text-[11px] leading-relaxed text-zinc-400">{error}</p>
            <p className="mt-1 text-[10px] text-zinc-600">为避免误导，未回退到缓存或模拟数据。</p>
          </div>
        </div>
      </section>
    );
  }

  return (
    <section className="flex flex-col gap-2 rounded-xl border border-zinc-800 bg-zinc-950/70 px-4 py-3 text-[10px] text-zinc-600 sm:flex-row sm:items-center sm:justify-between">
      <span className="flex items-center gap-2">
        {status === 'loading'
          ? <Loader2 className="h-3.5 w-3.5 animate-spin text-cyan-400" />
          : <CheckCircle2 className="h-3.5 w-3.5 text-emerald-400" />}
        {status === 'loading' ? '正在读取公开记录…' : '公开记录已从 Supabase RLS 接口读取'}
      </span>
      <span>Last sync: {lastLoadedAt ? lastLoadedAt.toLocaleTimeString() : '—'}</span>
    </section>
  );
}

function SubmissionForm({
  title,
  description,
  busy,
  notice,
  disabled,
  onSubmit,
  children,
}: {
  title: string;
  description: string;
  busy: boolean;
  notice: SubmissionNotice | null;
  disabled: boolean;
  onSubmit: (event: FormEvent) => void;
  children: ReactNode;
}) {
  return (
    <form onSubmit={onSubmit} className="rounded border border-zinc-800 bg-zinc-950 p-4 sm:p-5">
      <h3 className="text-sm font-black text-zinc-200">{title}</h3>
      <p className="mt-2 text-[10px] leading-relaxed text-zinc-600">{description}</p>

      {!operationsBackendConfig.intakeEnabled && (
        <div className="mt-4 rounded border border-yellow-400/20 bg-yellow-400/5 px-3 py-2 text-[10px] leading-relaxed text-yellow-200/70">
          提交功能默认关闭。只有钱包身份 migration、Web3 Auth、Redirect URL 和滥用防护全部验证后，
          才能显式启用 wallet-staging intake。
        </div>
      )}
      {operationsBackendConfig.intakeEnabled && disabled && (
        <div className="mt-4 rounded border border-yellow-400/20 bg-yellow-400/5 px-3 py-2 text-[10px] leading-relaxed text-yellow-200/70">
          当前表单已锁定。请连接钱包并完成上方的 Solana Web3 签名认证。
        </div>
      )}

      <fieldset disabled={disabled || busy} className="mt-4 space-y-4 disabled:opacity-60">
        {children}
        <button
          type="submit"
          className="inline-flex w-full items-center justify-center gap-2 rounded border border-cyan-400/30 bg-cyan-400/10 px-4 py-2.5 text-xs font-black text-cyan-200 transition-colors hover:bg-cyan-400/15 disabled:cursor-not-allowed"
        >
          {busy ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Send className="h-3.5 w-3.5" />}
          {busy ? '提交中…' : '提交进入审核队列'}
        </button>
      </fieldset>

      {notice && (
        <div className={`mt-4 rounded border px-3 py-2 text-[10px] leading-relaxed ${
          notice.tone === 'success'
            ? 'border-emerald-400/25 bg-emerald-400/5 text-emerald-200'
            : 'border-red-400/25 bg-red-400/5 text-red-200'
        }`}>
          {notice.message}
        </div>
      )}
    </form>
  );
}

function PublicList({
  title,
  icon: Icon,
  loaded,
  emptyText,
  children,
}: {
  title: string;
  icon: typeof ClipboardCheck;
  loaded: boolean;
  emptyText: string;
  children: ReactNode;
}) {
  const hasChildren = Array.isArray(children) ? children.length > 0 : Boolean(children);

  return (
    <div>
      <h3 className="mb-4 flex items-center gap-2 text-sm font-black text-zinc-200">
        <Icon className="h-4 w-4 text-cyan-400" />
        {title}
      </h3>
      {!loaded && <LoadingRows />}
      {loaded && !hasChildren && <EmptyState text={emptyText} />}
      <div className="space-y-3">{children}</div>
    </div>
  );
}

function TwoColumnPanel({ children }: { children: ReactNode }) {
  return <div className="grid gap-6 lg:grid-cols-[minmax(0,1.1fr)_minmax(320px,0.9fr)]">{children}</div>;
}

function FormField({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="block">
      <span className="mb-1.5 block text-[10px] font-black text-zinc-500">{label}</span>
      {children}
    </label>
  );
}

function WalletField({
  value,
  label,
}: {
  value: string;
  label: string;
}) {
  return (
    <FormField label={label}>
      <div className="relative">
        <WalletCards className="pointer-events-none absolute left-3 top-3 h-3.5 w-3.5 text-zinc-700" />
        <input
          value={value}
          className={`${fieldClassName} pl-9`}
          placeholder="完成钱包签名认证后自动绑定"
          readOnly
          required
        />
      </div>
      <span className="mt-1.5 block text-[9px] leading-relaxed text-zinc-700">
        该地址必须同时匹配 Supabase Web3 身份与当前连接钱包；付款前仍需独立复核收款资格。
      </span>
    </FormField>
  );
}

function BoundaryStep({
  step,
  title,
  body,
  icon: Icon,
  tone,
}: {
  step: string;
  title: string;
  body: string;
  icon: typeof Users;
  tone: 'cyan' | 'emerald' | 'yellow' | 'violet';
}) {
  const toneClass = {
    cyan: 'text-cyan-300',
    emerald: 'text-emerald-300',
    yellow: 'text-yellow-300',
    violet: 'text-violet-300',
  }[tone];

  return (
    <div className="bg-zinc-950 p-4">
      <div className={`flex items-center gap-2 text-[9px] font-black tracking-widest ${toneClass}`}>
        <Icon className="h-3.5 w-3.5" />
        {step}
      </div>
      <h3 className="mt-3 text-xs font-black text-zinc-200">{title}</h3>
      <p className="mt-2 text-[10px] leading-relaxed text-zinc-600">{body}</p>
    </div>
  );
}

function StatusBadge({
  label,
  tone,
}: {
  label: string;
  tone: 'cyan' | 'emerald' | 'yellow' | 'red' | 'zinc';
}) {
  const toneClass = {
    cyan: 'border-cyan-400/25 bg-cyan-400/5 text-cyan-300',
    emerald: 'border-emerald-400/25 bg-emerald-400/5 text-emerald-300',
    yellow: 'border-yellow-400/25 bg-yellow-400/5 text-yellow-300',
    red: 'border-red-400/25 bg-red-400/5 text-red-300',
    zinc: 'border-zinc-800 bg-zinc-900/70 text-zinc-500',
  }[tone];

  return (
    <span className={`rounded border px-2 py-1 text-[9px] font-black uppercase tracking-widest ${toneClass}`}>
      {label}
    </span>
  );
}

function LoadingRows() {
  return (
    <div className="space-y-3">
      {[0, 1].map((index) => (
        <div key={index} className="h-24 animate-pulse rounded border border-zinc-900 bg-zinc-900/30" />
      ))}
    </div>
  );
}

function EmptyState({ text }: { text: string }) {
  return (
    <div className="rounded border border-dashed border-zinc-800 bg-zinc-950 p-6 text-center text-[11px] leading-relaxed text-zinc-600">
      {text}
    </div>
  );
}

function SafeLink({ href, label }: { href: string; label: string }) {
  return (
    <a
      href={href}
      target="_blank"
      rel="noreferrer noopener"
      className="inline-flex items-center gap-1 text-cyan-500 hover:text-cyan-300"
    >
      <Link2 className="h-3 w-3" />
      {label}
    </a>
  );
}

function useSubmission() {
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<SubmissionNotice | null>(null);

  async function run(
    action: () => Promise<void>,
    successMessage: string,
    afterSuccess: () => void,
  ): Promise<boolean> {
    setBusy(true);
    setNotice(null);

    try {
      await action();
      afterSuccess();
      setNotice({ tone: 'success', message: successMessage });
      return true;
    } catch (error) {
      setNotice({
        tone: 'error',
        message: error instanceof Error ? error.message : '提交失败，请稍后重试',
      });
      return false;
    } finally {
      setBusy(false);
    }
  }

  return { busy, notice, run };
}

function usePrefillWallet(wallet: string | null, update: (wallet: string) => void) {
  const updateRef = useRef(update);
  updateRef.current = update;

  useEffect(() => {
    updateRef.current(wallet ?? '');
  }, [wallet]);
}

function formatDate(value: string | null): string {
  if (!value) {
    return '未设置';
  }

  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? '无效时间' : date.toLocaleString();
}

function shorten(value: string): string {
  return value.length <= 16 ? value : `${value.slice(0, 7)}…${value.slice(-7)}`;
}

function formatIntakeGateStatus(
  status: OperationsWalletAuthState['intakeGateStatus'],
): string {
  return {
    unavailable: '登录后读取',
    checking: '读取中',
    enabled: '已开启',
    disabled: '已关闭',
    error: '读取失败（按关闭处理）',
  }[status];
}
