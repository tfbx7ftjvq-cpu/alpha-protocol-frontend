import { type ElementType } from 'react';
import {
  AlertTriangle,
  CheckCircle2,
  ExternalLink,
  FileCheck2,
  Gauge,
  Landmark,
  LockKeyhole,
  ShieldAlert,
  ShieldCheck,
} from 'lucide-react';

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
  'Mainnet 前必须恢复 299U / 30天 / 7天 / 3天。',
  'config authority 不能长期由单钱包控制，应迁移到 DAO / multisig / Security Layer timelock。',
  'Devnet-only scripts 不可用于 Mainnet。',
  'update config 权限需要审计。',
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

export default function GreenLabelDashboard() {
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
          <div className="mt-4 grid grid-cols-1 gap-2 md:grid-cols-2">
            {VERIFIED_PATH.map((item) => (
              <div
                key={item}
                className="flex min-w-0 items-start gap-2 rounded border border-zinc-800 bg-zinc-950/70 px-3 py-2 text-xs font-bold text-zinc-300"
              >
                <CheckCircle2 className="mt-0.5 h-3.5 w-3.5 flex-shrink-0 text-emerald-400" />
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
