import { type ElementType } from 'react';
import {
  AlertTriangle,
  ArrowRight,
  BookOpen,
  CheckCircle2,
  FileText,
  Gauge,
  HandHeart,
  Landmark,
  LockKeyhole,
  ShieldCheck,
  TimerReset,
  Users,
  Vote,
} from 'lucide-react';

export type PublicLandingTarget = 'treasury' | 'shame' | 'greenLabel' | 'tokenRevenue';

interface PublicLandingPageProps {
  onNavigate: (target: PublicLandingTarget) => void;
}

const STATUS_BADGES = [
  { label: 'Devnet Verified', tone: 'emerald' },
  { label: 'Mainnet Not Live', tone: 'red' },
  { label: 'Read-only Public MVP', tone: 'cyan' },
  { label: 'No investment advice', tone: 'zinc' },
] as const;

const PROBLEMS = [
  '链上项目跑路、欺诈和信息不透明，让普通用户难以判断真实风险。',
  '普通用户缺少可验证的风险承诺机制，也缺少清晰的争议处理路径。',
  '项目方缺少透明的保证金、观察期、争议期和惩罚流程。',
  '协议收入和国库使用经常不透明，外部用户难以持续审查。',
  '很多 DAO 只有投票口号，缺少 timelock、queue、cancel、pause 等安全执行层。',
];

const SOLUTIONS = [
  {
    title: 'DAO Security Layer',
    text: 'Proposal decision / queue / timelock / cancel / pause 已在 Devnet 验证，用于保护敏感执行路径。',
    icon: LockKeyhole,
    tone: 'cyan',
  },
  {
    title: 'Green Label',
    text: '项目方锁定风险承诺金，经过观察期、争议期和治理裁决后执行 refund 或 slash。',
    icon: ShieldCheck,
    tone: 'emerald',
  },
  {
    title: 'Treasury V2',
    text: '协议 USDC 收入进入透明国库，并按 50/20/20/10 分流到 relief、buyback、builders、staking。',
    icon: Landmark,
    tone: 'yellow',
  },
  {
    title: 'Public Dashboards',
    text: '前端以只读方式展示链上状态，提高透明度，不提供写入按钮或交易入口。',
    icon: Gauge,
    tone: 'zinc',
  },
] as const;

const MODULES = [
  {
    title: 'DAO Governance',
    bullets: ['Security Layer V1 execution guard completed', 'Full ALPHA voting layer pending'],
    icon: Vote,
    tone: 'cyan',
  },
  {
    title: 'Green Label',
    bullets: ['Refund / slash E2E verified on Devnet', 'Not insurance, not credit rating'],
    icon: ShieldCheck,
    tone: 'emerald',
  },
  {
    title: 'Treasury V2',
    bullets: ['50% relief', '20% buyback / burn', '20% builders / contributors', '10% staking rewards'],
    icon: Landmark,
    tone: 'yellow',
  },
  {
    title: 'Staking V1',
    bullets: ['Stake / claim / unstake verified on Devnet', 'No guaranteed APY'],
    icon: TimerReset,
    tone: 'violet',
  },
  {
    title: 'Token / Revenue',
    bullets: ['Governance, staking, contributor coordination, ecosystem alignment', 'No price appreciation promise'],
    icon: HandHeart,
    tone: 'blue',
  },
] as const;

const COMPLETED = [
  'Treasury V2 Devnet USDC split verified',
  'Staking V1 Devnet stake / claim / unstake verified',
  'Security Layer V1 Devnet decision / queue / timelock / cancel / pause verified',
  'Green Label V1 Devnet refund / slash E2E verified',
  'DAO Governance Read-only Dashboard completed',
  'Token / Revenue / Treasury Dashboard completed',
  'Mainnet prelaunch safety framework completed',
];

const PENDING = [
  'Full ALPHA token voting layer',
  'Mainnet deployment / config',
  'Mainnet authority migration',
  'Mainnet sanity check',
  'Legal / risk review',
  'Public token launch',
];

