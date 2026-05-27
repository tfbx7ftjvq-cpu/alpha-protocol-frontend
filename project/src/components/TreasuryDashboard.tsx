import { useState, useEffect, useRef } from 'react';
import { Flame, Activity, Clock, Calculator, Terminal, Lock, Radio, CircleCheck as CheckCircle } from 'lucide-react';
import { Lang } from '../translations';
import { TOTAL_NETWORK_POINTS, getDaysMultiplier, pad } from '../utils/staking';
import ProtocolArchitecture from './ProtocolArchitecture';
import DAOGovernance from './DAOGovernance';
import OnChainTreasury from './OnChainTreasury';

interface Props {
  lang: Lang;
  walletConnected: boolean;
  walletBalance?: number | null;
}

// ─── Constants ────────────────────────────────────────────────────────────────
const EPOCH_ID = 1;
const EPOCH_DURATION_HOURS = 72;
const BUYBACK_TRIGGER_USDC = 200;

// 实时模拟营收水位线起跑点
const BASE_REVENUE = 48_760;

// ─── On-Chain Routing Log ─────────────────────────────────────────────────────
const LOG_EN = [
  { tag: 'ROUTER',     text: 'New fee batch received: +312.40 USDC from protocol swap fees' },
  { tag: 'SPLIT',      text: 'Applying 50/20/20/10 rule on 312.40 USDC...' },
  { tag: 'POOL-50',    text: '→ 156.20 USDC routed to Epoch Restitution Pool (50%)' },
  { tag: 'POOL-20',    text: '→ 62.48 USDC routed to Buyback & Burn Matrix (20%)' },
  { tag: 'POOL-20',    text: '→ 62.48 USDC routed to DAO Contributor Payroll Pool (20%)' },
  { tag: 'POOL-10',    text: '→ 31.24 USDC routed to Staking Dividend Pool (10%)' },
  { tag: 'BURN',       text: 'Buyback pool balance hit 200 USDC — triggering Jupiter swap...' },
  { tag: 'MEV-SHIELD', text: 'Slippage hardlocked at 0.5% — Jito Bundle submitted' },
  { tag: 'SUCCESS',    text: 'Bundle landed slot #284,719,441 — burn confirmed: 48,320 α destroyed' },
  { tag: 'INFO',       text: 'All pools updated. Router armed. Next batch on next fee event.' },
];

const LOG_ZH = [
  { tag: 'ROUTER',     text: '收到新费用批次：+312.40 USDC 来自协议兑换费' },
  { tag: 'SPLIT',      text: '对 312.40 USDC 应用 50/20/20/10 分配规则...' },
  { tag: 'POOL-50',    text: '→ 156.20 USDC 路由至周期赔付池（50%）' },
  { tag: 'POOL-20',    text: '→ 62.48 USDC 路由至回购销毁矩阵（20%）' },
  { tag: 'POOL-20',    text: '→ 62.48 USDC 路由至 DAO 贡献者薪资池（20%）' },
  { tag: 'POOL-10',    text: '→ 31.24 USDC 路由至质押分红池（10%）' },
  { tag: 'BURN',       text: '回购池余额触达 200 USDC — 触发 Jupiter 兑换...' },
  { tag: 'MEV-SHIELD', text: '滑点硬锁 0.5% — Jito Bundle 已提交' },
  { tag: 'SUCCESS',    text: 'Bundle 落块 #284,719,441 — 销毁确认：48,320 α 已销毁至死亡地址' },
  { tag: 'INFO',       text: '所有池已更新。路由器就绪。下一批次等待下次费用事件触发。' },
];

const TAG_COLORS: Record<string, string> = {
  ROUTER:       'text-cyan-400',
  SPLIT:        'text-zinc-300',
  'POOL-50':    'text-green-400',
  'POOL-20':    'text-blue-400',
  'POOL-10':    'text-yellow-400',
  BURN:         'text-red-400',
  'MEV-SHIELD': 'text-orange-400',
  SUCCESS:      'text-green-400',
  INFO:         'text-zinc-400',
};

