import { useEffect, useMemo, useState, type ReactNode } from 'react';
import { type Wallet as AnchorWallet } from '@coral-xyz/anchor';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import {
  AlertTriangle,
  Calculator,
  Coins,
  Gavel,
  Lock,
  ShieldCheck,
  TrendingUp,
  Wallet,
  Zap,
} from 'lucide-react';
import { type Lang } from '../translations';
import { createAlphaProgram, getTreasuryStatePda } from '../lib/alphaProgram';

interface Props {
  lang: Lang;
}

const TREASURY_STATE_PDA = getTreasuryStatePda();
const DAILY_POOL_TOTAL = 25_000;
const DAO_RELIEF_POOL = 150_000;
const DAO_OTHER_VICTIM_SCORE = 500_000;

const STAKING_TIERS = [
  { days: '30 天', weight: '1.0x 权重' },
  { days: '90 天', weight: '1.5x 权重' },
  { days: '180 天', weight: '2.0x 权重' },
  { days: '365 天', weight: '3.0x 权重' },
];

const STAKING_FEATURES = [
  'Stake ALPHA',
  'Unstake',
  'Claim Rewards',
  '查看个人质押份额',
  '查看当前质押等级',
];

interface AnchorTreasuryState {
  stakingPool: unknown;
}

function formatU64(value: unknown): string {
  const raw = value && typeof value === 'object' && 'toString' in value
    ? value.toString()
    : String(value ?? 0);

  try {
    return BigInt(raw).toString();
  } catch {
    return raw;
  }
}

function parsePositive(raw: string) {
  const n = parseFloat(raw);
  return Number.isFinite(n) && n > 0 ? n : 0;
}

function formatCompact(value: number, maxFrac = 2) {
  if (!Number.isFinite(value)) return '0';
  return new Intl.NumberFormat('en-US', {
    notation: 'compact',
    maximumFractionDigits: maxFrac,
  }).format(value);
}

function formatFull(value: number, maxFrac = 2) {
  if (!Number.isFinite(value)) return '0';
  return value.toLocaleString('en-US', { maximumFractionDigits: maxFrac });
}

function useBump(value: number) {
  const [bump, setBump] = useState(false);

  useEffect(() => {
    setBump(true);
    const id = setTimeout(() => setBump(false), 160);
    return () => clearTimeout(id);
  }, [value]);

  return bump;
}

function LiveMetric({
  label,
  value,
  suffix,
  colorClass,
  bump,
  sub,
}: {
  label: string;
  value: string;
  suffix?: string;
  colorClass: string;
  bump: boolean;
  sub?: string;
}) {
  return (
    <div className="bg-zinc-950/80 border border-zinc-700/50 rounded-lg p-4 text-center relative overflow-hidden min-w-0">
      <p className="text-zinc-500 text-xs font-mono uppercase tracking-wider mb-1 truncate">{label}</p>
      <p
        className={`text-xl sm:text-2xl md:text-3xl font-bold font-mono tabular-nums break-all leading-tight transition-transform duration-150 ${colorClass} ${
          bump ? 'scale-[1.03]' : 'scale-100'
        }`}
      >
        {value}
        {suffix && <span className="text-sm sm:text-base text-zinc-600 ml-1">{suffix}</span>}
      </p>
      {sub && <p className="text-zinc-600 font-mono text-[10px] mt-1 leading-snug">{sub}</p>}
    </div>
  );
}

function PoolShell({
  accent,
  icon,
  title,
  badge,
  children,
}: {
  accent: 'green' | 'cyan' | 'yellow';
  icon: ReactNode;
  title: string;
  badge: string;
  children: ReactNode;
}) {
  const border = {
    green: 'border-green-400/25',
    cyan: 'border-cyan-400/25',
    yellow: 'border-yellow-400/25',
  }[accent];
  const bg = {
    green: 'bg-green-400/5',
    cyan: 'bg-cyan-400/5',
    yellow: 'bg-yellow-400/5',
  }[accent];
  const text = {
    green: 'text-green-400',
    cyan: 'text-cyan-400',
    yellow: 'text-yellow-400',
  }[accent];
  const bar = {
    green: 'from-green-500 via-emerald-400 to-green-500',
    cyan: 'from-cyan-500 via-blue-400 to-cyan-500',
    yellow: 'from-yellow-500 via-amber-300 to-yellow-500',
  }[accent];

  return (
    <div className={`relative border ${border} ${bg} rounded-xl overflow-hidden backdrop-blur-sm`}>
      <div className={`absolute inset-x-0 top-0 h-0.5 bg-gradient-to-r ${bar} opacity-80 pointer-events-none z-10`} />
      <div className="px-5 py-4 border-b border-zinc-800/80 bg-zinc-950/60 flex flex-wrap items-center gap-3">
        <div className="flex items-center gap-2 min-w-0">
          {icon}
          <h3 className={`${text} font-mono font-bold uppercase tracking-wider text-sm sm:text-base truncate`}>
            {title}
          </h3>
        </div>
        <span
          className={`ml-auto text-[10px] font-mono font-bold uppercase tracking-widest px-2.5 py-1 rounded border ${border} ${text}`}
        >
          {badge}
        </span>
      </div>
      <div className="p-5 sm:p-6 space-y-5">{children}</div>
    </div>
  );
}