const JOURNEYS = [
  {
    title: 'For community',
    items: ['Inspect treasury', 'Inspect DAO execution layer', 'Follow Green Label disputes', 'Participate in future governance'],
    icon: Users,
  },
  {
    title: 'For project teams',
    items: ['Apply for Green Label in future', 'Lock bond', 'Pass observation / dispute window', 'Earn accountability signal'],
    icon: ShieldCheck,
  },
  {
    title: 'For contributors',
    items: ['Help research risk', 'Build tools', 'Join DAO governance roadmap'],
    icon: BookOpen,
  },
] as const;

const RISK_DISCLOSURES = [
  'Alpha Protocol is not live on Mainnet.',
  'Green Label is not insurance.',
  'Green Label is not a credit rating.',
  'ALPHA does not guarantee profit, yield, dividend, or price appreciation.',
  'Staking rewards are protocol-rule-based incentives, not guaranteed returns.',
  'Relief pool does not mean automatic payout.',
  'DAO voting layer is not fully launched yet.',
  'Current dashboards are read-only Devnet / Public MVP views.',
];

const DOC_LINKS = {
  litepaper: '../docs/alpha-protocol-litepaper.md',
  goNoGo: '../docs/mainnet-go-no-go-checklist.md',
};

export default function PublicLandingPage({ onNavigate }: PublicLandingPageProps) {
  return (
    <div className="space-y-8">
      <HeroSection onNavigate={onNavigate} />

      <section className="grid grid-cols-1 gap-4 lg:grid-cols-[minmax(0,0.95fr)_minmax(0,1.05fr)]">
        <ProblemSection />
        <SolutionSection />
      </section>

      <ProtocolModulesSection />
      <CurrentStatusSection />
      <UserJourneySection />
      <RiskDisclosureSection />
      <FinalCtaSection onNavigate={onNavigate} />
    </div>
  );
}

