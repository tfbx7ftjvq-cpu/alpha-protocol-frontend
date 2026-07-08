import { type ElementType } from 'react';
import {
  AlertTriangle,
  ArrowRight,
  CheckCircle2,
  Coins,
  ExternalLink,
  Gauge,
  HandHeart,
  Hammer,
  Landmark,
  Loader2,
  RefreshCw,
  ShieldCheck,
  Users,
  Vote,
} from 'lucide-react';
import { useTreasuryV2 } from '../hooks/useTreasuryV2';
import {
  TREASURY_V2_DEVNET_RPC_ENDPOINT,
  formatTreasuryUsdcAmount,
  getTreasuryV2ExplorerAddressUrl,
  type TreasuryV2Overview,
  type TreasuryV2VaultRead,
} from '../lib/treasuryV2';

const TREASURY_STATE_ADDRESS = '5e7eyC5ViwH9GBn73cY6so7J6KpRCX6XsbxozHabk2fE';

const REVENUE_SPLIT = [
  {
    label: 'Relief Pool',
    ratio: '50%',
    purpose: 'Victim protection, risk response, Green Label slash proceeds, and future DAO-reviewed relief policy.',
    tone: 'border-emerald-400/25 bg-emerald-400/5 text-emerald-300',
    icon: HandHeart,
  },
  {
    label: 'Buyback / Burn',
    ratio: '20%',
    purpose: 'Protocol-level buyback or burn policy, subject to future governance and execution controls.',
    tone: 'border-red-400/25 bg-red-400/5 text-red-300',
    icon: Coins,
  },
  {
    label: 'Builders / Contributors',
    ratio: '20%',
    purpose: 'Contributors, maintenance, audits, ecosystem operations, and DAO-approved spending.',
    tone: 'border-blue-400/25 bg-blue-400/5 text-blue-300',
    icon: Hammer,
  },
  {
    label: 'Staking Rewards',
    ratio: '10%',
    purpose: 'Protocol-rule-based incentives funded by the staking rewards pool. No guaranteed APY.',
    tone: 'border-yellow-400/25 bg-yellow-400/5 text-yellow-300',
    icon: ShieldCheck,
  },
] as const;

const TOKEN_UTILITY = [
  'DAO governance participation',
  'Staking participation',
  'Protocol incentives',
  'Contributor / builders coordination',
  'Green Label ecosystem alignment',
  'Risk reporting / dispute governance participation',
];

const RISK_NOTES = [
  'Staking rewards are protocol-rule-based incentives, not guaranteed returns.',
  'Green Label is not insurance, not a credit rating, and not investment advice.',
  'ALPHA token utility is subject to future governance and protocol development.',
];

const FLOW_STAGES = [
  {
    title: 'Revenue Source',
    text: 'Green Label fees, protocol services, future creator fees, and ecosystem revenue.',
    icon: Coins,
    tone: 'border-cyan-400/25 bg-cyan-400/5 text-cyan-300',
  },
  {
    title: 'Treasury V2',
    text: 'Transparent on-chain accounting for USDC revenue and split totals.',
    icon: Landmark,
    tone: 'border-emerald-400/25 bg-emerald-400/5 text-emerald-300',
  },
  {
    title: '50 / 20 / 20 / 10 Split',
    text: 'Relief, buyback/burn, builders/contributors, and staking rewards.',
    icon: Gauge,
    tone: 'border-yellow-400/25 bg-yellow-400/5 text-yellow-300',
  },
  {
    title: 'Governance Oversight',
    text: 'DAO, Security Layer, timelock, and queue guard critical execution paths.',
    icon: Vote,
    tone: 'border-violet-400/25 bg-violet-400/5 text-violet-300',
  },
] as const;

