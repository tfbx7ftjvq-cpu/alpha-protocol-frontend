import { useEffect, useMemo, useState, type FormEvent, type ReactNode } from 'react';
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
  MessageSquareText,
  RefreshCw,
  Send,
  ShieldCheck,
  Users,
  WalletCards,
} from 'lucide-react';
import {
  submitDiscussion,
  submitReliefApplication,
  submitRiskReport,
  submitTaskResult,
} from '../features/operations/repository';
import { useOperations } from '../hooks/useOperations';
import { operationsBackendConfig } from '../lib/operationsSupabase';

type OperationsTab = 'tasks' | 'risk' | 'relief' | 'discussion' | 'decisions';
type NoticeTone = 'success' | 'error';

interface OperationsDashboardProps {
  connectedWallet: string | null;
}

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
  { key: 'decisions', label: '公开决定', icon: BookOpenCheck },
];

const fieldClassName = [
  'w-full rounded border border-zinc-800 bg-zinc-950 px-3 py-2.5',
  'text-xs text-zinc-200 outline-none transition-colors',
  'placeholder:text-zinc-700 focus:border-cyan-400/50',
  'disabled:cursor-not-allowed disabled:opacity-50',
].join(' ');

export default function OperationsDashboard({ connectedWallet }: OperationsDashboardProps) {
  const [activeTab, setActiveTab] = useState<OperationsTab>('tasks');
  const operations = useOperations();

  const counts = useMemo(() => ({
    tasks: operations.overview.tasks.length,
    risk: operations.overview.riskReports.length,
    relief: operations.overview.reliefUpdates.length,
    discussion: operations.overview.discussions.length,
    decisions: operations.overview.governanceDecisions.length,
  }), [operations.overview]);

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
                  label={operationsBackendConfig.intakeEnabled ? 'INTAKE ENABLED' : 'INTAKE LOCKED'}
                  tone={operationsBackendConfig.intakeEnabled ? 'emerald' : 'zinc'}
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

      <section className="rounded-xl border border-zinc-800 bg-zinc-950/70">
        <div className="flex overflow-x-auto border-b border-zinc-800">
          {TABS.map((tab) => {
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
              loaded={operations.status === 'ready'}
              connectedWallet={connectedWallet}
            />
          )}
          {activeTab === 'risk' && (
            <RiskPanel
              reports={operations.overview.riskReports}
              loaded={operations.status === 'ready'}
              connectedWallet={connectedWallet}
            />
          )}
          {activeTab === 'relief' && (
            <ReliefPanel
              updates={operations.overview.reliefUpdates}
              loaded={operations.status === 'ready'}
              connectedWallet={connectedWallet}
            />
          )}
          {activeTab === 'discussion' && (
            <DiscussionPanel
              discussions={operations.overview.discussions}
              loaded={operations.status === 'ready'}
              connectedWallet={connectedWallet}
            />
          )}
          {activeTab === 'decisions' && (
            <DecisionsPanel
              decisions={operations.overview.governanceDecisions}
              loaded={operations.status === 'ready'}
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
  loaded,
  connectedWallet,
}: {
  tasks: ReturnType<typeof useOperations>['overview']['tasks'];
  loaded: boolean;
  connectedWallet: string | null;
}) {
  const [form, setForm] = useState({
    taskId: '',
    summary: '',
    deliverableUrl: '',
    walletAddress: connectedWallet ?? '',
  });
  const submission = useSubmission();

  usePrefillWallet(connectedWallet, (walletAddress) => {
    setForm((current) => current.walletAddress ? current : { ...current, walletAddress });
  });

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    await submission.run(
      () => submitTaskResult(form),
      '任务成果已进入私有审核队列；这不是付款承诺。',
      () => setForm((current) => ({
        ...current,
        summary: '',
        deliverableUrl: '',
      })),
    );
  }

  return (
    <TwoColumnPanel>
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

      <SubmissionForm
        title="提交任务成果"
        description="提交后仅进入审核。accepted 不代表已付款；付款需另行形成治理决定与执行记录。"
        busy={submission.busy}
        notice={submission.notice}
        onSubmit={handleSubmit}
        disabled={!operationsBackendConfig.intakeEnabled || tasks.length === 0}
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
          onChange={(walletAddress) => setForm({ ...form, walletAddress })}
          label="拟收款钱包（未签名验证）"
        />
      </SubmissionForm>
    </TwoColumnPanel>
  );
}

function RiskPanel({
  reports,
  loaded,
  connectedWallet,
}: {
  reports: ReturnType<typeof useOperations>['overview']['riskReports'];
  loaded: boolean;
  connectedWallet: string | null;
}) {
  const [form, setForm] = useState({
    projectIdentifier: '',
    summary: '',
    referenceUrl: '',
    walletAddress: connectedWallet ?? '',
  });
  const submission = useSubmission();

  usePrefillWallet(connectedWallet, (walletAddress) => {
    setForm((current) => current.walletAddress ? current : { ...current, walletAddress });
  });

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    await submission.run(
      () => submitRiskReport(form),
      '风险报告已私密提交；公开前必须人工核验并净化敏感信息。',
      () => setForm((current) => ({
        ...current,
        projectIdentifier: '',
        summary: '',
        referenceUrl: '',
      })),
    );
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
        disabled={!operationsBackendConfig.intakeEnabled}
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
          onChange={(walletAddress) => setForm({ ...form, walletAddress })}
          label="可选提交钱包（未签名验证）"
          required={false}
        />
      </SubmissionForm>
    </TwoColumnPanel>
  );
}

