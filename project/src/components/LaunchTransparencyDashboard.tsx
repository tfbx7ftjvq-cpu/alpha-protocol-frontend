import {
  AlertTriangle,
  ArrowRight,
  CheckCircle2,
  ExternalLink,
  Landmark,
  Loader2,
  RefreshCw,
  Rocket,
  ShieldCheck,
  Wallet,
} from 'lucide-react';
import { useLaunchTransparency } from '../hooks/useLaunchTransparency';
import {
  REVENUE_SPLIT,
  getMainnetExplorerAddressUrl,
} from '../lib/launchTransparency';

const POOLS = [
  { key: 'relief', label: 'Victim Relief', ratio: REVENUE_SPLIT.relief, gate: '余额足够后由治理批准赔付' },
  { key: 'buyback', label: 'Buyback / Burn', ratio: REVENUE_SPLIT.buyback, gate: '达到批次门槛后公开执行' },
  { key: 'builders', label: 'Builders', ratio: REVENUE_SPLIT.builders, gate: '任务验收并经治理认可后支付' },
  { key: 'staking', label: 'Staking', ratio: REVENUE_SPLIT.staking, gate: '奖励池有真实余额后才开放' },
] as const;

export default function LaunchTransparencyDashboard() {
  const { config, error, lastLoadedAt, refresh, snapshot, status } = useLaunchTransparency();
  const distributable = snapshot?.usdcBalance ?? null;

  return (
    <section className="space-y-6">
      <section className="rounded-xl border border-yellow-400/25 bg-yellow-400/5 p-5">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <div className="flex flex-wrap gap-2">
              <Badge text="Mainnet launch layer" tone="yellow" />
              <Badge text={config.status === 'live' ? 'LIVE' : 'PRELAUNCH'} tone={config.status === 'live' ? 'green' : 'zinc'} />
              <Badge text="Zero-funded treasury" tone="cyan" />
            </div>
            <h2 className="mt-4 text-2xl font-black text-zinc-100">零初始国库 · 收入驱动运行</h2>
            <p className="mt-2 max-w-4xl text-xs leading-relaxed text-zinc-400">
              国库不预注资。代币发射后产生的平台 Creator Rewards 先进入公开收入钱包，
              收入资产确认或兑换为 USDC 后，再按 50/20/20/10 分配。任何板块余额不足时不垫资、不负债、不承诺固定收益。
            </p>
          </div>
          {config.pumpUrl ? (
            <a href={config.pumpUrl} target="_blank" rel="noreferrer" className="inline-flex items-center gap-2 rounded border border-yellow-400/30 bg-yellow-400/10 px-4 py-2 text-xs font-black text-yellow-200">
              Pump launch page <ExternalLink className="h-3.5 w-3.5" />
            </a>
          ) : (
            <span className="rounded border border-zinc-800 bg-zinc-950 px-4 py-2 text-xs font-bold text-zinc-500">
              Pump链接：发射后配置
            </span>
          )}
        </div>
      </section>

      <section className="grid grid-cols-1 gap-4 lg:grid-cols-3">
        <StatusCard
          icon={Rocket}
          title="ALPHA Mainnet Mint"
          value={config.alphaMint?.toBase58() ?? '尚未发射'}
          note={config.alphaMint ? '已配置，只读展示' : '不能把Devnet Program ID当作代币Mint'}
          href={config.alphaMint ? getMainnetExplorerAddressUrl(config.alphaMint) : null}
        />
        <StatusCard
          icon={Wallet}
          title="公开收入钱包"
          value={snapshot?.revenueWallet ?? config.revenueWallet?.toBase58() ?? '尚未配置'}
          note={config.revenueWallet ? 'Mainnet只读，不具备前端付款权限' : '发射前必须公布并验证'}
          href={config.revenueWallet ? getMainnetExplorerAddressUrl(config.revenueWallet) : null}
        />
        <StatusCard
          icon={Landmark}
          title="待分配USDC"
          value={distributable === null ? '不可用' : `${distributable.toFixed(6)} USDC`}
          note={`累计达到 ${config.distributionThresholdUsdc} USDC 后执行分配批次`}
        />
      </section>

      <section className="rounded-xl border border-zinc-800 bg-zinc-950/70 p-5">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h3 className="font-black text-zinc-100">Mainnet收入读取状态</h3>
            <p className="mt-1 text-[11px] text-zinc-500">
              {status === 'unconfigured' && '收入钱包未配置，因此不展示伪造的零余额。'}
              {status === 'loading' && '正在读取Mainnet公开账户……'}
              {status === 'ready' && `读取成功${lastLoadedAt ? ` · ${lastLoadedAt.toLocaleString()}` : ''}`}
              {status === 'error' && `读取失败：${error ?? 'unknown error'}`}
            </p>
          </div>
          <button type="button" onClick={() => void refresh()} disabled={!config.revenueWallet || status === 'loading'} className="inline-flex items-center gap-2 rounded border border-zinc-700 px-3 py-2 text-xs font-bold text-zinc-300 disabled:cursor-not-allowed disabled:opacity-40">
            {status === 'loading' ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <RefreshCw className="h-3.5 w-3.5" />}
            刷新
          </button>
        </div>
        {snapshot && (
          <div className="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2">
            <Metric label="收入钱包SOL" value={`${snapshot.solBalance.toFixed(6)} SOL`} />
            <Metric label="收入钱包USDC" value={`${snapshot.usdcBalance.toFixed(6)} USDC`} />
          </div>
        )}
      </section>

      <section className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-4">
        {POOLS.map((pool) => {
          const projected = distributable === null ? null : distributable * pool.ratio / 100;
          return (
            <div key={pool.key} className="rounded-xl border border-zinc-800 bg-zinc-950/70 p-4">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-black text-zinc-100">{pool.label}</h3>
                <span className="text-lg font-black text-green-400">{pool.ratio}%</span>
              </div>
              <p className="mt-4 font-mono text-xl font-black text-zinc-200">
                {projected === null ? '—' : `${projected.toFixed(6)} USDC`}
              </p>
              <p className="mt-2 text-[11px] leading-relaxed text-zinc-500">{pool.gate}</p>
            </div>
          );
        })}
      </section>

      <section className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <Boundary
          icon={CheckCircle2}
          title="首发即可运行"
          tone="green"
          items={['公开收入地址和余额', '收入批次与5/2/2/1计算', '社区任务、举报、证据和讨论', '每笔真实支出的链上交易凭证']}
        />
        <Boundary
          icon={AlertTriangle}
          title="资金到位后才启用"
          tone="yellow"
          items={['真实赔付', '贡献者工资', '回购与销毁', 'USDC质押奖励']}
        />
      </section>

      <section className="rounded-xl border border-cyan-400/20 bg-cyan-400/5 p-5 text-xs leading-relaxed text-cyan-100/80">
        <div className="flex items-start gap-3">
          <ShieldCheck className="mt-0.5 h-5 w-5 flex-shrink-0 text-cyan-300" />
          <p>
            当前完整Alpha程序最新版本没有升级到Devnet，也没有部署Mainnet。本页面是独立的Mainnet只读启动层；
            现有Devnet模块继续作为技术证明，不能据此宣称Mainnet协议已经上线。
          </p>
        </div>
      </section>
    </section>
  );
}

