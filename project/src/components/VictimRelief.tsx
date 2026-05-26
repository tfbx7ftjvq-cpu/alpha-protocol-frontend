import { useEffect, useMemo, useState, type ReactNode } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import {
  AlertTriangle,
  Calculator,
  Zap,
  TrendingUp,
  TrendingDown,
  ShieldCheck,
  Shield,
  Activity,
  Lock,
  Gavel,
  Wallet,
} from 'lucide-react';
import { t, Lang } from '../translations';

interface Props {
  lang: Lang;
}

// ─── Track 1: Daily Consensus Pool (10% treasury) ───
const DAILY_POOL_TOTAL = 25_000;
const DAILY_NETWORK_BASE = 20_000_000;

// ─── Track 2: DAO Relief Hall (50% treasury) ───
const DAO_RELIEF_POOL = 150_000;
const DAO_OTHER_VICTIM_SCORE = 500_000;

function useBump(value: number) {
  const [bump, setBump] = useState(false);
  useEffect(() => {
    setBump(true);
    const id = setTimeout(() => setBump(false), 160);
    return () => clearTimeout(id);
  }, [value]);
  return bump;
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

function parsePositive(raw: string) {
  const n = parseFloat(raw);
  return Number.isFinite(n) && n > 0 ? n : 0;
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
  accent: 'green' | 'cyan';
  icon: ReactNode;
  title: string;
  badge: string;
  children: ReactNode;
}) {
  const border = accent === 'green' ? 'border-green-400/25' : 'border-cyan-400/25';
  const bg = accent === 'green' ? 'bg-green-400/5' : 'bg-cyan-400/5';
  const text = accent === 'green' ? 'text-green-400' : 'text-cyan-400';
  const bar =
    accent === 'green'
      ? 'from-green-500 via-emerald-400 to-green-500'
      : 'from-cyan-500 via-blue-400 to-cyan-500';

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
  const tr = t[lang];
  const zh = lang === 'zh';
  const { publicKey, connected } = useWallet();
  const [isPriceDropped, setIsPriceDropped] = useState(true);

  const shortAddress = publicKey
    ? `${publicKey.toBase58().slice(0, 4)}...${publicKey.toBase58().slice(-4)}`
    : null;

  const connectHint = zh ? '请在右上角连接钱包以激活交互' : 'Connect wallet (top-right) to enable';
  const dailyActionLabel = zh ? '授信并授权质押' : 'Authorize & Stake';
  const daoActionLabel = zh ? '向 DAO 提交救济申请' : 'Submit DAO Relief Application';

  // ─── Track 1 state ───
  const [stakeAmount, setStakeAmount] = useState('5000');
  const [lockDays, setLockDays] = useState(90);

  const dailyStake = useMemo(() => parsePositive(stakeAmount), [stakeAmount]);
  const dailyPoints = useMemo(() => {
    if (dailyStake <= 0 || lockDays <= 0) return 0;
    return dailyStake * lockDays;
  }, [dailyStake, lockDays]);
  const dailyTotalPoints = DAILY_NETWORK_BASE + dailyPoints;
  const dailyRewards = useMemo(() => {
    if (dailyPoints <= 0 || dailyTotalPoints <= 0) return 0;
    const ratio = Math.min(1, dailyPoints / dailyTotalPoints);
    return DAILY_POOL_TOTAL * ratio;
  }, [dailyPoints, dailyTotalPoints]);

  const dailyPointsBump = useBump(dailyPoints);
  const dailyRewardsBump = useBump(dailyRewards);

  // ─── Track 2 state ───
  const [lossAmount, setLossAmount] = useState(12_500);
  const [victimStakeAmount, setVictimStakeAmount] = useState('8000');

  const isVictimEligible = lossAmount > 0;
  const victimStake = useMemo(() => parsePositive(victimStakeAmount), [victimStakeAmount]);
  const victimScore = useMemo(() => {
    if (!isVictimEligible || victimStake <= 0) return 0;
    return Math.sqrt(victimStake) * (1 + Math.log10(lossAmount));
  }, [isVictimEligible, victimStake, lossAmount]);
  const poolSharePct = useMemo(() => {
    if (victimScore <= 0) return 0;
    const denom = DAO_OTHER_VICTIM_SCORE + victimScore;
    return (victimScore / denom) * 100;
  }, [victimScore]);
  const daoCompensation = useMemo(() => {
    const ratio = Math.min(1, Math.max(0, poolSharePct / 100));
    return DAO_RELIEF_POOL * ratio;
  }, [poolSharePct]);

  const victimScoreBump = useBump(victimScore);
  const poolShareBump = useBump(poolSharePct);
  const daoRewardsBump = useBump(daoCompensation);

  return (
    <div className="space-y-8">
      {/* Section header */}
      <div className="flex items-center gap-3">
        <Zap className="text-green-400 w-6 h-6" />
        <h2 className="text-2xl font-bold text-green-400 tracking-widest uppercase font-mono">
          {tr.reliefTitle}
        </h2>
      </div>

      {/* Dual-track overview */}
      <div className="border border-green-400/30 bg-green-400/5 rounded-xl p-4 relative overflow-hidden">
        <div className="absolute top-0 left-0 w-1 h-full bg-green-400/60 rounded-l-xl" />
        <div className="flex items-start gap-3 pl-2">
          <ShieldCheck className="w-5 h-5 text-green-400 flex-shrink-0 mt-0.5" />
          <div>
            <p className="text-green-400 font-mono text-xs font-bold uppercase tracking-widest mb-1">
              {zh ? '双轨制质押池 · DeFi 反套利风控' : 'Dual-Track Pools · DeFi Anti-Arbitrage'}
            </p>
            <p className="text-green-200/80 font-mono text-xs leading-relaxed">
              {zh
                ? '日常共识池（国库 10%）与 DAO 裁决救济大厅（国库 50%）业务隔离、风控独立。闪电贷防护、熔断惩罚、Merkle 快照核验与 √Stake 大户钝化算法均已启用。'
                : 'Daily Consensus (10% treasury) and DAO Relief Hall (50% treasury) are isolated with independent risk controls: flash-loan guards, circuit-breaker penalties, Merkle snapshot verification, and √Stake whale dampening.'}
            </p>
          </div>
        </div>
      </div>

      {/* Wallet identity */}
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
            {zh ? '授信账户' : 'Using Account'}: <span className="font-bold">{shortAddress}</span>
          </span>
        ) : (
          <span>{zh ? '未连接钱包 · 交互已锁定' : 'Wallet disconnected · interactions locked'}</span>
        )}
      </div>

      {/* Price Protection */}
      <div className="border border-zinc-700/60 bg-zinc-950/50 rounded-xl overflow-hidden backdrop-blur-sm">
        <div className="flex items-center justify-between px-5 py-3 border-b border-zinc-800/50 bg-zinc-950/70">
          <div className="flex items-center gap-2">
            <Activity className="w-4 h-4 text-zinc-400" />
            <span className="text-zinc-400 font-mono text-xs font-bold uppercase tracking-widest">
              {tr.marketStatus}
            </span>
          </div>
          <button
            type="button"
            onClick={() => setIsPriceDropped((v) => !v)}
            className={`flex items-center gap-2 text-xs font-mono font-bold px-3 py-1.5 rounded-lg border transition-all duration-200 ${
              isPriceDropped
                ? 'border-red-400/50 text-red-400 bg-red-400/10 hover:bg-red-400/15'
                : 'border-green-400/40 text-green-400 bg-green-400/8 hover:bg-green-400/15'
            }`}
          >
            {isPriceDropped ? (
              <>
                <TrendingDown className="w-3 h-3" />
                {tr.priceDropToggleOff}
              </>
            ) : (
              <>
                <TrendingUp className="w-3 h-3" />
                {tr.priceDropToggleOn}
              </>
            )}
          </button>
        </div>
        <div className="p-5">
          {isPriceDropped ? (
            <div
              className="relative border border-red-400/50 rounded-xl p-4 overflow-hidden"
              style={{
                background:
                  'linear-gradient(135deg, rgba(127,29,29,0.25) 0%, rgba(120,53,15,0.20) 50%, rgba(127,29,29,0.15) 100%)',
              }}
            >
              <div className="flex items-start gap-3">
                <TrendingDown className="w-4 h-4 text-red-400 animate-pulse flex-shrink-0 mt-1" />
                <p className="text-red-200 font-mono text-sm leading-relaxed font-bold">{tr.priceDropAlert}</p>
              </div>
            </div>
          ) : (
            <div className="flex items-center gap-3 py-2">
              <ShieldCheck className="w-5 h-5 text-green-400" />
              <p className="text-green-400 font-mono font-bold text-sm">{tr.priceNormal}</p>
            </div>
          )}
        </div>
      </div>

      {/* Staking alert */}
      <div className="border border-red-400/50 bg-red-400/10 rounded-xl p-5 relative overflow-hidden">
        <div className="absolute top-0 left-0 w-1 h-full bg-red-500" />
        <div className="flex items-start gap-3 pl-4">
          <AlertTriangle className="text-red-400 w-5 h-5 mt-0.5 flex-shrink-0 animate-pulse" />
          <p className="text-red-300 font-mono text-sm leading-relaxed font-bold">{tr.stakingAlert}</p>
        </div>
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        {/* ═══════════════════════════════════════════════════════
            TRACK 1 — Daily Consensus Staking Pool (10%)
        ═══════════════════════════════════════════════════════ */}
        <PoolShell
          accent="green"
          icon={<Lock className="w-5 h-5 text-green-400" />}
          title={zh ? '日常共识质押池' : 'Daily Consensus Staking Pool'}
          badge={zh ? '国库 10% · 25,000 USDC' : 'Treasury 10% · 25,000 USDC'}
        >
          <p className="text-zinc-400 font-mono text-xs leading-relaxed">
            {zh
              ? '奖励长期持有并锁定 α 的共识参与者。锁定天数系数与退仓熔断机制可有效抵御分红前闪电存入、分红后即刻抽离的吸血鬼攻击。'
              : 'Rewards long-term α stakers. Lock-day multiplier + exit circuit breaker defend against flash-deposit vampire attacks around dividend snapshots.'}
          </p>

          <div className="bg-zinc-950/70 border border-zinc-800/60 rounded-lg px-4 py-3 space-y-1 font-mono text-xs">
            <p className="text-zinc-500 uppercase tracking-widest text-[10px]">{zh ? '透明公式' : 'Formula'}</p>
            <p>
              <span className="text-cyan-400 font-bold">Points</span>
              <span className="text-zinc-600"> = </span>
              <span className="text-green-400">Stake</span>
              <span className="text-zinc-600"> × </span>
              <span className="text-yellow-400">LockDays</span>
            </p>
            <p>
              <span className="text-green-400 font-bold">{zh ? '日常收益' : 'Daily Yield'}</span>
              <span className="text-zinc-600"> = 25,000 × (Points / TotalPoints)</span>
            </p>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="space-y-2 min-w-0">
              <div className="flex items-center justify-between gap-2">
                <label className="text-zinc-500 text-xs font-mono uppercase tracking-wider">
                  {zh ? '质押 α 数量' : 'Stake Amount (α)'}
                </label>
                <span className="text-[11px] font-mono text-green-300/80 tabular-nums truncate">
                  {stakeAmount || '0'} α
                </span>
              </div>
              <input
                type="number"
                min={0}
                step="any"
                value={stakeAmount}
                onChange={(e) => setStakeAmount(e.target.value)}
                placeholder={zh ? '无上限' : 'No cap'}
                className="w-full bg-zinc-950 border border-green-400/25 rounded-lg px-3 py-2.5 text-sm font-mono text-zinc-200 focus:outline-none focus:border-green-400/60 transition-colors"
              />
            </div>

            <div className="space-y-2 min-w-0">
              <div className="flex items-center justify-between gap-2">
                <label className="text-zinc-500 text-xs font-mono uppercase tracking-wider">
                  {zh ? '锁定天数' : 'Lock Days'}
                </label>
                <span className="text-[11px] font-mono text-yellow-300/80 tabular-nums">
                  {lockDays} {zh ? '天' : 'd'}
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

          <div className="border border-red-400/40 bg-red-400/5 rounded-lg px-4 py-3">
            <p className="text-red-400 font-mono text-xs leading-relaxed font-bold">
              ⚡{' '}
              {zh
                ? '熔断机制已激活：中途任何提前解锁或部分提取行为，将导致累计的锁仓天数与算力积分瞬间强制归零，并触发 5% 提退维税注入日常池。'
                : 'Circuit breaker active: any early unlock or partial withdrawal instantly zeroes lock days & points, and triggers a 5% exit tax into the daily pool.'}
            </p>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <LiveMetric
              label="Points"
              value={formatCompact(dailyPoints)}
              colorClass="text-cyan-400"
              bump={dailyPointsBump}
              sub={`Total: ${formatCompact(dailyTotalPoints)}`}
            />
            <LiveMetric
              label={zh ? '预估日常收益' : 'Est. Daily Yield'}
              value={formatCompact(dailyRewards)}
              suffix="USDC"
              colorClass="text-green-400"
              bump={dailyRewardsBump}
              sub={zh ? '25,000 × (Points / TotalPoints)' : '25,000 × (Points / TotalPoints)'}
            />
          </div>

          <button
            type="button"
            disabled={!connected || dailyPoints <= 0}
            className={`w-full py-3 font-mono font-bold uppercase tracking-wider rounded-lg transition-all duration-200 text-sm ${
              connected
                ? 'bg-green-500/25 hover:bg-green-500/35 border border-green-500/60 text-green-400 hover:shadow-green-500/20 hover:shadow-lg'
                : 'bg-zinc-900/50 border border-zinc-700/50 text-zinc-500'
            } disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:shadow-none`}
          >
            {!connected ? connectHint : dailyActionLabel}
          </button>
        </PoolShell>

        {/* ═══════════════════════════════════════════════════════
            TRACK 2 — DAO Relief Hall (50%)
        ═══════════════════════════════════════════════════════ */}
        <PoolShell
          accent="cyan"
          icon={<Gavel className="w-5 h-5 text-cyan-400" />}
          title={zh ? 'DAO 裁决救济大厅' : 'DAO Relief Adjudication Hall'}
          badge={zh ? '国库 50% · 150,000 USDC' : 'Treasury 50% · 150,000 USDC'}
        >
          <div className="border border-cyan-400/30 bg-cyan-400/5 rounded-lg px-4 py-3 space-y-1">
            <p className="text-cyan-400 font-mono text-xs font-bold uppercase tracking-widest">
              {zh ? 'DAO 003 号救济提案' : 'DAO Proposal #003'}
            </p>
            <p className="text-zinc-300 font-mono text-sm">
              {zh ? '针对 X 项目 Rug 事件 · 定向救济分发' : 'X Protocol Rug Event · Targeted Relief'}
            </p>
            <p className="text-green-400 font-mono text-lg font-bold tabular-nums">
              {formatFull(DAO_RELIEF_POOL)} <span className="text-sm text-zinc-500">USDC</span>
            </p>
          </div>

          <div className="bg-zinc-950/70 border border-zinc-800/60 rounded-lg px-4 py-3 space-y-1 font-mono text-xs">
            <p className="text-zinc-500 uppercase tracking-widest text-[10px]">{zh ? '风控公式' : 'Risk Formula'}</p>
            <p>
              <span className="text-cyan-400 font-bold">VictimScore</span>
              <span className="text-zinc-600"> = √Stake × (1 + log10(Loss))</span>
            </p>
            <p>
              <span className="text-green-400 font-bold">Pool Share %</span>
              <span className="text-zinc-600"> = VictimScore / (500,000 + VictimScore) × 100</span>
            </p>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="space-y-2 min-w-0">
              <div className="flex items-center justify-between gap-2">
                <label className="text-zinc-500 text-xs font-mono uppercase tracking-wider">
                  {zh ? '历史真实损失（快照）' : 'Verified Historical Loss'}
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
              <p className="text-zinc-600 font-mono text-[10px]">
                {zh ? '链上 Merkle 快照自动读取 · 0 - 100,000 USDC' : 'Merkle snapshot · 0 - 100,000 USDC'}
              </p>
            </div>

            <div className="space-y-2 min-w-0">
              <div className="flex items-center justify-between gap-2">
                <label className="text-zinc-500 text-xs font-mono uppercase tracking-wider">
                  {zh ? '事件质押 α 数量' : 'Event Stake (α)'}
                </label>
                <span className="text-[11px] font-mono text-cyan-300/80 tabular-nums truncate">
                  {victimStakeAmount || '0'} α
                </span>
              </div>
              <input
                type="number"
                min={0}
                step="any"
                value={victimStakeAmount}
                onChange={(e) => setVictimStakeAmount(e.target.value)}
                disabled={!isVictimEligible}
                className="w-full bg-zinc-950 border border-cyan-400/25 rounded-lg px-3 py-2.5 text-sm font-mono text-zinc-200 focus:outline-none focus:border-cyan-400/60 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
              />
            </div>
          </div>

          {!isVictimEligible && (
            <div className="border border-zinc-600/50 bg-zinc-900/60 rounded-lg px-4 py-3">
              <p className="text-zinc-400 font-mono text-xs leading-relaxed">
                {zh
                  ? '未检测到您在事故快照中的受灾数据，无权参与此定向救济分发。'
                  : 'No victim data detected in incident snapshot. You are not eligible for this targeted relief.'}
              </p>
            </div>
          )}

          <div className="border border-green-400/30 bg-green-400/5 rounded-lg px-4 py-3 flex items-start gap-2">
            <Shield className="w-4 h-4 text-green-400 flex-shrink-0 mt-0.5" />
            <p className="text-green-400 font-mono text-xs leading-relaxed">
              🛡️{' '}
              {zh
                ? '散户正义防线已激活：当前救济完全基于事故发生前的链上 Merkle Tree 历史快照。大户盲目质押将因 Points ∝ √Stake 算法导致收益严重钝化，无法稀释散户份额。'
                : 'Retail defense active: relief is based on pre-incident Merkle Tree snapshots. Whale stakes are dampened via Points ∝ √Stake — cannot dilute retail share.'}
            </p>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
            <LiveMetric
              label="VictimScore"
              value={isVictimEligible ? formatCompact(victimScore) : '—'}
              colorClass="text-cyan-400"
              bump={victimScoreBump}
            />
            <LiveMetric
              label="Pool Share %"
              value={isVictimEligible ? poolSharePct.toFixed(4) : '0.0000'}
              suffix="%"
              colorClass="text-yellow-400"
              bump={poolShareBump}
            />
            <LiveMetric
              label={zh ? '预估损失补偿' : 'Est. Compensation'}
              value={isVictimEligible ? formatCompact(daoCompensation) : '0'}
              suffix="USDC"
              colorClass="text-green-400"
              bump={daoRewardsBump}
              sub={`${formatFull(DAO_RELIEF_POOL)} × Share`}
            />
          </div>

          <button
            type="button"
            disabled={!connected || !isVictimEligible || victimScore <= 0}
            className={`w-full py-3 font-mono font-bold uppercase tracking-wider rounded-lg transition-all duration-200 text-sm flex items-center justify-center gap-2 ${
              connected
                ? 'bg-green-500/25 hover:bg-green-500/35 border border-green-500/60 text-green-400 hover:shadow-green-500/20 hover:shadow-lg'
                : 'bg-zinc-900/50 border border-zinc-700/50 text-zinc-500'
            } disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:shadow-none`}
          >
            <Shield className="w-4 h-4" />
            {!connected ? connectHint : daoActionLabel}
          </button>
        </PoolShell>
      </div>

      {/* Tax notice */}
      <div className="border border-yellow-400/30 bg-yellow-400/5 rounded-lg px-4 py-3 flex items-start gap-2">
        <Calculator className="w-4 h-4 text-yellow-400 flex-shrink-0 mt-0.5" />
        <p className="text-yellow-300 font-mono text-xs leading-relaxed">{tr.taxWarning}</p>
      </div>
    </div>
  );
}