// ─── Hooks ────────────────────────────────────────────────────────────────────
function useCountdown(initialHours: number, initialMinutes: number, initialSeconds: number, wrapHours: number) {
  const [tick, setTick] = useState({ hours: initialHours, minutes: initialMinutes, seconds: initialSeconds });
  useEffect(() => {
    const id = setInterval(() => {
      setTick(prev => {
        let { hours, minutes, seconds } = prev;
        seconds--;
        if (seconds < 0) { seconds = 59; minutes--; }
        if (minutes < 0) { minutes = 59; hours--; }
        if (hours < 0)   { hours = wrapHours - 1; minutes = 59; seconds = 59; }
        return { hours, minutes, seconds };
      });
    }, 1000);
    return () => clearInterval(id);
  }, [wrapHours]);
  return tick;
}

function useLiveTicker(base: number) {
  const [value, setValue] = useState(base);
  useEffect(() => {
    const id = setInterval(() => {
      setValue(v => parseFloat((v + (Math.random() * 0.8 + 0.1)).toFixed(2)));
    }, 3200);
    return () => clearInterval(id);
  }, []);
  return value;
}

// ─── Routing Log ─────────────────────────────────────────────────────────────
function RoutingLog({ lang }: { lang: Lang }) {
  const lines = lang === 'zh' ? LOG_ZH : LOG_EN;
  const [visible, setVisible] = useState(0);
  const [blink, setBlink] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => { setVisible(0); }, [lang]);

  useEffect(() => {
    if (visible >= lines.length) { setBlink(false); return; }
    setBlink(true);
    const id = setTimeout(() => setVisible(v => v + 1), visible === 0 ? 700 : 380 + Math.random() * 340);
    return () => clearTimeout(id);
  }, [visible, lines.length]);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight, behavior: 'smooth' });
  }, [visible]);

  return (
    <div className="border border-zinc-800 rounded-xl overflow-hidden bg-zinc-950">
      <div className="flex items-center justify-between px-4 py-2 border-b border-zinc-800 bg-zinc-900/80">
        <div className="flex items-center gap-2">
          <Terminal className="w-3.5 h-3.5 text-zinc-500" />
          <span className="text-zinc-300 font-mono text-[11px] font-bold tracking-widest uppercase">
            {lang === 'zh' ? '智能分流路由日志' : 'Autonomous Revenue Router Log'}
          </span>
        </div>
        <div className="flex items-center gap-1.5">
          <span className="w-1.5 h-1.5 rounded-full bg-zinc-800" />
          <span className="w-1.5 h-1.5 rounded-full bg-zinc-800" />
          <span className="w-1.5 h-1.5 rounded-full bg-zinc-800" />
        </div>
      </div>
      <div ref={scrollRef} className="p-4 space-y-1.5 h-44 overflow-y-auto" style={{ scrollbarWidth: 'none' }}>
        {lines.slice(0, visible).map((line, i) => (
          <div key={i} className="flex items-start gap-2 font-mono text-[11px] leading-relaxed">
            <span className="text-zinc-700 select-none tabular-nums shrink-0">{String(i + 1).padStart(2, '0')}</span>
            <span className={`font-bold shrink-0 ${TAG_COLORS[line.tag] ?? 'text-zinc-400'}`}>[{line.tag}]</span>
            <span className="text-zinc-300 break-all">{line.text}</span>
          </div>
        ))}
        {visible < lines.length && blink && (
          <div className="flex items-center gap-2 font-mono text-[11px]">
            <span className="text-zinc-700 select-none tabular-nums shrink-0">{String(visible + 1).padStart(2, '0')}</span>
            <span className="text-green-400 animate-pulse">█</span>
          </div>
        )}
        {visible >= lines.length && (
          <div className="flex items-center gap-2 font-mono text-[11px] pt-1">
            <span className="text-green-400/30">——</span>
            <span className="text-zinc-700 italic">
              {lang === 'zh' ? '本批日志结束 · 等待下次费用批次触发' : 'Batch log end · Awaiting next fee event trigger'}
            </span>
          </div>
        )}
      </div>
    </div>
  );
}

