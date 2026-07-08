import { type ElementType } from 'react';
import {
  AlertTriangle,
  CheckCircle2,
  ExternalLink,
  FileCheck2,
  Gauge,
  Landmark,
  Loader2,
  LockKeyhole,
  RefreshCw,
  ShieldAlert,
  ShieldCheck,
} from 'lucide-react';
import { useGreenLabelConfig } from '../hooks/useGreenLabelConfig';
import { useGreenLabelE2EResults } from '../hooks/useGreenLabelE2EResults';
import {
  formatBps,
  formatDuration,
  formatUnixTimestamp,
  formatUsdcAmount,
  getGreenLabelParameterMode,
  getGreenLabelExplorerAddressUrl,
  type GreenLabelConfigV1,
  type GreenLabelDisputeV1,
  type GreenLabelE2EResult,
  type GreenLabelParameterMode,
  type GreenLabelProjectV1,
} from '../lib/greenLabel';

const PROGRAM_ID = 'HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY';
const GREEN_LABEL_CONFIG = '7hNAeoqZxqvp38giY9gZwfR5ai3ttYTrse63QNYrRBWS';
const USDC_MINT = '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU';
const TREASURY_USDC_STATE_V2 = '5e7eyC5ViwH9GBn73cY6so7J6KpRCX6XsbxozHabk2fE';
const SECURITY_GOVERNANCE_CONFIG = '5np4fcpSP8eHVLD6dsgLHf7H11VLaGcYgdadxidt9ro3';

const ADDRESS_ITEMS = [
  { label: 'Program ID', value: PROGRAM_ID, tone: 'text-cyan-300', icon: ShieldCheck },
  { label: 'Green Label config PDA', value: GREEN_LABEL_CONFIG, tone: 'text-emerald-300', icon: FileCheck2 },
  { label: 'USDC Mint', value: USDC_MINT, tone: 'text-blue-300', icon: Landmark },
  { label: 'Treasury USDC State V2', value: TREASURY_USDC_STATE_V2, tone: 'text-yellow-300', icon: Landmark },
  { label: 'Security Governance Config', value: SECURITY_GOVERNANCE_CONFIG, tone: 'text-red-300', icon: LockKeyhole },
];

const DEVNET_PARAMETERS = [
  { label: 'min_base_bond_usdc', value: '1 USDC' },
  { label: 'observation_period_seconds', value: '30' },
  { label: 'dispute_window_seconds', value: '30' },
  { label: 'response_window_seconds', value: '30' },
];

const MAINNET_PARAMETERS = [
  { label: 'min_base_bond_usdc', value: '299 USDC' },
  { label: 'observation_period_seconds', value: '2592000', caption: '30 天' },
  { label: 'dispute_window_seconds', value: '604800', caption: '7 天' },
  { label: 'response_window_seconds', value: '259200', caption: '3 天' },
];

const VERIFIED_PATH = [
  'submit_green_label_application',
  'initialize_green_bond_vault',
  'lock_green_label_bond',
  'open_green_label_dispute',
  'mark_dispute_ready_for_decision',
  'create Security Layer decision',
  'queue execution',
  'link_green_label_security_decision',
  'execute_green_label_refund',
  'execute_green_label_slash',
];

const MAINNET_READINESS = [
  '恢复正式参数：299 USDC / 30天 / 7天 / 3天。',
  '将 config authority 迁移到 DAO / multisig / Security Layer timelock。',
  'Devnet-only scripts 不可用于 Mainnet。',
  'update config 权限需完成审计。',
];

const EXPLORER_LINKS = [
  {
    label: 'Program Explorer',
    href: `https://explorer.solana.com/address/${PROGRAM_ID}?cluster=devnet`,
  },
  {
    label: 'Refund execute tx',
    href: 'https://explorer.solana.com/tx/5JZYiMM5EjsrrjBDGSdDpuzf7xrkLYQispzAxwdo3WagtKFxBD9beE1KLNVmjShMZBoCvYVcSgwe1s7DDhMymNha?cluster=devnet',
  },
  {
    label: 'Slash execute tx',
    href: 'https://explorer.solana.com/tx/4be5Vxt6rfgXNTz7PxVT73FGoCYqCpoQwSjTZGXcvGrHe7R5anrLZZ9KCNoLiSDacW1Ahze3HwEdQn4gEdkUmZ8A?cluster=devnet',
  },
];

const PROJECT_STATUS_LABELS: Record<string, string> = {
  PendingBondDeposit: '待锁定 Bond',
  PendingObservation: '观察期中',
  ActiveGreenLabel: 'Green Label 生效',
  Disputed: '争议中',
  RefundQueued: '退款已排队',
  SlashQueued: '罚没已排队',
  Refunded: '已退款',
  Slashed: '已罚没',
  Cancelled: '已取消',
};

const DISPUTE_STATUS_LABELS: Record<string, string> = {
  Open: '已开启',
  EvidencePeriod: '证据期',
  ProjectResponsePeriod: '项目方回应期',
  ReadyForDecision: '等待裁决',
  DecisionQueued: '裁决已排队',
  ResolvedRefund: '退款解决',
  ResolvedSlash: '罚没解决',
  Rejected: '已驳回',
  Cancelled: '已取消',
};

const BOND_TIER_LABELS: Record<string, string> = {
  Base: '基础承诺',
  Bronze: '青铜承诺',
  Silver: '白银承诺',
  Gold: '黄金承诺',
  Platinum: '铂金承诺',
  Custom: '自定义承诺',
};