export default function TokenRevenueDashboard() {
  const { error, lastLoadedAt, overview, refresh, status } = useTreasuryV2();
  const isLoading = status === 'loading';

  return (
    <section className="space-y-6">
      <div className="rounded-xl border border-cyan-400/20 bg-cyan-400/5 p-5">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div className="min-w-0 space-y-3">
            <div className="flex flex-wrap items-center gap-2">
              <Badge icon={Coins} label="Token & Revenue" tone="cyan" />
              <Badge icon={ShieldCheck} label="Devnet" tone="emerald" />
              <Badge icon={Gauge} label="Read-only Public MVP" tone="zinc" />
            </div>
            <div>
              <h3 className="text-xl font-black uppercase tracking-wide text-zinc-100">
                ALPHA Token / Protocol Revenue Loop
              </h3>
              <p className="mt-2 max-w-4xl text-xs leading-relaxed text-zinc-400">
                Productized read-only view of how ALPHA, protocol revenue, Treasury V2, staking rewards,
                builders funding, relief reserves, Green Label outcomes, and DAO oversight connect into one
                protocol loop.
              </p>
            </div>
          </div>

          <a
            href={getTreasuryV2ExplorerAddressUrl(TREASURY_STATE_ADDRESS)}
            target="_blank"
            rel="noreferrer"
            className="inline-flex w-fit items-center justify-center gap-2 rounded border border-cyan-400/30 bg-cyan-400/10 px-4 py-2 text-xs font-bold text-cyan-300 transition-all hover:bg-cyan-400/15"
          >
            <ExternalLink className="h-3.5 w-3.5" />
            Treasury State
          </a>
        </div>
      </div>

      <TreasuryV2Card
        error={error}
        isLoading={isLoading}
        lastLoadedAt={lastLoadedAt}
        overview={overview}
        onRefresh={() => void refresh()}
        status={status}
      />

      <RevenueFlow />

      <div className="grid grid-cols-1 gap-4 xl:grid-cols-[minmax(0,0.95fr)_minmax(0,1.05fr)]">
        <TokenUtilityCard />
        <RevenueSplitCard />
      </div>

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
        <NarrativeCard
          icon={ShieldCheck}
          title="Staking Rewards"
          badge="Devnet verified"
          tone="yellow"
          items={[
            'Staking V1 has verified stake / claim / unstake paths on Devnet.',
            'The staking rewards pool receives 10% of the protocol revenue split.',
            'Rewards depend on pool balance and protocol rules.',
            'No guaranteed APY or fixed yield is presented.',
          ]}
        />
        <NarrativeCard
          icon={Hammer}
          title="Builders / Contributors"
          badge="20% pool"
          tone="blue"
          items={[
            'Supports contributors, protocol maintenance, audits, and ecosystem operations.',
            'Future spending should be DAO / governance controlled.',
            'Builders pool funds should not be controlled arbitrarily.',
          ]}
        />
        <NarrativeCard
          icon={HandHeart}
          title="Relief Pool"
          badge="50% pool"
          tone="emerald"
          items={[
            'Supports victim protection, risk response, and future DAO-reviewed relief policy.',
            'Green Label slash proceeds can route into risk / relief reserves.',
            'The relief pool does not mean automatic insurance.',
          ]}
        />
      </div>

      <section className="rounded-xl border border-red-400/25 bg-red-400/5 p-5">
        <div className="flex items-start gap-3">
          <AlertTriangle className="mt-0.5 h-5 w-5 flex-shrink-0 text-red-300" />
          <div>
            <h3 className="text-lg font-black text-red-100">Risk and Messaging Boundary</h3>
            <div className="mt-3 grid grid-cols-1 gap-2 md:grid-cols-3">
              {RISK_NOTES.map((note) => (
                <div key={note} className="rounded border border-red-400/20 bg-zinc-950/50 px-3 py-2 text-xs leading-relaxed text-red-100/80">
                  {note}
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>
    </section>
  );
}

function TreasuryV2Card({
  error,
  isLoading,
  lastLoadedAt,
  overview,
  onRefresh,
  status,
}: {
  error: string | null;
  isLoading: boolean;
  lastLoadedAt: Date | null;
  overview: TreasuryV2Overview | null;
  onRefresh: () => void;
  status: 'idle' | 'loading' | 'ready' | 'error';
}) {
  const state = overview?.state ?? null;
  const stateFields = [
    { label: 'total_usdc_inflow', value: state?.totalUsdcInflow },
    { label: 'relief_usdc_total', value: state?.reliefUsdcTotal },
    { label: 'buyback_usdc_total', value: state?.buybackUsdcTotal },
    { label: 'builders_usdc_total', value: state?.buildersUsdcTotal },
    { label: 'staking_usdc_total', value: state?.stakingUsdcTotal },
  ];

  return (
    <section className="rounded-xl border border-emerald-400/20 bg-emerald-400/5 p-5">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <SectionHeader
          icon={Landmark}
          eyebrow="Treasury V2 On-chain Read"
          title="USDC four-pool treasury accounting"
          description="Reads the Devnet Treasury USDC State V2 account and four SPL Token vault balances. This card is read-only and does not require wallet signatures."
        />
        <div className="flex flex-col items-start gap-2 lg:items-end">
          <StatusPill status={status} />
          <button
            type="button"
            onClick={onRefresh}
            disabled={isLoading}
            className="inline-flex items-center justify-center gap-2 rounded border border-emerald-400/30 bg-emerald-400/10 px-4 py-2 text-xs font-bold text-emerald-300 transition-all hover:bg-emerald-400/15 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {isLoading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <RefreshCw className="h-3.5 w-3.5" />}
            {isLoading ? 'Loading...' : 'Refresh'}
          </button>
        </div>
      </div>

      {(error || overview?.stateError) && (
        <div className="mt-4 rounded border border-red-400/30 bg-red-400/10 px-3 py-2 text-xs leading-relaxed text-red-200">
          {error && <p className="break-words">Treasury read failed: {error}</p>}
          {overview?.stateError && <p className="break-words">State decode warning: {overview.stateError}</p>}
        </div>
      )}

      <div className="mt-4 grid grid-cols-1 gap-3 lg:grid-cols-3">
        <AddressTile label="Treasury USDC State V2" value={overview?.treasuryUsdcState ?? TREASURY_STATE_ADDRESS} />
        <AddressTile label="USDC Mint" value={overview?.usdcMint ?? 'loading'} />
        <AddressTile label="Vault authority" value={overview?.vaultAuthority ?? 'loading'} />
      </div>

      <div className="mt-4 grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-5">
        {stateFields.map((field) => (
          <div key={field.label} className="rounded border border-zinc-800 bg-zinc-950/70 p-4">
            <p className="text-[10px] font-black uppercase tracking-widest text-zinc-600">{field.label}</p>
            <p className="mt-3 font-mono text-xl font-black text-emerald-300">
              {formatTreasuryUsdcAmount(field.value)}
            </p>
          </div>
        ))}
      </div>

      <div className="mt-4 grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-4">
        {(overview?.vaults ?? []).map((vault) => (
          <VaultBalanceCard key={vault.key} vault={vault} />
        ))}
        {!overview && [0, 1, 2, 3].map((item) => (
          <div key={item} className="rounded border border-zinc-800 bg-zinc-950/70 p-4">
            <p className="h-4 w-32 rounded bg-zinc-800/80" />
            <p className="mt-4 h-8 w-24 rounded bg-zinc-800/60" />
            <p className="mt-3 h-3 w-full rounded bg-zinc-900" />
          </div>
        ))}
      </div>

      <div className="mt-4 flex flex-col gap-2 border-t border-zinc-800 pt-4 text-[10px] text-zinc-600 md:flex-row md:items-center md:justify-between">
        <span>RPC: {TREASURY_V2_DEVNET_RPC_ENDPOINT}</span>
        <span>Last loaded: {lastLoadedAt ? lastLoadedAt.toLocaleString() : 'not loaded yet'}</span>
      </div>
    </section>
  );
}

function RevenueFlow() {
  return (
    <section className="rounded-xl border border-zinc-800 bg-zinc-950/50 p-5">
      <SectionHeader
        icon={ArrowRight}
        eyebrow="Revenue Flow"
        title="Protocol revenue to governed allocation"
        description="The product loop is intentionally simple: revenue enters transparent accounting, splits into four pools, and remains under DAO / Security Layer oversight."
      />
      <div className="mt-4 grid grid-cols-1 gap-3 xl:grid-cols-4">
        {FLOW_STAGES.map((stage, index) => (
          <div key={stage.title} className="flex flex-col gap-3 xl:flex-row">
            <FlowStage stage={stage} step={index + 1} />
            {index < FLOW_STAGES.length - 1 && (
              <div className="hidden items-center justify-center xl:flex">
                <ArrowRight className="h-5 w-5 text-zinc-700" />
              </div>
            )}
          </div>
        ))}
      </div>
    </section>
  );
}

function TokenUtilityCard() {
  return (
    <section className="rounded-xl border border-cyan-400/20 bg-cyan-400/5 p-5">
      <SectionHeader
        icon={Users}
        eyebrow="Token Utility"
        title="Why ALPHA exists"
        description="ALPHA is positioned as the coordination asset for governance, staking participation, protocol incentives, contributor alignment, Green Label ecosystem behavior, and dispute governance."
      />
      <div className="mt-4 grid grid-cols-1 gap-2 sm:grid-cols-2">
        {TOKEN_UTILITY.map((item) => (
          <div key={item} className="flex items-start gap-2 rounded border border-cyan-400/15 bg-zinc-950/60 px-3 py-2 text-xs text-cyan-100">
            <CheckCircle2 className="mt-0.5 h-3.5 w-3.5 flex-shrink-0 text-cyan-300" />
            <span>{item}</span>
          </div>
        ))}
      </div>
    </section>
  );
}

function RevenueSplitCard() {
  return (
    <section className="rounded-xl border border-zinc-800 bg-zinc-950/50 p-5">
      <SectionHeader
        icon={Gauge}
        eyebrow="Treasury V2 Split"
        title="50% / 20% / 20% / 10%"
        description="Treasury V2 has verified the USDC split on Devnet. Future Mainnet policy should remain governed and auditable."
      />
      <div className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2">
        {REVENUE_SPLIT.map((item) => {
          const Icon = item.icon;

          return (
            <div key={item.label} className={`rounded border p-4 ${item.tone}`}>
              <div className="flex items-start justify-between gap-3">
                <div>
                  <p className="text-sm font-black text-zinc-100">{item.label}</p>
                  <p className="mt-2 text-xs leading-relaxed text-zinc-400">{item.purpose}</p>
                </div>
                <div className="flex h-12 w-12 flex-shrink-0 items-center justify-center rounded border border-current/30 bg-zinc-950/50">
                  <Icon className="h-5 w-5" />
                </div>
              </div>
              <p className="mt-4 font-mono text-3xl font-black">{item.ratio}</p>
            </div>
          );
        })}
      </div>
    </section>
  );
}

function NarrativeCard({
  badge,
  icon: Icon,
  items,
  title,
  tone,
}: {
  badge: string;
  icon: ElementType;
  items: string[];
  title: string;
  tone: 'blue' | 'emerald' | 'yellow';
}) {
  const toneClass = {
    blue: 'border-blue-400/20 bg-blue-400/5 text-blue-300',
    emerald: 'border-emerald-400/20 bg-emerald-400/5 text-emerald-300',
    yellow: 'border-yellow-400/20 bg-yellow-400/5 text-yellow-300',
  }[tone];

  return (
    <section className={`rounded-xl border p-5 ${toneClass}`}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-[10px] font-black uppercase tracking-widest text-current/80">{badge}</p>
          <h3 className="mt-2 text-lg font-black text-zinc-100">{title}</h3>
        </div>
        <Icon className="h-5 w-5 flex-shrink-0" />
      </div>
      <div className="mt-4 space-y-2">
        {items.map((item) => (
          <div key={item} className="flex items-start gap-2 text-xs leading-relaxed text-zinc-300">
            <CheckCircle2 className="mt-0.5 h-3.5 w-3.5 flex-shrink-0 text-current" />
            <span>{item}</span>
          </div>
        ))}
      </div>
    </section>
  );
}

function FlowStage({
  stage,
  step,
}: {
  stage: typeof FLOW_STAGES[number];
  step: number;
}) {
  const Icon = stage.icon;

  return (
    <div className={`min-h-full flex-1 rounded border p-4 ${stage.tone}`}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-mono text-[10px] font-black uppercase tracking-widest text-current/80">
            Step {step}
          </p>
          <h4 className="mt-2 text-base font-black text-zinc-100">{stage.title}</h4>
        </div>
        <Icon className="h-5 w-5 flex-shrink-0" />
      </div>
      <p className="mt-3 text-xs leading-relaxed text-zinc-400">{stage.text}</p>
    </div>
  );
}

function VaultBalanceCard({ vault }: { vault: TreasuryV2VaultRead }) {
  const balanceText = vault.balanceUi ? `${vault.balanceUi} USDC` : 'unavailable';
  const vaultTones: Record<TreasuryV2VaultRead['key'], string> = {
    relief: 'border-emerald-400/25 bg-emerald-400/5 text-emerald-300',
    buyback: 'border-red-400/25 bg-red-400/5 text-red-300',
    builders: 'border-blue-400/25 bg-blue-400/5 text-blue-300',
    staking: 'border-yellow-400/25 bg-yellow-400/5 text-yellow-300',
  };

  return (
    <div className={`rounded border p-4 ${vaultTones[vault.key]}`}>
      <p className="min-h-10 text-xs font-black leading-snug text-zinc-100">{vault.label}</p>
      <p className="mt-3 font-mono text-2xl font-black tabular-nums">{balanceText}</p>
      <p className="mt-1 text-[10px] text-zinc-500">decimals: {vault.decimals ?? 'unknown'}</p>
      {vault.error && (
        <p className="mt-2 rounded border border-red-400/20 bg-red-400/10 px-2 py-1 text-[10px] leading-relaxed text-red-200">
          {vault.error}
        </p>
      )}
      <a
        href={vault.explorerUrl}
        target="_blank"
        rel="noreferrer"
        className="mt-3 inline-flex max-w-full items-center gap-1.5 rounded border border-zinc-700 bg-zinc-950/60 px-2 py-1 text-[10px] font-bold text-zinc-300 transition-all hover:border-cyan-400/40 hover:text-cyan-300"
      >
        <ExternalLink className="h-3 w-3 flex-shrink-0" />
        <span className="truncate font-mono">{shortAddress(vault.address)}</span>
      </a>
    </div>
  );
}

function AddressTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded border border-zinc-800 bg-zinc-950/70 p-4">
      <p className="text-[10px] font-black uppercase tracking-widest text-zinc-600">{label}</p>
      <a
        href={value === 'loading' ? undefined : getTreasuryV2ExplorerAddressUrl(value)}
        target="_blank"
        rel="noreferrer"
        className="mt-2 block break-all font-mono text-xs font-bold leading-relaxed text-cyan-300 hover:text-cyan-200"
      >
        {value}
      </a>
    </div>
  );
}

function StatusPill({ status }: { status: 'idle' | 'loading' | 'ready' | 'error' }) {
  const meta = {
    idle: 'border-zinc-700 bg-zinc-900/60 text-zinc-400',
    loading: 'border-yellow-400/30 bg-yellow-400/10 text-yellow-300',
    ready: 'border-emerald-400/30 bg-emerald-400/10 text-emerald-300',
    error: 'border-red-400/30 bg-red-400/10 text-red-300',
  }[status];

  return (
    <span className={`inline-flex items-center gap-2 rounded border px-3 py-1.5 text-xs font-bold ${meta}`}>
      {status === 'loading' && <Loader2 className="h-3.5 w-3.5 animate-spin" />}
      {status === 'ready' ? 'Treasury synced' : status}
    </span>
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
      <div className="mb-2 flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-emerald-400">
        <Icon className="h-3.5 w-3.5" />
        {eyebrow}
      </div>
      <h3 className="text-lg font-black text-zinc-100">{title}</h3>
      <p className="mt-2 text-xs leading-relaxed text-zinc-400">{description}</p>
    </div>
  );
}

function Badge({
  icon: Icon,
  label,
  tone,
}: {
  icon: ElementType;
  label: string;
  tone: 'cyan' | 'emerald' | 'zinc';
}) {
  const className = {
    cyan: 'border-cyan-400/25 bg-cyan-400/10 text-cyan-300',
    emerald: 'border-emerald-400/25 bg-emerald-400/10 text-emerald-300',
    zinc: 'border-zinc-700 bg-zinc-900/60 text-zinc-400',
  }[tone];

  return (
    <span className={`inline-flex items-center gap-1.5 rounded border px-2 py-1 text-[10px] font-bold uppercase tracking-widest ${className}`}>
      <Icon className="h-3 w-3" />
      {label}
    </span>
  );
}

function shortAddress(address: string): string {
  if (address.length <= 14) return address;
  return `${address.slice(0, 6)}...${address.slice(-6)}`;
}