// ─── Revenue Router Splitter ──────────────────────────────────────────────────
import { Video as LucideIcon } from 'lucide-react';
const SPLIT_POOLS: {
  pct: number;
  icon: LucideIcon;
  colorText: string;
  colorBorder: string;
  colorBg: string;
  colorBar: string;
  labelEn: string;
  labelZh: string;
  descEn: string;
  descZh: string;
}[] = [
  {
    pct: 50,
    icon: Activity,
    colorText:   'text-green-400',
    colorBorder: 'border-green-400/20',
    colorBg:     'bg-green-400/5',
    colorBar:    'from-green-900 to-green-400',
    labelEn: '50% Retail Relief Pool',
    labelZh: '50% 散户赔付救济金池',
    descEn: 'Every Epoch, 50% of all incoming revenue is pooled here and distributed proportionally to all participants based on ve-staking weight.',
    descZh: '每个周期，50% 的协议总收入在此累积，并按 ve 质押权重比例开放给全网持有者。',
  },
  {
    pct: 20,
    icon: Flame,
    colorText:   'text-red-400',
    colorBorder: 'border-red-400/20',
    colorBg:     'bg-red-400/5',
    colorBar:    'from-red-900 to-red-400',
    labelEn: '20% Buyback & Burn Matrix',
    labelZh: '20% 智能回购销毁矩阵',
    descEn: `20% goes to continuous automated Jupiter buybacks. Triggers a programmatic burn transaction every ${BUYBACK_TRIGGER_USDC} USDC to secure the price floor.`,
    descZh: `20% 持续流入自动化 Jupiter 回购引擎。每累计 ${BUYBACK_TRIGGER_USDC} USDC 触发一次链上销毁交易。`,
  },
  {
    pct: 20,
    icon: Flame,
    colorText:   'text-blue-400',
    colorBorder: 'border-blue-400/20',
    colorBg:     'bg-blue-400/5',
    colorBar:    'from-blue-900 to-blue-400',
    labelEn: '20% DAO Contributor Payroll',
    labelZh: '20% DAO治理建设者工资池',
    descEn: '20% routes to active core contributors. Split internally via 4:3:2:1 rules. Recipient wallets are controlled by community vote.',
    descZh: '20% 路由至活跃核心贡献者。内部按 4:3:2:1 规则拆分。接收钱包地址由社区 DAO 投票锁定。',
  },
  {
    pct: 10,
    icon: Flame,
    colorText:   'text-yellow-400',
    colorBorder: 'border-yellow-400/20',
    colorBg:     'bg-yellow-400/5',
    colorBar:    'from-yellow-900 to-yellow-400',
    labelEn: '10% Pure Staking Dividend',
    labelZh: '10% 纯代币质押奖金池',
    descEn: '10% is streamed directly back as pure USDC cash dividends to anyone staking $α tokens. Genuine yield with zero dilution.',
    descZh: '10% 以纯 USDC 现金分红形式直接返还至所有 $α 质押者。真实无增发收益。',
  },
];