function Badge({ text, tone }: { text: string; tone: 'yellow' | 'green' | 'cyan' | 'zinc' }) {
  const classes = {
    yellow: 'border-yellow-400/25 bg-yellow-400/5 text-yellow-300',
    green: 'border-green-400/25 bg-green-400/5 text-green-300',
    cyan: 'border-cyan-400/25 bg-cyan-400/5 text-cyan-300',
    zinc: 'border-zinc-700 bg-zinc-900 text-zinc-400',
  }[tone];
  return <span className={`rounded border px-2 py-1 text-[10px] font-black uppercase tracking-widest ${classes}`}>{text}</span>;
}

function StatusCard({ icon: Icon, title, value, note, href }: {
  icon: typeof Rocket;
  title: string;
  value: string;
  note: string;
  href?: string | null;
}) {
  return (
    <div className="rounded-xl border border-zinc-800 bg-zinc-950/70 p-4">
      <div className="flex items-center gap-2 text-zinc-500"><Icon className="h-4 w-4" /><span className="text-[10px] font-black uppercase tracking-widest">{title}</span></div>
      <p className="mt-3 break-all font-mono text-sm font-black text-zinc-100">{value}</p>
      <div className="mt-2 flex items-center justify-between gap-2 text-[10px] text-zinc-500">
        <span>{note}</span>
        {href && <a href={href} target="_blank" rel="noreferrer" aria-label={`${title} explorer`}><ExternalLink className="h-3.5 w-3.5" /></a>}
      </div>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return <div className="rounded border border-zinc-800 bg-zinc-950 px-3 py-2"><p className="text-[10px] text-zinc-600">{label}</p><p className="mt-1 font-mono text-sm font-black text-zinc-200">{value}</p></div>;
}

function Boundary({ icon: Icon, title, items, tone }: {
  icon: typeof CheckCircle2;
  title: string;
  items: string[];
  tone: 'green' | 'yellow';
}) {
  const color = tone === 'green' ? 'border-green-400/20 bg-green-400/5 text-green-300' : 'border-yellow-400/20 bg-yellow-400/5 text-yellow-300';
  return (
    <div className={`rounded-xl border p-5 ${color}`}>
      <div className="flex items-center gap-2"><Icon className="h-4 w-4" /><h3 className="font-black">{title}</h3></div>
      <div className="mt-3 space-y-2">{items.map((item) => <p key={item} className="flex items-center gap-2 text-xs text-zinc-300"><ArrowRight className="h-3 w-3" />{item}</p>)}</div>
    </div>
  );
}