const RUG_REASON_LABELS: Record<string, string> = {
  LiquidityRemoved: '移除流动性',
  DeveloperDump: '开发者抛售',
  WebsiteOrCommunityAbandoned: '网站或社区废弃',
  MintOrFreezeAuthorityAbuse: 'Mint/Freeze 权限滥用',
  TreasuryMisuse: '项目金库滥用',
  FalseDisclosure: '虚假披露',
  MaliciousContractUpgrade: '恶意合约升级',
  Other: '其他',
};

const ACTION_TYPE_LABELS: Record<string, string> = {
  Noop: '空操作',
  GreenLabelSlash: 'Green Label 罚没',
  GreenLabelRefund: 'Green Label 退款',
  PayrollEmployeeImpeach: 'Payroll 弹劾',
  PayrollPayout: 'Payroll 支付',
  TreasuryParamChange: '国库参数变更',
  EmergencyPause: '紧急暂停',
};

export default function GreenLabelDashboard() {
  const { config, error, lastLoadedAt, refresh, status } = useGreenLabelConfig();
  const {
    error: e2eError,
    lastLoadedAt: e2eLastLoadedAt,
    refresh: refreshE2EResults,
    results: e2eResults,
    status: e2eStatus,
  } = useGreenLabelE2EResults();
  const parameterMode = config ? getGreenLabelParameterMode(config) : null;

  return (
    <div className="space-y-8">
      <section className="space-y-5">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div className="space-y-3">
            <div className="flex flex-wrap items-center gap-2">
              <Badge icon={ShieldCheck} label="Green Label V1" tone="emerald" />
              <Badge icon={Gauge} label="Devnet E2E Milestone" tone="cyan" />
              <Badge icon={LockKeyhole} label="Read-only" tone="zinc" />
            </div>
            <div>
              <h2 className="text-xl font-black uppercase tracking-wide text-zinc-100">
                Green Label / 风险承诺金
              </h2>
              <p className="mt-2 max-w-3xl text-xs leading-relaxed text-zinc-500">
                展示 Green Label V1 Devnet refund/slash E2E 已完成的只读里程碑。本页面不发起链上交易，不提供 initialize、update、refund 或 slash 操作。
              </p>
            </div>
          </div>

          <div className="rounded border border-emerald-400/25 bg-emerald-400/5 px-4 py-3 text-xs font-bold text-emerald-300">
            Devnet status: verified
          </div>
        </div>

        <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-5">
          {ADDRESS_ITEMS.map((item) => (
            <InfoTile
              key={item.label}
              icon={item.icon}
              label={item.label}
              value={item.value}
              tone={item.tone}
            />
          ))}
        </div>
      </section>

      <DevnetParameterBanner />

      <OnChainConfigPanel
        config={config}
        error={error}
        lastLoadedAt={lastLoadedAt}
        onRefresh={refresh}
        parameterMode={parameterMode}
        status={status}
      />

      <E2EResultsPanel
        error={e2eError}
        lastLoadedAt={e2eLastLoadedAt}
        onRefresh={refreshE2EResults}
        results={e2eResults}
        status={e2eStatus}
      />

      <section className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <div className="rounded-xl border border-red-400/25 bg-red-400/5 p-5">
          <SectionHeader
            icon={AlertTriangle}
            eyebrow="Devnet Test Parameters"
            title="Devnet 测试参数"
            description="这些是为了快速完成 Devnet E2E 验证而设置的测试参数，不是 Mainnet 正式参数。"
            tone="text-red-300"
          />
          <ParameterGrid items={DEVNET_PARAMETERS} tone="red" />
          <div className="mt-4 rounded border border-red-400/30 bg-red-400/10 px-3 py-2 text-xs font-bold leading-relaxed text-red-200">
            警告：这些是 Devnet E2E 测试参数，不代表 Mainnet 上线配置。
          </div>
        </div>

        <div className="rounded-xl border border-emerald-400/20 bg-emerald-400/5 p-5">
          <SectionHeader
            icon={ShieldCheck}
            eyebrow="Mainnet Production Parameters"
            title="Mainnet 正式参数"
            description="Mainnet 前必须恢复正式 bond 与时间窗口参数。"
            tone="text-emerald-300"
          />
          <ParameterGrid items={MAINNET_PARAMETERS} tone="emerald" />
        </div>
      </section>

      <section className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <MilestoneCard
          icon={CheckCircle2}
          title="Refund E2E 结果"
          tone="emerald"
          rows={[
            ['project_id', '2'],
            ['status', 'Refunded'],
            ['dispute status', 'ResolvedRefund'],
            ['bond', '1 USDC'],
            ['project_owner_usdc_ata delta', '-0.2 USDC'],
            ['base_bond_treasury_vault delta', '+0.2 USDC'],
            ['green_bond_vault final', '0'],
          ]}
          note="验证 base bond 80% 退回项目方，20% 进入协议国库。"
        />

        <MilestoneCard
          icon={ShieldAlert}
          title="Slash E2E 结果"
          tone="red"
          rows={[
            ['project_id', '3'],
            ['status', 'Slashed'],
            ['dispute status', 'ResolvedSlash'],
            ['bond', '1 USDC'],
            ['project_owner_usdc_ata delta', '-1 USDC'],
            ['relief_or_risk_vault delta', '+1 USDC'],
            ['green_bond_vault final', '0'],
          ]}
          note="验证恶意/风险裁定后 bond 全额进入 relief/risk vault。"
        />
      </section>

      <section className="grid grid-cols-1 gap-4 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,0.9fr)]">
        <div className="rounded-xl border border-cyan-400/20 bg-cyan-400/5 p-5">
          <SectionHeader
            icon={FileCheck2}
            eyebrow="Verified Flow"
            title="已验证路径"
            description="Green Label 与 Security Layer 的 Devnet 闭环路径已经完成验证。"
            tone="text-cyan-300"
          />
          <div className="mt-4 grid grid-cols-1 gap-1.5 sm:grid-cols-2 xl:grid-cols-3">
            {VERIFIED_PATH.map((item) => (
              <div
                key={item}
                className="flex min-w-0 items-start gap-2 rounded border border-zinc-800 bg-zinc-950/70 px-2 py-1.5 text-[10px] font-bold text-zinc-300"
              >
                <CheckCircle2 className="mt-0.5 h-3 w-3 flex-shrink-0 text-emerald-400" />
                <span className="break-all font-mono">{item}</span>
              </div>
            ))}
          </div>
        </div>

        <div className="rounded-xl border border-zinc-800 bg-zinc-950/60 p-5">
          <SectionHeader
            icon={ExternalLink}
            eyebrow="Explorer"
            title="Explorer 链接"
            description="Devnet program 与已完成的 refund/slash execute 交易。"
            tone="text-zinc-300"
          />
          <div className="mt-4 space-y-2">
            {EXPLORER_LINKS.map((link) => (
              <a
                key={link.href}
                href={link.href}
                target="_blank"
                rel="noreferrer"
                className="flex items-center justify-between gap-3 rounded border border-zinc-800 bg-zinc-950/80 px-3 py-2 text-xs font-bold text-zinc-300 transition-all hover:border-cyan-400/40 hover:text-cyan-300"
              >
                <span>{link.label}</span>
                <ExternalLink className="h-3.5 w-3.5 flex-shrink-0" />
              </a>
            ))}
          </div>
        </div>
      </section>

      <section className="rounded-xl border border-orange-400/25 bg-orange-400/5 p-5">
        <SectionHeader
          icon={AlertTriangle}
          eyebrow="Mainnet Readiness"
          title="风险提示 / Mainnet 前置事项"
          description="Green Label V1 进入 Mainnet 前仍需处理参数恢复、权限迁移和脚本隔离。"
          tone="text-orange-300"
        />
        <div className="mt-4 grid grid-cols-1 gap-2 md:grid-cols-2">
          {MAINNET_READINESS.map((item) => (
            <div
              key={item}
              className="flex items-start gap-2 rounded border border-orange-400/20 bg-zinc-950/70 px-3 py-2 text-xs leading-relaxed text-orange-100"
            >
              <AlertTriangle className="mt-0.5 h-3.5 w-3.5 flex-shrink-0 text-orange-300" />
              <span>{item}</span>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}

function Badge({ icon: Icon, label, tone }: { icon: ElementType; label: string; tone: 'emerald' | 'cyan' | 'zinc' }) {
  const className = {
    emerald: 'border-emerald-400/25 bg-emerald-400/10 text-emerald-400',
    cyan: 'border-cyan-400/25 bg-cyan-400/10 text-cyan-400',
    zinc: 'border-zinc-700 bg-zinc-900/60 text-zinc-400',
  }[tone];

  return (
    <span className={`inline-flex items-center gap-1.5 rounded border px-2 py-1 text-[10px] font-bold uppercase tracking-widest ${className}`}>
      <Icon className="h-3 w-3" />
      {label}
    </span>
  );
}

function SectionHeader({
  icon: Icon,
  eyebrow,
  title,
  description,
  tone,
}: {
  icon: ElementType;
  eyebrow: string;
  title: string;
  description: string;
  tone: string;
}) {
  return (
    <div>
      <div className={`mb-2 flex items-center gap-2 text-[10px] font-black uppercase tracking-widest ${tone}`}>
        <Icon className="h-3.5 w-3.5" />
        {eyebrow}
      </div>
      <h3 className="text-lg font-black text-zinc-100">{title}</h3>
      <p className="mt-2 text-xs leading-relaxed text-zinc-400">{description}</p>
    </div>
  );
}

function InfoTile({
  icon: Icon,
  label,
  value,
  tone,
}: {
  icon: ElementType;
  label: string;
  value: string;
  tone: string;
}) {
  return (
    <div className="rounded border border-zinc-800 bg-zinc-950/70 p-4">
      <div className="mb-2 flex items-center gap-2">
        <Icon className={`h-4 w-4 ${tone}`} />
        <p className="text-[10px] font-bold uppercase tracking-widest text-zinc-600">{label}</p>
      </div>
      <p className={`break-all font-mono text-xs font-bold leading-relaxed ${tone}`}>{value}</p>
    </div>
  );
}

function DevnetParameterBanner() {
  return (
    <section className="rounded-xl border border-red-400/35 bg-red-400/10 p-4">
      <div className="flex items-start gap-3">
        <AlertTriangle className="mt-0.5 h-5 w-5 flex-shrink-0 text-red-300" />
        <div className="space-y-1">
          <p className="text-sm font-black text-red-100">
            当前为 Devnet E2E 测试配置：1 USDC / 30s / 30s / 30s，不代表 Mainnet 正式规则。
          </p>
          <p className="text-xs font-bold leading-relaxed text-red-200/90">
            Mainnet 前必须恢复 299 USDC / 30天 / 7天 / 3天。
          </p>
        </div>
      </div>
    </section>
  );
}

function OnChainConfigPanel({
  config,
  error,
  lastLoadedAt,
  onRefresh,
  parameterMode,
  status,
}: {
  config: GreenLabelConfigV1 | null;
  error: string | null;
  lastLoadedAt: Date | null;
  onRefresh: () => Promise<void>;
  parameterMode: GreenLabelParameterMode | null;
  status: 'idle' | 'loading' | 'ready' | 'error';
}) {
  const isLoading = status === 'loading';
  const keyRows = config ? getOnChainConfigKeyRows(config) : [];
  const advancedRows = config ? getOnChainConfigAdvancedRows(config) : [];

  return (
    <section className="rounded-xl border border-cyan-400/20 bg-cyan-400/5 p-5">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <SectionHeader
          icon={Gauge}
          eyebrow="On-chain GreenLabelConfigV1"
          title="链上实时配置"
          description="从 Devnet RPC 只读读取 GreenLabelConfigV1 PDA；无需连接钱包，不要求签名，也不会发起任何写入交易。"
          tone="text-cyan-300"
        />

        <div className="flex flex-col items-start gap-2 lg:items-end">
          <StatusPill status={status} />
          <button
            type="button"
            onClick={() => void onRefresh()}
            disabled={isLoading}
            className="inline-flex items-center justify-center gap-2 rounded border border-cyan-400/30 bg-cyan-400/10 px-4 py-2 text-xs font-bold text-cyan-300 transition-all hover:bg-cyan-400/15 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {isLoading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <RefreshCw className="h-3.5 w-3.5" />}
            {isLoading ? '读取中...' : '刷新链上配置'}
          </button>
          {lastLoadedAt && (
            <p className="text-[10px] text-zinc-600">
              Last loaded: {lastLoadedAt.toLocaleTimeString('zh-CN')}
            </p>
          )}
        </div>
      </div>

      <div className="mt-4 flex flex-wrap gap-2 text-[10px] font-bold text-zinc-500">
        <span className="rounded border border-zinc-800 bg-zinc-950/70 px-2 py-1">
          RPC: https://api.devnet.solana.com
        </span>
        <span className="break-all rounded border border-zinc-800 bg-zinc-950/70 px-2 py-1">
          PDA: {GREEN_LABEL_CONFIG}
        </span>
      </div>

      {parameterMode && <ParameterModeNotice mode={parameterMode} />}

      {error && (
        <div className="mt-4 flex items-start gap-2 rounded border border-red-400/30 bg-red-400/10 px-3 py-2 text-xs leading-relaxed text-red-200">
          <ShieldAlert className="mt-0.5 h-3.5 w-3.5 flex-shrink-0" />
          <span className="break-words">链上读取失败：{error}</span>
        </div>
      )}

      {isLoading && !config && (
        <div className="mt-4 flex items-center gap-2 rounded border border-zinc-800 bg-zinc-950/70 px-3 py-3 text-xs font-bold text-zinc-400">
          <Loader2 className="h-4 w-4 animate-spin text-cyan-300" />
          正在读取 GreenLabelConfigV1...
        </div>
      )}

      {config && (
        <div className="mt-4 space-y-3">
          <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-6">
            {keyRows.map((row) => (
              <ConfigMetricTile key={row.label} row={row} />
            ))}
          </div>

          <details className="rounded border border-zinc-800 bg-zinc-950/60 p-3">
            <summary className="cursor-pointer text-xs font-black uppercase tracking-widest text-zinc-400 hover:text-cyan-300">
              Advanced config
            </summary>
            <div className="mt-3 grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
              {advancedRows.map((row) => (
                <ConfigMetricTile key={row.label} row={row} />
              ))}
            </div>
          </details>
        </div>
      )}
    </section>
  );
}

function ConfigMetricTile({ row }: { row: ConfigRow }) {
  return (
    <div className="rounded border border-zinc-800 bg-zinc-950/75 p-3">
      <p className="break-all font-mono text-[10px] font-bold uppercase tracking-widest text-zinc-600">
        {row.label}
      </p>
      <p className={`mt-2 break-all font-mono text-sm font-black leading-relaxed ${row.tone}`}>
        {row.value}
      </p>
      {row.caption && (
        <p className="mt-1 break-all text-[10px] font-bold text-zinc-500">{row.caption}</p>
      )}
    </div>
  );
}

function StatusPill({ status }: { status: 'idle' | 'loading' | 'ready' | 'error' }) {
  const statusMeta = {
    idle: {
      label: '等待读取',
      className: 'border-zinc-700 bg-zinc-900/60 text-zinc-400',
    },
    loading: {
      label: '读取中',
      className: 'border-yellow-400/30 bg-yellow-400/10 text-yellow-300',
    },
    ready: {
      label: '链上已同步',
      className: 'border-emerald-400/30 bg-emerald-400/10 text-emerald-300',
    },
    error: {
      label: '读取失败',
      className: 'border-red-400/30 bg-red-400/10 text-red-300',
    },
  }[status];

  return (
    <span className={`inline-flex w-fit items-center gap-2 rounded border px-3 py-1.5 text-xs font-bold ${statusMeta.className}`}>
      {status === 'loading' && <Loader2 className="h-3.5 w-3.5 animate-spin" />}
      {statusMeta.label}
    </span>
  );
}

function ParameterModeNotice({ mode }: { mode: GreenLabelParameterMode }) {
  if (mode === 'mainnet-like') {
    return (
      <div className="mt-4 flex items-start gap-2 rounded border border-emerald-400/30 bg-emerald-400/10 px-3 py-2 text-xs font-bold leading-relaxed text-emerald-200">
        <ShieldCheck className="mt-0.5 h-3.5 w-3.5 flex-shrink-0" />
        <span>链上配置接近正式参数：299U / 30天 / 7天 / 3天。</span>
      </div>
    );
  }

  if (mode === 'devnet-test') {
    return (
      <div className="mt-4 flex items-start gap-2 rounded border border-orange-400/35 bg-orange-400/10 px-3 py-2 text-xs font-bold leading-relaxed text-orange-200">
        <AlertTriangle className="mt-0.5 h-3.5 w-3.5 flex-shrink-0" />
        <span>链上配置命中 Devnet 测试参数：1 USDC 或 30 秒窗口。Mainnet 前必须恢复正式参数。</span>
      </div>
    );
  }

  return (
    <div className="mt-4 flex items-start gap-2 rounded border border-yellow-400/30 bg-yellow-400/10 px-3 py-2 text-xs font-bold leading-relaxed text-yellow-200">
      <AlertTriangle className="mt-0.5 h-3.5 w-3.5 flex-shrink-0" />
      <span>链上配置不是已知 Devnet 测试参数，也不是完整正式参数，请在 Mainnet 前单独复核。</span>
    </div>
  );
}

function getOnChainConfigKeyRows(config: GreenLabelConfigV1): ConfigRow[] {
  return [
    {
      label: 'min_base_bond_usdc',
      value: formatUsdcAmount(config.minBaseBondUsdc),
      caption: `${config.minBaseBondUsdc.toString()} raw units`,
      tone: 'text-yellow-300',
    },
    {
      label: 'observation_period_seconds',
      value: `${config.observationPeriodSeconds.toString()} seconds`,
      caption: formatDuration(config.observationPeriodSeconds),
      tone: 'text-orange-300',
    },
    {
      label: 'dispute_window_seconds',
      value: `${config.disputeWindowSeconds.toString()} seconds`,
      caption: formatDuration(config.disputeWindowSeconds),
      tone: 'text-orange-300',
    },
    {
      label: 'response_window_seconds',
      value: `${config.responseWindowSeconds.toString()} seconds`,
      caption: formatDuration(config.responseWindowSeconds),
      tone: 'text-orange-300',
    },
    { label: 'project_count', value: config.projectCount.toString(), tone: 'text-zinc-100' },
    { label: 'is_paused', value: config.isPaused ? 'true' : 'false', tone: config.isPaused ? 'text-red-300' : 'text-emerald-300' },
  ];
}

function getOnChainConfigAdvancedRows(config: GreenLabelConfigV1): ConfigRow[] {
  return [
    { label: 'authority', value: config.authority, tone: 'text-emerald-300' },
    { label: 'usdc_mint', value: config.usdcMint, tone: 'text-blue-300' },
    {
      label: 'base_refund_bps',
      value: `${config.baseRefundBps} bps`,
      caption: formatBps(config.baseRefundBps),
      tone: 'text-emerald-300',
    },
    {
      label: 'base_treasury_bps',
      value: `${config.baseTreasuryBps} bps`,
      caption: formatBps(config.baseTreasuryBps),
      tone: 'text-cyan-300',
    },
    { label: 'treasury_usdc_state_v2', value: config.treasuryUsdcStateV2, tone: 'text-yellow-300' },
    { label: 'base_bond_treasury_vault', value: config.baseBondTreasuryVault, tone: 'text-yellow-300' },
    { label: 'relief_or_risk_vault', value: config.reliefOrRiskVault, tone: 'text-red-300' },
    { label: 'vault_authority_v2', value: config.vaultAuthorityV2, tone: 'text-cyan-300' },
    { label: 'security_governance_config', value: config.securityGovernanceConfig, tone: 'text-red-300' },
  ];
}

interface ConfigRow {
  label: string;
  value: string;
  caption?: string;
  tone: string;
}

function E2EResultsPanel({
  error,
  lastLoadedAt,
  onRefresh,
  results,
  status,
}: {
  error: string | null;
  lastLoadedAt: Date | null;
  onRefresh: () => Promise<void>;
  results: GreenLabelE2EResult[];
  status: 'idle' | 'loading' | 'ready' | 'error';
}) {
  const isLoading = status === 'loading';

  return (
    <section className="rounded-xl border border-emerald-400/20 bg-emerald-400/5 p-5">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <SectionHeader
          icon={FileCheck2}
          eyebrow="On-chain E2E Project Results"
          title="链上 E2E 结果验证"
          description="真实读取 Refund Project #2 与 Slash Project #3 的 GreenLabelProjectV1 / GreenLabelDisputeV1 账户，并核对最终状态。"
          tone="text-emerald-300"
        />

        <div className="flex flex-col items-start gap-2 lg:items-end">
          <StatusPill status={status} />
          <button
            type="button"
            onClick={() => void onRefresh()}
            disabled={isLoading}
            className="inline-flex items-center justify-center gap-2 rounded border border-emerald-400/30 bg-emerald-400/10 px-4 py-2 text-xs font-bold text-emerald-300 transition-all hover:bg-emerald-400/15 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {isLoading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <RefreshCw className="h-3.5 w-3.5" />}
            {isLoading ? '读取中...' : '刷新 E2E 账户'}
          </button>
          {lastLoadedAt && (
            <p className="text-[10px] text-zinc-600">
              Last loaded: {lastLoadedAt.toLocaleTimeString('zh-CN')}
            </p>
          )}
        </div>
      </div>

      {error && (
        <div className="mt-4 flex items-start gap-2 rounded border border-red-400/30 bg-red-400/10 px-3 py-2 text-xs leading-relaxed text-red-200">
          <ShieldAlert className="mt-0.5 h-3.5 w-3.5 flex-shrink-0" />
          <span className="break-words">链上 E2E 账户读取失败：{error}</span>
        </div>
      )}

      {isLoading && results.length === 0 && (
        <div className="mt-4 flex items-center gap-2 rounded border border-zinc-800 bg-zinc-950/70 px-3 py-3 text-xs font-bold text-zinc-400">
          <Loader2 className="h-4 w-4 animate-spin text-emerald-300" />
          正在读取 Project / Dispute 账户...
        </div>
      )}

      {results.length > 0 && (
        <div className="mt-4 grid grid-cols-1 gap-4 xl:grid-cols-2">
          {results.map((result) => (
            <E2EResultCard key={result.key} result={result} />
          ))}
        </div>
      )}
    </section>
  );
}

function E2EResultCard({ result }: { result: GreenLabelE2EResult }) {
  const isRefund = result.key === 'refund';
  const statusMatches = result.project.status === result.expectedProjectStatus
    && result.dispute.status === result.expectedDisputeStatus;
  const toneClass = isRefund
    ? 'border-emerald-400/25 bg-emerald-400/5 text-emerald-300'
    : 'border-orange-400/30 bg-orange-400/5 text-orange-300';
  const Icon = isRefund ? CheckCircle2 : ShieldAlert;
  const summaryRows = getE2ESummaryRows(result, statusMatches);

  return (
    <div className={`rounded-xl border p-5 ${toneClass}`}>
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div className="flex items-center gap-2">
          <Icon className="h-5 w-5 flex-shrink-0" />
          <div>
            <h3 className="text-lg font-black text-zinc-100">{result.title}</h3>
            <p className="mt-1 text-[10px] font-bold uppercase tracking-widest text-zinc-500">
              project_id: {result.projectId}
            </p>
          </div>
        </div>
        <div className={`rounded border px-2 py-1 text-[10px] font-black ${statusMatches ? 'border-emerald-400/30 bg-emerald-400/10 text-emerald-300' : 'border-yellow-400/35 bg-yellow-400/10 text-yellow-200'}`}>
          {statusMatches ? 'Status verified' : 'Status mismatch'}
        </div>
      </div>

      {!statusMatches && (
        <div className="mt-4 rounded border border-yellow-400/30 bg-yellow-400/10 px-3 py-2 text-xs font-bold leading-relaxed text-yellow-200">
          Expected project={result.expectedProjectStatus}, dispute={result.expectedDisputeStatus}; actual project={result.project.status}, dispute={result.dispute.status}.
        </div>
      )}

      <div className="mt-4 grid grid-cols-1 gap-2 sm:grid-cols-2 xl:grid-cols-3">
        {summaryRows.map((row) => (
          <div key={row.label} className="rounded border border-zinc-800 bg-zinc-950/75 p-3">
            <p className="text-[10px] font-black uppercase tracking-widest text-zinc-600">{row.label}</p>
            <p className={`mt-2 break-all font-mono text-sm font-black ${row.tone}`}>{row.value}</p>
          </div>
        ))}
      </div>

      <div className="mt-4 grid grid-cols-1 gap-3">
        <E2EAccountLinks result={result} />
        <AccountDetailSection summary="Show raw project account" title="GreenLabelProjectV1" rows={getProjectRows(result.project)} />
        <AccountDetailSection summary="Show raw dispute account" title="GreenLabelDisputeV1" rows={getDisputeRows(result.dispute)} />
      </div>
    </div>
  );
}

function getE2ESummaryRows(result: GreenLabelE2EResult, statusMatches: boolean): SummaryRow[] {
  const sharedRows: SummaryRow[] = [
    {
      label: 'Status',
      value: result.project.status,
      tone: result.project.status === result.expectedProjectStatus ? 'text-emerald-300' : 'text-yellow-200',
    },
    {
      label: 'Dispute',
      value: result.dispute.status,
      tone: result.dispute.status === result.expectedDisputeStatus ? 'text-emerald-300' : 'text-yellow-200',
    },
    { label: 'Bond', value: formatUsdcAmount(result.project.totalBondAmount), tone: 'text-cyan-300' },
  ];

  if (result.key === 'refund') {
    return [
      ...sharedRows,
      { label: 'Treasury Delta', value: '+0.2 USDC', tone: 'text-yellow-300' },
      { label: 'Green Bond Vault', value: '0', tone: 'text-zinc-100' },
      { label: 'Path', value: statusMatches ? 'Path Verified' : 'Needs Review', tone: statusMatches ? 'text-emerald-300' : 'text-yellow-200' },
    ];
  }

  return [
    ...sharedRows,
    { label: 'Relief/Risk Vault Delta', value: '+1 USDC', tone: 'text-orange-300' },
    { label: 'Green Bond Vault', value: '0', tone: 'text-zinc-100' },
    { label: 'Path', value: statusMatches ? 'Path Verified' : 'Needs Review', tone: statusMatches ? 'text-emerald-300' : 'text-yellow-200' },
  ];
}

function E2EAccountLinks({ result }: { result: GreenLabelE2EResult }) {
  const links = [
    { label: 'Project account', address: result.project.account },
    { label: 'Dispute account', address: result.dispute.account },
    { label: 'Bond vault', address: result.project.bondVault },
    { label: 'Terminal proposal decision', address: result.project.terminalProposalDecision },
    { label: 'Terminal execution queue item', address: result.project.terminalExecutionQueueItem },
  ];

  return (
    <div className="rounded border border-zinc-800 bg-zinc-950/75 p-3">
      <p className="mb-2 text-[10px] font-black uppercase tracking-widest text-zinc-600">Explorer links</p>
      <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
        {links.map((link) => (
          <ExplorerAddressLink key={`${link.label}-${link.address}`} label={link.label} address={link.address} />
        ))}
      </div>
    </div>
  );
}

function AccountDetailSection({ summary, title, rows }: { summary: string; title: string; rows: DetailRow[] }) {
  return (
    <details className="rounded border border-zinc-800 bg-zinc-950/75 p-3">
      <summary className="cursor-pointer text-xs font-black text-zinc-300 hover:text-cyan-300">
        {summary}
      </summary>
      <p className="mb-3 mt-3 text-[10px] font-black uppercase tracking-widest text-zinc-600">{title}</p>
      <div className="grid grid-cols-1 gap-2">
        {rows.map((row) => (
          <DetailRowItem key={row.label} row={row} />
        ))}
      </div>
    </details>
  );
}

function DetailRowItem({ row }: { row: DetailRow }) {
  return (
    <div className="grid grid-cols-1 gap-1 rounded border border-zinc-800 bg-zinc-950/80 px-3 py-2 text-xs sm:grid-cols-[minmax(0,0.75fr)_minmax(0,1.25fr)]">
      <span className="break-all font-mono font-bold text-zinc-500">{row.label}</span>
      <span className="min-w-0 font-mono font-black text-zinc-100 sm:text-right">
        {row.href ? (
          <a
            href={row.href}
            target="_blank"
            rel="noreferrer"
            className="inline-flex max-w-full items-center justify-end gap-1.5 break-all text-cyan-300 hover:text-cyan-200"
          >
            <span className="break-all">{row.value}</span>
            <ExternalLink className="h-3 w-3 flex-shrink-0" />
          </a>
        ) : (
          <span className="break-all">{row.value}</span>
        )}
        {row.caption && (
          <span className="mt-1 block break-all text-[10px] font-bold text-zinc-500">{row.caption}</span>
        )}
      </span>
    </div>
  );
}

function ExplorerAddressLink({ label, address }: { label: string; address: string }) {
  return (
    <a
      href={getGreenLabelExplorerAddressUrl(address)}
      target="_blank"
      rel="noreferrer"
      className="flex min-w-0 items-center justify-between gap-2 rounded border border-zinc-800 bg-zinc-950/80 px-2 py-1.5 text-[10px] font-bold text-zinc-300 transition-all hover:border-cyan-400/40 hover:text-cyan-300"
    >
      <span>{label}</span>
      <span className="flex min-w-0 items-center gap-1.5">
        <span className="truncate font-mono text-zinc-500">{shortAddress(address)}</span>
        <ExternalLink className="h-3 w-3 flex-shrink-0" />
      </span>
    </a>
  );
}

function getProjectRows(project: GreenLabelProjectV1): DetailRow[] {
  return [
    { label: 'project_id', value: project.projectId.toString() },
    { label: 'project_owner', value: project.projectOwner, href: getGreenLabelExplorerAddressUrl(project.projectOwner) },
    { label: 'token_mint', value: project.tokenMint, href: getGreenLabelExplorerAddressUrl(project.tokenMint) },
    { label: 'project_treasury_wallet', value: project.projectTreasuryWallet, href: getGreenLabelExplorerAddressUrl(project.projectTreasuryWallet) },
    { label: 'base_bond_amount', value: formatUsdcAmount(project.baseBondAmount) },
    { label: 'extra_bond_amount', value: formatUsdcAmount(project.extraBondAmount) },
    { label: 'total_bond_amount', value: formatUsdcAmount(project.totalBondAmount) },
    { label: 'bond_vault', value: project.bondVault, href: getGreenLabelExplorerAddressUrl(project.bondVault) },
    { label: 'bond_vault_authority', value: project.bondVaultAuthority, href: getGreenLabelExplorerAddressUrl(project.bondVaultAuthority) },
    { label: 'bond_tier', value: formatEnumLabel(project.bondTier, BOND_TIER_LABELS) },
    { label: 'status', value: formatEnumLabel(project.status, PROJECT_STATUS_LABELS) },
    timestampRow('observation_start_ts', project.observationStartTs),
    timestampRow('observation_end_ts', project.observationEndTs),
    { label: 'dispute_count', value: project.disputeCount.toString() },
    { label: 'active_dispute', value: project.activeDispute, href: getGreenLabelExplorerAddressUrl(project.activeDispute) },
    timestampRow('refunded_at', project.refundedAt),
    timestampRow('slashed_at', project.slashedAt),
    { label: 'terminal_proposal_id', value: project.terminalProposalId.toString() },
    { label: 'terminal_proposal_decision', value: project.terminalProposalDecision, href: getGreenLabelExplorerAddressUrl(project.terminalProposalDecision) },
    { label: 'terminal_execution_queue_item', value: project.terminalExecutionQueueItem, href: getGreenLabelExplorerAddressUrl(project.terminalExecutionQueueItem) },
    { label: 'terminal_payload_hash', value: project.terminalPayloadHash },
    { label: 'terminal_action_type', value: formatEnumLabel(project.terminalActionType, ACTION_TYPE_LABELS) },
  ];
}

function getDisputeRows(dispute: GreenLabelDisputeV1): DetailRow[] {
  return [
    { label: 'project_id', value: dispute.projectId.toString() },
    { label: 'dispute_id', value: dispute.disputeId.toString() },
    { label: 'project', value: dispute.project, href: getGreenLabelExplorerAddressUrl(dispute.project) },
    { label: 'disputer', value: dispute.disputer, href: getGreenLabelExplorerAddressUrl(dispute.disputer) },
    { label: 'reason_code', value: formatEnumLabel(dispute.reasonCode, RUG_REASON_LABELS) },
    { label: 'status', value: formatEnumLabel(dispute.status, DISPUTE_STATUS_LABELS) },
    timestampRow('opened_at', dispute.openedAt),
    timestampRow('evidence_end_ts', dispute.evidenceEndTs),
    timestampRow('response_end_ts', dispute.responseEndTs),
    timestampRow('resolved_at', dispute.resolvedAt),
    { label: 'proposal_id', value: dispute.proposalId.toString() },
    { label: 'proposal_decision', value: dispute.proposalDecision, href: getGreenLabelExplorerAddressUrl(dispute.proposalDecision) },
    { label: 'execution_queue_item', value: dispute.executionQueueItem, href: getGreenLabelExplorerAddressUrl(dispute.executionQueueItem) },
    { label: 'payload_hash', value: dispute.payloadHash },
    { label: 'action_type', value: formatEnumLabel(dispute.actionType, ACTION_TYPE_LABELS) },
  ];
}

function timestampRow(label: string, timestamp: bigint): DetailRow {
  return {
    label,
    value: `${timestamp.toString()} unix`,
    caption: formatUnixTimestamp(timestamp),
  };
}

function formatEnumLabel(value: string, labels: Record<string, string>): string {
  return `${value} / ${labels[value] ?? '未知'}`;
}

function shortAddress(address: string): string {
  if (address.length <= 12) return address;
  return `${address.slice(0, 4)}...${address.slice(-4)}`;
}

interface DetailRow {
  label: string;
  value: string;
  caption?: string;
  href?: string;
}

interface SummaryRow {
  label: string;
  value: string;
  tone: string;
}

function ParameterGrid({
  items,
  tone,
}: {
  items: { label: string; value: string; caption?: string }[];
  tone: 'red' | 'emerald';
}) {
  const toneClass = tone === 'red'
    ? 'border-red-400/25 bg-zinc-950/70 text-red-200'
    : 'border-emerald-400/25 bg-zinc-950/70 text-emerald-200';

  return (
    <div className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2">
      {items.map((item) => (
        <div key={item.label} className={`rounded border p-3 ${toneClass}`}>
          <p className="break-all font-mono text-[10px] font-bold uppercase tracking-widest text-zinc-500">
            {item.label}
          </p>
          <p className="mt-2 font-mono text-xl font-black tabular-nums">{item.value}</p>
          {item.caption && <p className="mt-1 text-xs font-bold text-zinc-400">{item.caption}</p>}
        </div>
      ))}
    </div>
  );
}

function MilestoneCard({
  icon: Icon,
  title,
  tone,
  rows,
  note,
}: {
  icon: ElementType;
  title: string;
  tone: 'emerald' | 'red';
  rows: [string, string][];
  note: string;
}) {
  const toneClass = tone === 'emerald'
    ? 'border-emerald-400/20 bg-emerald-400/5 text-emerald-300'
    : 'border-red-400/25 bg-red-400/5 text-red-300';

  return (
    <div className={`rounded-xl border p-5 ${toneClass}`}>
      <div className="mb-4 flex items-center gap-2">
        <Icon className="h-5 w-5" />
        <h3 className="text-lg font-black text-zinc-100">{title}</h3>
      </div>
      <div className="space-y-2">
        {rows.map(([label, value]) => (
          <div
            key={label}
            className="grid grid-cols-1 gap-1 rounded border border-zinc-800 bg-zinc-950/75 px-3 py-2 text-xs sm:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]"
          >
            <span className="break-all font-mono font-bold text-zinc-500">{label}</span>
            <span className="break-all font-mono font-black text-zinc-100 sm:text-right">{value}</span>
          </div>
        ))}
      </div>
      <p className="mt-4 rounded border border-zinc-800 bg-zinc-950/60 px-3 py-2 text-xs font-bold leading-relaxed text-zinc-300">
        {note}
      </p>
    </div>
  );
}