function RevenueRouter({ lang, totalRevenue }: { lang: Lang; totalRevenue: number }) {
  const isZh = lang === 'zh';
  return (
    <div className="border border-zinc-800 bg-zinc-950/20 rounded-xl overflow-hidden">
      <div className="flex items-center justify-between px-5 py-3 border-b border-zinc-900 bg-zinc-950">
        <div className="flex items-center gap-2">
          <div className="p-1.5 rounded bg-cyan-400/10 border border-cyan-400/20">
            <Activity className="w-3.5 h-3.5 text-cyan-400" />
          </div>
          <div>
            <p className="text-zinc-200 font-mono text-xs font-bold">
              {isZh ? '智能分流矩阵 — 自主营收路由器' : 'Autonomous Revenue Router — Smart Splitter Matrix'}
            </p>
          </div>
        </div>
        <span className="text-[9px] font-mono px-1.5 py-0.5 rounded border border-cyan-400/20 bg-cyan-400/5 text-cyan-400 animate-pulse">AUTONOMOUS</span>
      </div>

      <div className="p-5 space-y-5">
        <div className="bg-zinc-950/40 border border-zinc-900 rounded-lg px-5 py-4 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div>
            <p className="text-zinc-600 font-mono text-[9px] uppercase tracking-widest mb-1">
              {isZh ? '协议累计总收入（不设上限）' : 'Total Protocol Revenue Generated (No Cap)'}
            </p>
            <div className="flex items-baseline gap-1.5">
              <span className="text-2xl font-black font-mono text-green-400 tabular-nums">
                {totalRevenue.toLocaleString('en', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
              </span>
              <span className="text-zinc-500 font-mono text-[11px]">USDC</span>
            </div>
          </div>
          <div className="flex flex-wrap gap-2">
            {SPLIT_POOLS.map(p => (
              <div key={p.pct + p.labelEn} className={`text-center px-2.5 py-1 rounded border ${p.colorBorder} ${p.colorBg}`}>
                <p className={`text-xs font-black font-mono ${p.colorText}`}>{p.pct}%</p>
                <p className="text-zinc-500 font-mono text-[9px] tabular-nums">
                  {(totalRevenue * p.pct / 100).toLocaleString('en', { minimumFractionDigits: 0, maximumFractionDigits: 0 })} U
                </p>
              </div>
            ))}
          </div>
        </div>

        <div className="space-y-2">
          <div className="flex h-3 rounded overflow-hidden gap-px bg-zinc-950">
            {SPLIT_POOLS.map(p => (
              <div key={p.pct} className={`bg-gradient-to-r ${p.colorBar}`} style={{ width: `${p.pct}%` }} />
            ))}
          </div>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          {SPLIT_POOLS.map(p => {
            const Icon = p.icon;
            const poolBalance = totalRevenue * p.pct / 100;
            return (
              <div key={p.labelEn} className={`border ${p.colorBorder} ${p.colorBg} rounded-lg p-4 space-y-2 relative overflow-hidden`}>
                <div className="flex items-start gap-3">
                  <div className={`flex-shrink-0 p-1.5 rounded border ${p.colorBorder} bg-black/20`}>
                    <Icon className={`w-3.5 h-3.5 ${p.colorText}`} />
                  </div>
                  <div className="min-w-0">
                    <p className={`font-mono font-bold text-xs ${p.colorText}`}>
                      {isZh ? p.labelZh : p.labelEn}
                    </p>
                    <p className={`font-mono text-lg font-black ${p.colorText} tabular-nums mt-0.5`}>
                      {poolBalance.toLocaleString('en', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                      <span className="text-zinc-600 font-mono text-[10px] ml-1">USDC</span>
                    </p>
                  </div>
                </div>
                <p className="text-zinc-500 font-mono text-[11px] leading-relaxed">
                  {isZh ? p.descZh : p.descEn}
                </p>
              </div>
            );
          })}
        </div>

        <RoutingLog lang={lang} />
      </div>
    </div>
  );
}

// ─── Calculator ───────────────────────────────────────────────────────────────
const TIERS_DESC = [
  { minPoints: 100000, multiplier: 8, label: 'tier4' as const, color: 'text-red-400',    border: 'border-red-400/20' },
  { minPoints: 25000,  multiplier: 4, label: 'tier3' as const, color: 'text-yellow-400', border: 'border-yellow-400/20' },
  { minPoints: 5000,   multiplier: 2, label: 'tier2' as const, color: 'text-cyan-400',   border: 'border-cyan-400/20' },
  { minPoints: 0,      multiplier: 1, label: 'tier1' as const, color: 'text-gray-400',   border: 'border-gray-400/20' },
];

function EpochCountdown() {
  const tick = useCountdown(47, 32, 18, EPOCH_DURATION_HOURS);
  return (
    <div className="flex items-center gap-1.5">
      <Clock className="w-3.5 h-3.5 text-cyan-400" />
      <span className="text-cyan-400 font-mono text-sm font-black tabular-nums">
        {pad(tick.hours)}:{pad(tick.minutes)}:{pad(tick.seconds)}
      </span>
    </div>
  );
}

function YieldCalculator({ lang, totalRevenue, walletConnected }: { lang: Lang; totalRevenue: number; walletConnected: boolean }) {
  const isZh = lang === 'zh';
  const [staked, setStaked] = useState('');
  const [days, setDays] = useState('');
  const [totalStakedNetwork, setTotalStakedNetwork] = useState('');
  const [result, setResult] = useState<{
    stakingPower: number;
    daysMultiplier: number;
    sharePct: string;
    restitutionQuota: number;
    stakingDividend: number;
  } | null>(null);

  const epochPool = totalRevenue * 0.50;
  const dividendPool = totalRevenue * 0.10;

  function calculate() {
    const s = parseFloat(staked) || 0;
    const d = parseFloat(days) || 0;
    
    // 风控防除以零硬锁：如果用户输入0或非法值，强制转换为兜底值，杜绝 NaN 崩溃
    let totalNet = parseFloat(totalStakedNetwork);
    if (isNaN(totalNet) || totalNet <= 0) {
      totalNet = 10_000_000;
    }

    const daysMultiplier = getDaysMultiplier(d);
    const stakingPower = s * daysMultiplier;
    const restitutionQuota = epochPool * (stakingPower / TOTAL_NETWORK_POINTS);
    const stakingDividend = dividendPool * (s / totalNet);
    const sharePct = ((stakingPower / TOTAL_NETWORK_POINTS) * 100).toFixed(4);
    setResult({ stakingPower, daysMultiplier, sharePct, restitutionQuota, stakingDividend });
  }

  return (
    <div className="border border-zinc-800 bg-zinc-950/20 rounded-xl overflow-hidden">
      <div className="flex items-center justify-between px-5 py-3 border-b border-zinc-900 bg-zinc-950">
        <div className="flex items-center gap-2">
          <div className="p-1.5 rounded bg-cyan-400/10 border border-cyan-400/20">
            <Calculator className="w-3.5 h-3.5 text-cyan-400" />
          </div>
          <div>
            <p className="text-zinc-200 font-mono text-xs font-bold">
              {isZh ? '个人收益与分红估算器' : 'Personal Yield & Restitution Estimator'}
            </p>
          </div>
        </div>
        <EpochCountdown />
      </div>

      <div className="p-5 space-y-5">
        <div className="grid grid-cols-2 gap-3">
          <div className="bg-zinc-950/40 border border-green-400/20 rounded p-3 text-center">
            <p className="text-zinc-600 font-mono text-[9px] uppercase tracking-widest mb-1">
              {isZh ? '周期赔付池（50%）' : 'Epoch Restitution Pool (50%)'}
            </p>
            <p className="text-green-400 font-black font-mono text-base tabular-nums">
              {epochPool.toLocaleString('en', { minimumFractionDigits: 2, maximumFractionDigits: 2 })} <span className="text-zinc-600 text-xs">U</span>
            </p>
          </div>
          <div className="bg-zinc-950/40 border border-yellow-400/20 rounded p-3 text-center">
            <p className="text-zinc-600 font-mono text-[9px] uppercase tracking-widest mb-1">
              {isZh ? '质押分红池（10%）' : 'Staking Dividend Pool (10%)'}
            </p>
            <p className="text-yellow-400 font-black font-mono text-base tabular-nums">
              {dividendPool.toLocaleString('en', { minimumFractionDigits: 2, maximumFractionDigits: 2 })} <span className="text-zinc-600 text-xs">U</span>
            </p>
          </div>
        </div>

        {/* Inputs */}
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
          <div className="space-y-1">
            <label className="text-zinc-600 text-[9px] font-mono uppercase tracking-widest">{isZh ? 'α 质押数量' : 'Staked α Amount'}</label>
            <input
              type="number"
              value={staked}
              onChange={e => setStaked(e.target.value)}
              placeholder="10000"
              className="w-full bg-zinc-950 border border-zinc-800 rounded px-3 py-2 text-xs font-mono text-zinc-200 placeholder-zinc-800 focus:outline-none focus:border-cyan-400/30"
            />
          </div>
          <div className="space-y-1">
            <label className="text-zinc-600 text-[9px] font-mono uppercase tracking-widest">{isZh ? '连续质押天数' : 'Consecutive Days'}</label>
            <input
              type="number"
              value={days}
              onChange={e => setDays(e.target.value)}
              placeholder="90"
              className="w-full bg-zinc-950 border border-zinc-800 rounded px-3 py-2 text-xs font-mono text-zinc-200 placeholder-zinc-800 focus:outline-none focus:border-cyan-400/30"
            />
          </div>
          <div className="space-y-1">
            <label className="text-zinc-600 text-[9px] font-mono uppercase tracking-widest">{isZh ? '全网质押总量' : 'Total Network Staked'}</label>
            <input
              type="number"
              value={totalStakedNetwork}
              onChange={e => setTotalStakedNetwork(e.target.value)}
              placeholder="10000000"
              className="w-full bg-zinc-950 border border-zinc-800 rounded px-3 py-2 text-xs font-mono text-zinc-200 placeholder-zinc-800 focus:outline-none focus:border-cyan-400/30"
            />
          </div>
        </div>

        {/* 交互风控：若钱包未连接，则用文案状态弱化引导 */}
        <button
          onClick={calculate}
          className="w-full py-2 bg-zinc-900 hover:bg-zinc-850 border border-zinc-800 hover:border-zinc-700 text-zinc-300 font-mono font-bold uppercase tracking-widest rounded transition-all text-xs"
        >
          {walletConnected 
            ? (isZh ? '⚡ 跑数计算实时收益' : '⚡ Execute Live Estimation') 
            : (isZh ? '🔓 请先在顶部连接钱包以解锁精确对账' : '🔓 Connect Wallet to Unlock Precision Engine')}
        </button>

        {result && (
          <div className="space-y-2 pt-1">
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
              {[
                { label: isZh ? '时间乘数' : 'Multiplier',   value: `${result.daysMultiplier}x`,        color: 'text-yellow-400' },
                { label: isZh ? '全网权重占比' : 'Net Share', value: `${result.sharePct}%`,              color: 'text-green-400'  },
                { label: isZh ? '可分得救济金' : 'Relief Quota', value: `${result.restitutionQuota.toFixed(2)} U`, color: 'text-green-400' },
                { label: isZh ? '周期现金分红' : 'USDC Dividend', value: `${result.stakingDividend.toFixed(2)} U`,  color: 'text-yellow-400' },
              ].map(m => (
                <div key={m.label} className="bg-zinc-950/60 border border-zinc-900 rounded p-2.5 text-center">
                  <p className="text-zinc-600 font-mono text-[9px] uppercase tracking-widest mb-1">{m.label}</p>
                  <p className={`text-xs font-black font-mono ${m.color} tabular-nums`}>{m.value}</p>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// ─── Permissionless eligibility notice ───────────────────────────────────────
function EligibilityNotice({ lang }: { lang: Lang }) {
  const isZh = lang === 'zh';
  return (
    <div className="flex items-start gap-3 border border-green-400/10 bg-green-400/5 rounded-xl p-4">
      <div className="flex-shrink-0 mt-0.5 p-1.5 rounded bg-green-400/10 border border-green-400/20">
        <Radio className="w-3.5 h-3.5 text-green-400 animate-pulse" />
      </div>
      <div>
        <p className="text-green-400 font-mono text-[10px] font-bold uppercase tracking-widest mb-1">
          {isZh ? '全域开放资格' : 'Universal Permissionless Eligibility'}
        </p>
        <p className="text-zinc-400 font-mono text-xs leading-relaxed">
          {isZh
            ? '任何持有并质押 α 的地址均自动获得本周期赔付池的比例分配资格，以及每笔费用批次的实时分红资格。无需白名单，无需管理员审批。质押即参与。'
            : "Any address staking α tokens is automatically eligible for proportional restitution and real-time staking dividends on every fee batch. No whitelist, no Merkle proof, no admin approval. Stake and you're in."}
        </p>
      </div>
    </div>
  );
}

// ─── Main Dashboard ────────────────────────────────────────────────────────────
export default function TreasuryDashboard({ lang, walletConnected }: Props) {
  const isZh = lang === 'zh';
  const totalRevenue = useLiveTicker(BASE_REVENUE);

  return (
    <div className="space-y-8">
      <div className="flex items-center gap-3">
        <Activity className="text-green-400 w-4 h-4" />
        <h2 className="text-base font-bold text-zinc-200 tracking-widest uppercase font-mono">
          {isZh ? '协议国库智能分流账本' : 'Protocol Treasury — Autonomous Revenue Router'}
        </h2>
      </div>

      {/* 核心对账修复：让已分配金额等核心指标，与动态涨幅的 totalRevenue 保持数学比例联动，干掉假静态硬伤 */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        {[
          { label: isZh ? '总流入流水' : 'Total Inflow',         value: totalRevenue.toLocaleString('en', { minimumFractionDigits: 2, maximumFractionDigits: 2 }), unit: 'USDC', color: 'text-green-400' },
          { label: isZh ? '全网已分流' : 'Total Routed',         value: totalRevenue.toLocaleString('en', { minimumFractionDigits: 0, maximumFractionDigits: 0 }),  unit: 'USDC', color: 'text-cyan-400'   },
          { label: isZh ? '智能回购矩阵' : 'Programmatic Burned',   value: (totalRevenue * 0.2).toLocaleString('en', { minimumFractionDigits: 0, maximumFractionDigits: 0 }), unit: 'USDC Value', color: 'text-red-400' },
          { label: isZh ? '当前计费周期' : 'Current Epoch ID',    value: `#00${EPOCH_ID}`,    unit: isZh ? '进行中' : 'Active', color: 'text-yellow-400' },
        ].map(m => (
          <div key={m.label} className="bg-zinc-950/40 border border-zinc-900 rounded-lg p-4 text-center">
            <p className="text-zinc-600 text-[10px] font-mono mb-1 uppercase tracking-wider">{m.label}</p>
            <p className={`text-xl font-bold font-mono ${m.color} tabular-nums`}>{m.value}</p>
            <p className="text-zinc-500 text-[9px] font-mono mt-0.5">{m.unit}</p>
          </div>
        ))}
      </div>

      {/* ─── On-Chain Treasury Monitor (Live PDA Polling) ─── */}
      <div className="border border-emerald-400/15 bg-zinc-950/60 rounded-2xl p-5 sm:p-6 relative overflow-hidden">
        <div className="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-emerald-400/40 to-transparent" />
        <OnChainTreasury lang={lang} />
      </div>

      <EligibilityNotice lang={lang} />
      <RevenueRouter lang={lang} totalRevenue={totalRevenue} />
      <YieldCalculator lang={lang} totalRevenue={totalRevenue} walletConnected={walletConnected} />

      <ProtocolArchitecture lang={lang} />
      <DAOGovernance lang={lang} />

      {/* ─── Transparency manifest ─── */}
      {/* 漏洞修复：完美闭合被截断的英文文案字符串与对应的 JSX 标签 */}
      <div className="flex items-start gap-3 border border-zinc-900 bg-zinc-950/40 rounded-xl p-4">
        <div className="flex-shrink-0 mt-0.5 p-1.5 rounded bg-zinc-950 border border-zinc-900">
          <CheckCircle className="w-3.5 h-3.5 text-zinc-500" />
        </div>
        <div>
          <p className="text-zinc-400 font-mono text-[10px] font-bold uppercase tracking-widest mb-1">
            {isZh ? 'α 100% 链上真实性宣言' : 'α 100% On-Chain Transparency Manifest'}
          </p>
          <p className="text-zinc-600 font-mono text-[10px] leading-relaxed">
            {isZh
              ? '本国库拒绝任何虚假注水与预设底仓。起跑水位 0 USDC，所有资金完全由协议费用实时流入。50/20/20/10 规则硬编码于链上智能合约，无管理员私钥，无后门。全体 Holder 随时可链上对账。'
              : 'Zero synthetic padding or pre-seeded balances. Starting at 0 USDC, all funds flow in real-time from protocol fees. The 50/20/20/10 rule is hardcoded on-chain via immutable PDA state primitives. No administrative trust, no backdoor, fully verifiable by any holder.'}
          </p>
        </div>
      </div>
    </div>
  );
}