function HeroSection({ onNavigate }: PublicLandingPageProps) {
  return (
    <section className="relative isolate min-h-[520px] overflow-hidden rounded-xl border border-zinc-800 bg-zinc-950 px-5 py-8 sm:px-8 lg:px-10">
      <div className="absolute inset-0 -z-10">
        <div className="absolute inset-0 bg-[linear-gradient(90deg,rgba(39,39,42,0.75)_1px,transparent_1px),linear-gradient(180deg,rgba(39,39,42,0.55)_1px,transparent_1px)] bg-[size:72px_72px]" />
        <div className="absolute left-0 right-0 top-16 h-px bg-cyan-400/30" />
        <div className="absolute left-0 right-0 top-32 h-px bg-emerald-400/20" />
        <div className="absolute left-0 right-0 top-48 h-px bg-yellow-400/20" />
        <div className="absolute bottom-0 left-0 right-0 grid h-40 grid-cols-4 border-t border-zinc-800/80">
          {['50% Relief', '20% Buyback', '20% Builders', '10% Staking'].map((label) => (
            <div key={label} className="flex items-end border-r border-zinc-800/80 p-3 last:border-r-0">
              <span className="text-[10px] font-black uppercase tracking-widest text-zinc-700">{label}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="max-w-4xl pt-6 lg:pt-12">
        <div className="flex flex-wrap gap-2">
          {STATUS_BADGES.map((badge) => (
            <StatusBadge key={badge.label} label={badge.label} tone={badge.tone} />
          ))}
        </div>

        <h2 className="mt-8 max-w-3xl text-4xl font-black leading-tight tracking-tight text-zinc-100 sm:text-5xl lg:text-6xl">
          Alpha Protocol
        </h2>
        <p className="mt-4 max-w-3xl text-base font-bold leading-relaxed text-cyan-100 sm:text-lg">
          DAO-governed protection infrastructure for on-chain risk, treasury transparency, and Green Label accountability.
        </p>
        <p className="mt-4 max-w-3xl text-sm leading-relaxed text-zinc-400">
          Alpha Protocol 是一个面向链上风险保护、透明国库分流、Green Label 风险承诺与 DAO 治理执行的协议。
          当前为 Devnet verified / read-only Public MVP，Mainnet 尚未上线。
        </p>

        <div className="mt-8 flex flex-wrap gap-3">
          <button
            type="button"
            onClick={() => onNavigate('shame')}
            className="inline-flex items-center gap-2 rounded border border-cyan-400/35 bg-cyan-400/10 px-4 py-2 text-xs font-black text-cyan-200 transition-all hover:bg-cyan-400/15"
          >
            Explore DAO Dashboard
            <ArrowRight className="h-3.5 w-3.5" />
          </button>
          <button
            type="button"
            onClick={() => onNavigate('greenLabel')}
            className="inline-flex items-center gap-2 rounded border border-emerald-400/35 bg-emerald-400/10 px-4 py-2 text-xs font-black text-emerald-200 transition-all hover:bg-emerald-400/15"
          >
            View Green Label Verification
            <ArrowRight className="h-3.5 w-3.5" />
          </button>
          <button
            type="button"
            onClick={() => onNavigate('tokenRevenue')}
            className="inline-flex items-center gap-2 rounded border border-yellow-400/35 bg-yellow-400/10 px-4 py-2 text-xs font-black text-yellow-200 transition-all hover:bg-yellow-400/15"
          >
            View Token & Revenue Flow
            <ArrowRight className="h-3.5 w-3.5" />
          </button>
        </div>
      </div>
    </section>
  );
}

function ProblemSection() {
  return (
    <section className="rounded-xl border border-red-400/20 bg-red-400/5 p-5">
      <SectionHeader
        icon={AlertTriangle}
        eyebrow="Problem"
        title="链上风险缺少可验证的承诺与执行"
        description="风险、国库和 DAO 执行往往分散在不同叙事里，普通用户难以审查。"
      />
      <div className="mt-4 space-y-2">
        {PROBLEMS.map((problem) => (
          <ListItem key={problem} icon={AlertTriangle} text={problem} tone="red" />
        ))}
      </div>
    </section>
  );
}

function SolutionSection() {
  return (
    <section className="rounded-xl border border-emerald-400/20 bg-emerald-400/5 p-5">
      <SectionHeader
        icon={ShieldCheck}
        eyebrow="Solution"
        title="Alpha Protocol 把风险、国库和治理执行放进同一条可验证闭环"
        description="当前公开页面展示的是 Devnet 已验证能力，不宣传 Mainnet 已上线，也不宣传完整 DAO 投票层已完成。"
      />
      <div className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2">
        {SOLUTIONS.map((item) => (
          <InfoCard key={item.title} {...item} />
        ))}
      </div>
    </section>
  );
}

function ProtocolModulesSection() {
  return (
    <section className="rounded-xl border border-zinc-800 bg-zinc-950/50 p-5">
      <SectionHeader
        icon={Landmark}
        eyebrow="Protocol Modules"
        title="核心模块"
        description="Public MVP 将现有 Devnet 验证结果组织成外部用户可以理解的协议模块。"
      />
      <div className="mt-4 grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-5">
        {MODULES.map((module) => {
          const Icon = module.icon;

          return (
            <div key={module.title} className={`rounded border p-4 ${toneClass(module.tone)}`}>
              <div className="flex items-start justify-between gap-3">
                <h4 className="text-sm font-black text-zinc-100">{module.title}</h4>
                <Icon className="h-4 w-4 flex-shrink-0" />
              </div>
              <div className="mt-4 space-y-2">
                {module.bullets.map((bullet) => (
                  <div key={bullet} className="flex items-start gap-2 text-[11px] leading-relaxed text-zinc-300">
                    <CheckCircle2 className="mt-0.5 h-3 w-3 flex-shrink-0 text-current" />
                    <span>{bullet}</span>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}

function CurrentStatusSection() {
  return (
    <section className="grid grid-cols-1 gap-4 lg:grid-cols-2">
      <StatusColumn title="Completed" items={COMPLETED} tone="emerald" />
      <StatusColumn title="Pending" items={PENDING} tone="yellow" />
    </section>
  );
}

function UserJourneySection() {
  return (
    <section className="rounded-xl border border-cyan-400/20 bg-cyan-400/5 p-5">
      <SectionHeader
        icon={Users}
        eyebrow="User Journey"
        title="不同参与者如何理解协议"
        description="当前阶段强调审查、理解和只读验证。未来写入流程必须经过产品、合约、安全和治理阶段。"
      />
      <div className="mt-4 grid grid-cols-1 gap-3 md:grid-cols-3">
        {JOURNEYS.map((journey) => {
          const Icon = journey.icon;

          return (
            <div key={journey.title} className="rounded border border-cyan-400/15 bg-zinc-950/60 p-4">
              <div className="flex items-center gap-2">
                <Icon className="h-4 w-4 text-cyan-300" />
                <h4 className="text-sm font-black text-zinc-100">{journey.title}</h4>
              </div>
              <div className="mt-4 space-y-2">
                {journey.items.map((item) => (
                  <ListItem key={item} icon={CheckCircle2} text={item} tone="cyan" />
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}

function RiskDisclosureSection() {
  return (
    <section className="rounded-xl border border-red-400/30 bg-red-400/10 p-5">
      <SectionHeader
        icon={AlertTriangle}
        eyebrow="Risk Disclosure"
        title="公开风险边界"
        description="这些边界必须在 Mainnet、token launch 或对外传播之前保持清晰。"
      />
      <div className="mt-4 grid grid-cols-1 gap-2 md:grid-cols-2">
        {RISK_DISCLOSURES.map((risk) => (
          <div key={risk} className="rounded border border-red-400/20 bg-zinc-950/60 px-3 py-2 text-xs leading-relaxed text-red-100">
            {risk}
          </div>
        ))}
      </div>
    </section>
  );
}

function FinalCtaSection({ onNavigate }: PublicLandingPageProps) {
  return (
    <section id="landing-litepaper-summary" className="rounded-xl border border-zinc-800 bg-zinc-950/50 p-5">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <SectionHeader
          icon={FileText}
          eyebrow="Next Reading"
          title="Litepaper 与上线清单"
          description="CTA 仅跳转前端只读页面或文档，不连接交易，不提供买币或真实资金入口。"
        />
        <div className="flex flex-wrap gap-2">
          <button
            type="button"
            onClick={() => onNavigate('treasury')}
            className="inline-flex items-center gap-2 rounded border border-emerald-400/30 bg-emerald-400/10 px-4 py-2 text-xs font-bold text-emerald-200 hover:bg-emerald-400/15"
          >
            Inspect Treasury
          </button>
          <a
            href={DOC_LINKS.litepaper}
            target="_blank"
            rel="noreferrer"
            className="inline-flex items-center gap-2 rounded border border-cyan-400/30 bg-cyan-400/10 px-4 py-2 text-xs font-bold text-cyan-200 hover:bg-cyan-400/15"
          >
            Read Litepaper
            <FileText className="h-3.5 w-3.5" />
          </a>
          <a
            href={DOC_LINKS.goNoGo}
            target="_blank"
            rel="noreferrer"
            className="inline-flex items-center gap-2 rounded border border-yellow-400/30 bg-yellow-400/10 px-4 py-2 text-xs font-bold text-yellow-200 hover:bg-yellow-400/15"
          >
            View Mainnet Go/No-Go Checklist
            <FileText className="h-3.5 w-3.5" />
          </a>
        </div>
      </div>
    </section>
  );
}

function StatusColumn({
  items,
  title,
  tone,
}: {
  items: string[];
  title: string;
  tone: 'emerald' | 'yellow';
}) {
  const className = tone === 'emerald'
    ? 'border-emerald-400/20 bg-emerald-400/5 text-emerald-300'
    : 'border-yellow-400/20 bg-yellow-400/5 text-yellow-300';

  return (
    <section className={`rounded-xl border p-5 ${className}`}>
      <h3 className="text-lg font-black text-zinc-100">{title}</h3>
      <div className="mt-4 space-y-2">
        {items.map((item) => (
          <ListItem key={item} icon={tone === 'emerald' ? CheckCircle2 : TimerReset} text={item} tone={tone} />
        ))}
      </div>
    </section>
  );
}

function InfoCard({
  icon: Icon,
  text,
  title,
  tone,
}: {
  icon: ElementType;
  text: string;
  title: string;
  tone: 'cyan' | 'emerald' | 'yellow' | 'zinc';
}) {
  return (
    <div className={`rounded border p-4 ${toneClass(tone)}`}>
      <div className="flex items-start justify-between gap-3">
        <h4 className="text-sm font-black text-zinc-100">{title}</h4>
        <Icon className="h-4 w-4 flex-shrink-0" />
      </div>
      <p className="mt-3 text-xs leading-relaxed text-zinc-300">{text}</p>
    </div>
  );
}

function SectionHeader({
  description,
  eyebrow,
  icon: Icon,
  title,
}: {
  description: string;
  eyebrow: string;
  icon: ElementType;
  title: string;
}) {
  return (
    <div className="max-w-4xl">
      <div className="mb-2 flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-cyan-300">
        <Icon className="h-3.5 w-3.5" />
        {eyebrow}
      </div>
      <h3 className="text-lg font-black text-zinc-100">{title}</h3>
      <p className="mt-2 text-xs leading-relaxed text-zinc-400">{description}</p>
    </div>
  );
}

function ListItem({
  icon: Icon,
  text,
  tone,
}: {
  icon: ElementType;
  text: string;
  tone: 'cyan' | 'emerald' | 'red' | 'yellow';
}) {
  const textClass = {
    cyan: 'text-cyan-200',
    emerald: 'text-emerald-200',
    red: 'text-red-100',
    yellow: 'text-yellow-100',
  }[tone];

  return (
    <div className="flex items-start gap-2 text-xs leading-relaxed text-zinc-300">
      <Icon className={`mt-0.5 h-3.5 w-3.5 flex-shrink-0 ${textClass}`} />
      <span>{text}</span>
    </div>
  );
}

function StatusBadge({ label, tone }: { label: string; tone: 'cyan' | 'emerald' | 'red' | 'zinc' }) {
  const className = {
    cyan: 'border-cyan-400/30 bg-cyan-400/10 text-cyan-200',
    emerald: 'border-emerald-400/30 bg-emerald-400/10 text-emerald-200',
    red: 'border-red-400/30 bg-red-400/10 text-red-200',
    zinc: 'border-zinc-700 bg-zinc-900/70 text-zinc-300',
  }[tone];

  return (
    <span className={`rounded border px-2.5 py-1 text-[10px] font-black uppercase tracking-widest ${className}`}>
      {label}
    </span>
  );
}

function toneClass(tone: 'blue' | 'cyan' | 'emerald' | 'violet' | 'yellow' | 'zinc'): string {
  return {
    blue: 'border-blue-400/20 bg-blue-400/5 text-blue-300',
    cyan: 'border-cyan-400/20 bg-cyan-400/5 text-cyan-300',
    emerald: 'border-emerald-400/20 bg-emerald-400/5 text-emerald-300',
    violet: 'border-violet-400/20 bg-violet-400/5 text-violet-300',
    yellow: 'border-yellow-400/20 bg-yellow-400/5 text-yellow-300',
    zinc: 'border-zinc-800 bg-zinc-950/70 text-zinc-400',
  }[tone];
}