function ReliefPanel({
  updates,
  loaded,
  connectedWallet,
}: {
  updates: ReturnType<typeof useOperations>['overview']['reliefUpdates'];
  loaded: boolean;
  connectedWallet: string | null;
}) {
  const [form, setForm] = useState({
    incidentSummary: '',
    requestedAmountUsdc: '',
    evidenceUrl: '',
    walletAddress: connectedWallet ?? '',
  });
  const submission = useSubmission();

  usePrefillWallet(connectedWallet, (walletAddress) => {
    setForm((current) => current.walletAddress ? current : { ...current, walletAddress });
  });

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    await submission.run(
      () => submitReliefApplication(form),
      '救助申请已私密提交；申请、批准与实际赔付是三个独立阶段。',
      () => setForm((current) => ({
        ...current,
        incidentSummary: '',
        requestedAmountUsdc: '',
        evidenceUrl: '',
      })),
    );
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
        disabled={!operationsBackendConfig.intakeEnabled}
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
          onChange={(walletAddress) => setForm({ ...form, walletAddress })}
          label="拟收款钱包（未签名验证）"
        />
      </SubmissionForm>
    </TwoColumnPanel>
  );
}

function DiscussionPanel({
  discussions,
  loaded,
  connectedWallet,
}: {
  discussions: ReturnType<typeof useOperations>['overview']['discussions'];
  loaded: boolean;
  connectedWallet: string | null;
}) {
  const [form, setForm] = useState({
    topic: '',
    body: '',
    walletAddress: connectedWallet ?? '',
  });
  const submission = useSubmission();

  usePrefillWallet(connectedWallet, (walletAddress) => {
    setForm((current) => current.walletAddress ? current : { ...current, walletAddress });
  });

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    await submission.run(
      () => submitDiscussion(form),
      '讨论已进入审核队列；通过后才会出现在公开页面。',
      () => setForm((current) => ({ ...current, topic: '', body: '' })),
    );
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
        disabled={!operationsBackendConfig.intakeEnabled}
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
          onChange={(walletAddress) => setForm({ ...form, walletAddress })}
          label="可选发言钱包（未签名验证）"
          required={false}
        />
      </SubmissionForm>
    </TwoColumnPanel>
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
          提交功能默认关闭。只有部署者显式启用 anonymous intake 且 Supabase RLS 已应用后才会开放。
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
  onChange,
  label,
  required = true,
}: {
  value: string;
  onChange: (value: string) => void;
  label: string;
  required?: boolean;
}) {
  return (
    <FormField label={label}>
      <div className="relative">
        <WalletCards className="pointer-events-none absolute left-3 top-3 h-3.5 w-3.5 text-zinc-700" />
        <input
          value={value}
          onChange={(event) => onChange(event.target.value)}
          className={`${fieldClassName} pl-9`}
          placeholder="Solana public address"
          required={required}
        />
      </div>
      <span className="mt-1.5 block text-[9px] leading-relaxed text-zinc-700">
        自动填充连接钱包不构成签名证明；付款前必须另行完成地址所有权验证。
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

  async function run(action: () => Promise<void>, successMessage: string, afterSuccess: () => void) {
    setBusy(true);
    setNotice(null);

    try {
      await action();
      afterSuccess();
      setNotice({ tone: 'success', message: successMessage });
    } catch (error) {
      setNotice({
        tone: 'error',
        message: error instanceof Error ? error.message : '提交失败，请稍后重试',
      });
    } finally {
      setBusy(false);
    }
  }

  return { busy, notice, run };
}

function usePrefillWallet(wallet: string | null, update: (wallet: string) => void) {
  useEffect(() => {
    if (wallet) {
      update(wallet);
    }
  }, [update, wallet]);
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