export default function VictimRelief({ lang }: Props) {
  const locale = lang === 'zh' ? 'zh-CN' : 'en-US';
  const { connection } = useConnection();
  const wallet = useWallet();
  const { publicKey, connected, signAllTransactions, signTransaction } = wallet;
  const [stakeAmount, setStakeAmount] = useState('5000');
  const [lockDays, setLockDays] = useState(90);
  const [lossAmount, setLossAmount] = useState(12_500);
  const [victimStakeAmount, setVictimStakeAmount] = useState('8000');
  const [chainStakingPool, setChainStakingPool] = useState<string | null>(null);
  const [stakingPoolStatus, setStakingPoolStatus] = useState<'idle' | 'loading' | 'ready' | 'missing' | 'error'>('idle');
  const [stakingPoolError, setStakingPoolError] = useState<string | null>(null);

  const shortAddress = publicKey
    ? `${publicKey.toBase58().slice(0, 4)}...${publicKey.toBase58().slice(-4)}`
    : null;

  useEffect(() => {
    let mounted = true;

    if (!connected || !publicKey) {
      setChainStakingPool(null);
      setStakingPoolStatus('idle');
      setStakingPoolError(null);
      return () => {
        mounted = false;
      };
    }

    if (!signTransaction || !signAllTransactions) {
      setChainStakingPool(null);
      setStakingPoolStatus('error');
      setStakingPoolError('当前钱包不支持交易签名');
      return () => {
        mounted = false;
      };
    }

    const fetchStakingPool = async () => {
      setStakingPoolStatus('loading');
      setStakingPoolError(null);

      try {
        const program = createAlphaProgram(connection, {
          publicKey,
          signAllTransactions,
          signTransaction,
        } as unknown as AnchorWallet);
        const accountClient = (program.account as unknown as {
          treasuryState: {
            fetchNullable(address: typeof TREASURY_STATE_PDA): Promise<AnchorTreasuryState | null>;
          };
        }).treasuryState;
        const treasuryState = await accountClient.fetchNullable(TREASURY_STATE_PDA);

        if (!mounted) return;

        if (!treasuryState) {
          setChainStakingPool(null);
          setStakingPoolStatus('missing');
          return;
        }

        setChainStakingPool(formatU64(treasuryState.stakingPool));
        setStakingPoolStatus('ready');
      } catch (err) {
        if (!mounted) return;
        setChainStakingPool(null);
        setStakingPoolStatus('error');
        setStakingPoolError(err instanceof Error ? err.message : String(err));
      }
    };

    fetchStakingPool();

    return () => {
      mounted = false;
    };
  }, [connected, connection, publicKey, signAllTransactions, signTransaction]);

  const dailyStake = useMemo(() => parsePositive(stakeAmount), [stakeAmount]);
  const dailyPoints = useMemo(() => {
    if (dailyStake <= 0 || lockDays <= 0) return 0;
    return dailyStake * lockDays;
  }, [dailyStake, lockDays]);
  const dailyTotalPoints = 20_000_000 + dailyPoints;
  const dailyRewards = useMemo(() => {
    if (dailyPoints <= 0 || dailyTotalPoints <= 0) return 0;
    const ratio = Math.min(1, dailyPoints / dailyTotalPoints);
    return DAILY_POOL_TOTAL * ratio;
  }, [dailyPoints, dailyTotalPoints]);

  const victimStake = useMemo(() => parsePositive(victimStakeAmount), [victimStakeAmount]);
  const victimScore = useMemo(() => {
    if (lossAmount <= 0 || victimStake <= 0) return 0;
    return Math.sqrt(victimStake) * (1 + Math.log10(lossAmount));
  }, [victimStake, lossAmount]);
  const poolSharePct = useMemo(() => {
    if (victimScore <= 0) return 0;
    const denom = DAO_OTHER_VICTIM_SCORE + victimScore;
    return (victimScore / denom) * 100;
  }, [victimScore]);
  const daoCompensation = useMemo(() => {
    const ratio = Math.min(1, Math.max(0, poolSharePct / 100));
    return DAO_RELIEF_POOL * ratio;
  }, [poolSharePct]);

  const dailyPointsBump = useBump(dailyPoints);
  const dailyRewardsBump = useBump(dailyRewards);
  const victimScoreBump = useBump(victimScore);
  const poolShareBump = useBump(poolSharePct);
  const daoRewardsBump = useBump(daoCompensation);

  const stakingPoolHelp = stakingPoolStatus === 'loading'
    ? '读取中...'
    : stakingPoolStatus === 'ready'
      ? `Devnet TreasuryState · ${new Date().toLocaleTimeString(locale)}`
      : stakingPoolStatus === 'missing'
        ? 'TreasuryState 尚未初始化'
        : stakingPoolStatus === 'error'
          ? stakingPoolError ?? '读取失败'
          : '连接钱包后读取';

  return (
    <div className="space-y-8">
      <div className="flex items-center gap-3">
        <Zap className="text-green-400 w-6 h-6" />
        <h2 className="text-2xl font-bold text-green-400 tracking-widest uppercase font-mono">
          Devnet Alpha 估算演示
        </h2>
      </div>

      <div className="border border-green-400/30 bg-green-400/5 rounded-xl p-4 relative overflow-hidden">
        <div className="absolute top-0 left-0 w-1 h-full bg-green-400/60 rounded-l-xl" />
        <div className="flex items-start gap-3 pl-2">
          <ShieldCheck className="w-5 h-5 text-green-400 flex-shrink-0 mt-0.5" />
          <div>
            <p className="text-green-400 font-mono text-xs font-bold uppercase tracking-widest mb-1">
              资产质押与分红 · 测试网原型
            </p>
            <p className="text-green-200/80 font-mono text-xs leading-relaxed">
              本标签页保留质押奖励与赔付估算器的演示体验，用于展示未来规则方向。当前不提交真实 stake、unstake、claim 或赔付交易。
            </p>
          </div>
        </div>
      </div>

      <div
        className={`flex items-center gap-2 px-4 py-2.5 rounded-lg border font-mono text-xs transition-colors duration-200 ${
          connected
            ? 'border-green-400/30 bg-green-400/5 text-green-400'
            : 'border-zinc-800 bg-zinc-950/80 text-zinc-500'
        }`}
      >
        <Wallet className={`w-3.5 h-3.5 flex-shrink-0 ${connected ? 'text-green-400' : 'text-zinc-600'}`} />
        {connected && shortAddress ? (
          <span className="tabular-nums">
            当前钱包：<span className="font-bold">{shortAddress}</span>
          </span>
        ) : (
          <span>钱包未连接 · 交互已锁定</span>
        )}
      </div>

      <PoolShell
        accent="yellow"
        icon={<Coins className="w-5 h-5 text-yellow-400" />}
        title="Staking / 质押分红"
        badge="协议收入 10%"
      >
        <div className="grid grid-cols-1 lg:grid-cols-[260px_minmax(0,1fr)] gap-4">
          <div className="rounded-lg border border-zinc-800 bg-zinc-950/70 p-4">
            <p className="text-zinc-500 font-mono text-[10px] uppercase tracking-widest mb-2">
              stakingPool 当前余额
            </p>
            <p className="text-yellow-400 font-mono text-2xl font-black tabular-nums break-all">
              {chainStakingPool ?? '--'}
            </p>
            <p className="text-zinc-600 font-mono text-[10px] mt-1">{stakingPoolHelp}</p>
          </div>

          <div className="rounded-lg border border-zinc-800 bg-zinc-950/70 p-4 text-xs font-mono leading-relaxed text-zinc-300">
            <p>质押分红池来自协议收入的 10%。</p>
            <p className="mt-2">
              持币者未来质押 ALPHA 后，将根据质押时间长短和权重等级自动获得分红。
            </p>
            <p className="mt-2 text-yellow-300 font-bold">
              质押分红不需要 DAO 逐笔投票。分红由质押规则和链上合约自动计算。DAO 最多参与未来质押规则或协议参数调整。
            </p>
          </div>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
          {STAKING_TIERS.map((tier) => (
            <div key={tier.days} className="rounded-lg border border-zinc-800 bg-zinc-950/70 p-4">
              <p className="text-yellow-400 font-mono text-lg font-black">{tier.days}</p>
              <p className="text-zinc-300 font-mono text-xs font-bold mt-1">{tier.weight}</p>
            </div>
          ))}
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
          {STAKING_FEATURES.map((feature) => (
            <div key={feature} className="rounded-lg border border-zinc-800 bg-zinc-950/70 p-3">
              <p className="text-zinc-300 font-mono text-xs font-bold mb-2">{feature}</p>
              <button
                type="button"
                disabled
                className="w-full rounded border border-zinc-700 bg-zinc-900/70 px-3 py-2 text-[11px] font-mono font-bold text-zinc-500 disabled:cursor-not-allowed"
              >
                质押分红合约开发中
              </button>
            </div>
          ))}
        </div>
      </PoolShell>

      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        <PoolShell
          accent="green"
          icon={<Lock className="w-5 h-5 text-green-400" />}
          title="质押权重估算"
          badge="Devnet Alpha 演示"
        >
          <p className="text-zinc-400 font-mono text-xs leading-relaxed">
            该估算器只演示未来可能的时间权重逻辑，不代表当前已经开放的链上质押交易。
          </p>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="space-y-2 min-w-0">
              <div className="flex items-center justify-between gap-2">
                <label className="text-zinc-500 text-xs font-mono uppercase tracking-wider">
                  ALPHA 数量
                </label>
                <span className="text-[11px] font-mono text-green-300/80 tabular-nums truncate">
                  {stakeAmount || '0'} ALPHA
                </span>
              </div>
              <input
                type="number"
                min={0}
                step="any"
                value={stakeAmount}
                onChange={(e) => setStakeAmount(e.target.value)}
                className="w-full bg-zinc-950 border border-green-400/25 rounded-lg px-3 py-2.5 text-sm font-mono text-zinc-200 focus:outline-none focus:border-green-400/60 transition-colors"
              />
            </div>

            <div className="space-y-2 min-w-0">
              <div className="flex items-center justify-between gap-2">
                <label className="text-zinc-500 text-xs font-mono uppercase tracking-wider">
                  质押天数
                </label>
                <span className="text-[11px] font-mono text-yellow-300/80 tabular-nums">
                  {lockDays} 天
                </span>
              </div>
              <input
                type="range"
                min={1}
                max={365}
                step={1}
                value={lockDays}
                onChange={(e) => setLockDays(parseInt(e.target.value, 10))}
                className="w-full accent-green-400"
              />
              <input
                type="number"
                min={1}
                max={365}
                value={lockDays}
                onChange={(e) => {
                  const v = Number(e.target.value);
                  if (!Number.isFinite(v)) return;
                  setLockDays(Math.min(365, Math.max(1, Math.trunc(v))));
                }}
                className="w-full bg-zinc-950 border border-yellow-400/25 rounded-lg px-3 py-2.5 text-sm font-mono text-zinc-200 focus:outline-none focus:border-yellow-400/60 transition-colors"
              />
            </div>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <LiveMetric
              label="Weight Points"
              value={formatCompact(dailyPoints)}
              colorClass="text-cyan-400"
              bump={dailyPointsBump}
              sub={`Total: ${formatCompact(dailyTotalPoints)}`}
            />
            <LiveMetric
              label="演示分配份额"
              value={formatCompact(dailyRewards)}
              suffix="units"
              colorClass="text-green-400"
              bump={dailyRewardsBump}
              sub={`${formatCompact(DAILY_POOL_TOTAL)} × points ratio`}
            />
          </div>

          <button
            type="button"
            disabled
            className="w-full py-3 font-mono font-bold uppercase tracking-wider rounded-lg text-sm bg-zinc-900/50 border border-zinc-700/50 text-zinc-500 disabled:cursor-not-allowed"
          >
            质押分红合约开发中
          </button>
        </PoolShell>

        <PoolShell
          accent="cyan"
          icon={<Gavel className="w-5 h-5 text-cyan-400" />}
          title="赔付救济估算"
          badge="DAO 审核路线图"
        >
          <p className="text-zinc-400 font-mono text-xs leading-relaxed">
            赔付救济池来自协议收入的 50%。赔付申请未来由 DAO 治理审核，通过后再执行赔付。
          </p>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="space-y-2 min-w-0">
              <div className="flex items-center justify-between gap-2">
                <label className="text-zinc-500 text-xs font-mono uppercase tracking-wider">
                  损失金额
                </label>
                <span className="text-[11px] font-mono text-purple-300/80 tabular-nums">
                  {lossAmount.toLocaleString()} USDC
                </span>
              </div>
              <input
                type="range"
                min={0}
                max={100_000}
                step={500}
                value={lossAmount}
                onChange={(e) => setLossAmount(parseInt(e.target.value, 10))}
                className="w-full accent-purple-400"
              />
              <input
                type="number"
                min={0}
                max={100_000}
                value={lossAmount}
                onChange={(e) => {
                  const v = Number(e.target.value);
                  if (!Number.isFinite(v)) return;
                  setLossAmount(Math.min(100_000, Math.max(0, Math.trunc(v))));
                }}
                className="w-full bg-zinc-950 border border-purple-400/25 rounded-lg px-3 py-2.5 text-sm font-mono text-zinc-200 focus:outline-none focus:border-purple-400/60 transition-colors"
              />
            </div>

            <div className="space-y-2 min-w-0">
              <div className="flex items-center justify-between gap-2">
                <label className="text-zinc-500 text-xs font-mono uppercase tracking-wider">
                  事件质押数量
                </label>
                <span className="text-[11px] font-mono text-cyan-300/80 tabular-nums truncate">
                  {victimStakeAmount || '0'} ALPHA
                </span>
              </div>
              <input
                type="number"
                min={0}
                step="any"
                value={victimStakeAmount}
                onChange={(e) => setVictimStakeAmount(e.target.value)}
                className="w-full bg-zinc-950 border border-cyan-400/25 rounded-lg px-3 py-2.5 text-sm font-mono text-zinc-200 focus:outline-none focus:border-cyan-400/60 transition-colors"
              />
            </div>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
            <LiveMetric
              label="VictimScore"
              value={formatCompact(victimScore)}
              colorClass="text-cyan-400"
              bump={victimScoreBump}
            />
            <LiveMetric
              label="Pool Share %"
              value={poolSharePct.toFixed(4)}
              suffix="%"
              colorClass="text-yellow-400"
              bump={poolShareBump}
            />
            <LiveMetric
              label="演示赔付额"
              value={formatCompact(daoCompensation)}
              suffix="USDC"
              colorClass="text-green-400"
              bump={daoRewardsBump}
              sub={`${formatFull(DAO_RELIEF_POOL)} × share`}
            />
          </div>

          <button
            type="button"
            disabled
            className="w-full py-3 font-mono font-bold uppercase tracking-wider rounded-lg text-sm flex items-center justify-center gap-2 bg-zinc-900/50 border border-zinc-700/50 text-zinc-500 disabled:cursor-not-allowed"
          >
            <ShieldCheck className="w-4 h-4" />
            将通过 DAO 提案提交
          </button>
        </PoolShell>
      </div>

      <div className="border border-red-400/25 bg-red-400/5 rounded-lg px-4 py-3 flex items-start gap-2">
        <AlertTriangle className="w-4 h-4 text-red-400 flex-shrink-0 mt-0.5" />
        <p className="text-red-200 font-mono text-xs leading-relaxed">
          估算结果仅用于测试网演示，不代表真实赔付承诺。
        </p>
      </div>

      <div className="border border-yellow-400/30 bg-yellow-400/5 rounded-lg px-4 py-3 flex items-start gap-2">
        <Calculator className="w-4 h-4 text-yellow-400 flex-shrink-0 mt-0.5" />
        <p className="text-yellow-300 font-mono text-xs leading-relaxed">
          当前版本不涉及真实主网资金、真实质押收益或正式赔付执行。后续功能将通过合约升级、测试、安全审查和社区治理逐步开放。
        </p>
      </div>

      <div className="border border-green-400/20 bg-green-400/5 rounded-lg px-4 py-3 flex items-start gap-2">
        <TrendingUp className="w-4 h-4 text-green-400 flex-shrink-0 mt-0.5" />
        <p className="text-green-200/80 font-mono text-xs leading-relaxed">
          ALPHA 质押分红是自动机制，不需要 DAO 逐笔投票；DAO 最多负责未来调整质押规则或协议参数。
        </p>
      </div>
    </div>
  );
}
