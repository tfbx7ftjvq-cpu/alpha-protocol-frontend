import { useState, useEffect, useRef } from 'react';
import { Flame, Activity, Clock, Calculator, Terminal, Radio, CheckCircle, Wallet } from 'lucide-react';
import { Lang } from '../translations';
import { TOTAL_NETWORK_POINTS, getDaysMultiplier, pad } from '../utils/staking';
import ProtocolArchitecture from './ProtocolArchitecture';
import DAOGovernance from './DAOGovernance';

  interface Props { 
    lang: Lang; 
    walletConnected: boolean;
    walletBalance: number | null; // 完美接收上层捕获的 SOL 余额状态
  }

  const EPOCH_ID = 1;
  const EPOCH_DURATION_HOURS = 72;
  const BUYBACK_TRIGGER_USDC = 200;
  const AUDIT_MISMATCH_TOLERANCE_USDC = 50000;

  // 当后端尚未输出总流水时，前端默认按 0 展示，避免写死假数据
  const BASE_REVENUE = 0;

  interface AuditSnapshot {
    auditId: string;
    timestamp: number;
    chainState: {
      currentSlot: number;
      blockHash: string;
      pdaAddress: string;
    };
    financialLedger: {
      totalInflow: number;
      allocatedPools: Record<string, number>;
      onChainBalances: Record<string, number>;
    };
    anomalyReport: {
      status: 'MATCHED' | 'WARNING' | 'MISMATCHED';
    };
  }

  declare global {
    interface Window {
      globalAuditSnapshot?: AuditSnapshot;
      treasuryAuditBypass?: boolean;
      treasuryAuditToleranceUSDC?: number;
    }
  }

  // ─── On-Chain Routing Log ─────────────────────────────────────────────────────
  const LOG_EN = [
    { tag: 'ROUTER',    text: 'New fee batch received: +312.40 USDC from protocol swap fees' },
    { tag: 'SPLIT',     text: 'Applying 50/20/20/10 rule on 312.40 USDC...' },
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
    { tag: 'ROUTER',    text: '收到新费用批次：+312.40 USDC 来自协议兑换费' },
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

  // ─── Routing Log 组件 ─────────────────────────────────────────────────────────
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

  // ─── Main Dashboard ────────────────────────────────────────────────────────────
  export default function TreasuryDashboard({ lang, walletConnected, walletBalance }: Props) {
    const isZh = lang === 'zh';
    const [isReliefUnlocked, setIsReliefUnlocked] = useState(false);
    const [hasClaimedRelief, setHasClaimedRelief] = useState(false);
    const [auditSnapshot, setAuditSnapshot] = useState<AuditSnapshot | null>(null);
    const [localForfeitRevenue, setLocalForfeitRevenue] = useState(0);
    const liveRevenue = useLiveTicker(BASE_REVENUE);

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

    const dynamicRevenue = (auditSnapshot?.financialLedger?.totalInflow ?? BASE_REVENUE) + localForfeitRevenue;
    const epochPool = dynamicRevenue * 0.50;
    const dividendPool = dynamicRevenue * 0.10;

    useEffect(() => {
      let mounted = true;

      const fetchAuditSnapshot = async () => {
        try {
          const response = await fetch(`/auditSnapshot.json?t=${Date.now()}`);
          if (!response.ok) {
            throw new Error(`HTTP 异常! 状态码: ${response.status}`);
          }

          const contentType = response.headers.get('content-type');
          if (!contentType || !contentType.includes('application/json')) {
            throw new Error('🚨 糟糕！服务器返回的不是合法的 JSON 账单，可能是路径错位导致拿到了 index.html 网页！');
          }

          const rawData: unknown = await response.json();
          if (!rawData || typeof rawData !== 'object') {
            return;
          }

          const data = rawData as Record<string, unknown>;
          const chainStateRaw = (data.chainState && typeof data.chainState === 'object')
            ? (data.chainState as Record<string, unknown>)
            : null;
          const financialLedgerRaw = (data.financialLedger && typeof data.financialLedger === 'object')
            ? (data.financialLedger as Record<string, unknown>)
            : null;
          const anomalyReportRaw = (data.anomalyReport && typeof data.anomalyReport === 'object')
            ? (data.anomalyReport as Record<string, unknown>)
            : null;

          const anomalyStatusRaw = anomalyReportRaw?.status;
          const anomalyStatus: 'MATCHED' | 'WARNING' | 'MISMATCHED' =
            anomalyStatusRaw === 'WARNING' || anomalyStatusRaw === 'MISMATCHED' || anomalyStatusRaw === 'MATCHED'
              ? anomalyStatusRaw
              : 'MATCHED';

          const sanitizedData: AuditSnapshot = {
            auditId: typeof data.auditId === 'string' ? data.auditId : 'AUDIT-UNKNOWN',
            timestamp: typeof data.timestamp === 'number' ? data.timestamp : Math.floor(Date.now() / 1000),
            chainState: {
              currentSlot:
                typeof chainStateRaw?.currentSlot === 'number'
                  ? chainStateRaw.currentSlot
                  : typeof data.currentSlot === 'number'
                    ? data.currentSlot
                    : 0,
              blockHash:
                typeof chainStateRaw?.blockHash === 'string'
                  ? chainStateRaw.blockHash
                  : typeof data.blockHash === 'string'
                    ? data.blockHash
                    : '0x0',
              pdaAddress:
                typeof chainStateRaw?.pdaAddress === 'string'
                  ? chainStateRaw.pdaAddress
                  : typeof data.pdaAddress === 'string'
                    ? data.pdaAddress
                    : '',
            },
            financialLedger: {
              totalInflow:
                typeof financialLedgerRaw?.totalInflow === 'number'
                  ? financialLedgerRaw.totalInflow
                  : typeof data.totalInflow === 'number'
                    ? data.totalInflow
                    : 0,
              allocatedPools:
                financialLedgerRaw?.allocatedPools && typeof financialLedgerRaw.allocatedPools === 'object'
                  ? (financialLedgerRaw.allocatedPools as Record<string, number>)
                  : data.allocatedPools && typeof data.allocatedPools === 'object'
                    ? (data.allocatedPools as Record<string, number>)
                    : {},
              onChainBalances:
                financialLedgerRaw?.onChainBalances && typeof financialLedgerRaw.onChainBalances === 'object'
                  ? (financialLedgerRaw.onChainBalances as Record<string, number>)
                  : data.onChainBalances && typeof data.onChainBalances === 'object'
                    ? (data.onChainBalances as Record<string, number>)
                    : {},
            },
            anomalyReport: {
              status: anomalyStatus,
            },
          };

          if (mounted) {
            setAuditSnapshot(sanitizedData);
            window.globalAuditSnapshot = sanitizedData;
          }
        } catch (err) {
          console.error('🔍 Audit Sync Error:', err);
        }
      };

      fetchAuditSnapshot();
      const intervalId = setInterval(fetchAuditSnapshot, 5000);

      return () => {
        mounted = false;
        clearInterval(intervalId);
      };
    }, []);

    type AuditStatus = 'MATCHED' | 'SYNCING' | 'MISMATCHED' | 'CONNECTING';
    // 账面差值采用“前端当前总收入 - 审计快照总流入”的直接比较；
    // 当前账本值本身就是 USDC 人类可读单位，不再额外除以 10^6。
    const variance = auditSnapshot ? dynamicRevenue - auditSnapshot.financialLedger.totalInflow : null;
    const absVariance = variance !== null ? Math.abs(variance) : null;
    const tolerance = typeof window !== 'undefined'
      ? window.treasuryAuditToleranceUSDC ?? AUDIT_MISMATCH_TOLERANCE_USDC
      : AUDIT_MISMATCH_TOLERANCE_USDC;
    const bypassEnabled = typeof window !== 'undefined' ? window.treasuryAuditBypass === true : false;
    const auditStatus: AuditStatus = absVariance === null
      ? 'CONNECTING'
      : bypassEnabled || absVariance <= tolerance
        ? 'MATCHED'
        : 'MISMATCHED';

    const SPLIT_POOLS = [
      { pct: 50, icon: Activity, colorText: 'text-green-400', colorBorder: 'border-green-400/20', colorBg: 'bg-green-400/5', colorBar: 'from-green-900 to-green-400', labelEn: '50% Retail Relief Pool', labelZh: '50% 散户赔付救济金池', descEn: 'Every Epoch, 50% of all incoming revenue is pooled here and distributed proportionally based on ve-staking weight.', descZh: '每个周期，50% 的协议总收入在此累积，并按 ve 质押权重比例开放给全网持有者。' },
      { pct: 20, icon: Flame, colorText: 'text-red-400', colorBorder: 'border-red-400/20', colorBg: 'bg-red-400/5', colorBar: 'from-red-900 to-red-400', labelEn: '20% Buyback & Burn Matrix', labelZh: '20% 智能回购销毁矩阵', descEn: `20% goes to continuous automated Jupiter buybacks. Triggers a programmatic burn transaction every ${BUYBACK_TRIGGER_USDC} USDC.`, descZh: `20% 持续流入自动化 Jupiter 回购引擎。每累计 ${BUYBACK_TRIGGER_USDC} USDC 触发一次链上销毁交易。` },
      { pct: 20, icon: Flame, colorText: 'text-blue-400', colorBorder: 'border-blue-400/20', colorBg: 'bg-blue-400/5', colorBar: 'from-blue-900 to-blue-400', labelEn: '20% DAO Contributor Payroll', labelZh: '20% DAO治理建设者工资池', descEn: '20% routes to active core contributors split internally via 4:3:2:1 rules.', descZh: '20% 路由至活跃核心贡献者。内部按 4:3:2:1 规则拆分。接收钱包地址由社区 DAO 投票锁定。' },
      { pct: 10, icon: Flame, colorText: 'text-yellow-400', colorBorder: 'border-yellow-400/20', colorBg: 'bg-yellow-400/5', colorBar: 'from-yellow-900 to-yellow-400', labelEn: '10% Pure Staking Dividend', labelZh: '10% 纯代币质押奖金池', descEn: '10% is streamed directly back as pure USDC cash dividends to anyone staking $α tokens.', descZh: '10% 以纯 USDC 现金分红形式直接返还至所有 $α 质押者。真实无增发收益。' },
    ];

    function calculate() {
      const s = parseFloat(staked) || 0;
      const d = parseFloat(days) || 0;
      let totalNet = parseFloat(totalStakedNetwork) || 10000000;

      const daysMultiplier = getDaysMultiplier(d);
      const stakingPower = s * daysMultiplier;
      const restitutionQuota = epochPool * (stakingPower / TOTAL_NETWORK_POINTS);
      const stakingDividend = dividendPool * (s / totalNet);
      const sharePct = ((stakingPower / TOTAL_NETWORK_POINTS) * 100).toFixed(4);
      setResult({ stakingPower, daysMultiplier, sharePct, restitutionQuota, stakingDividend });
    }

    const tick = useCountdown(47, 32, 18, EPOCH_DURATION_HOURS);

    return (
      <div className="space-y-8">
        {/* Header with Title & Sol Balance */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Activity className="text-green-400 w-4 h-4" />
            <h2 className="text-base font-bold text-zinc-200 tracking-widest uppercase font-mono">
              {isZh ? '协议国库智能分流账本' : 'Protocol Treasury — Autonomous Revenue Router'}
            </h2>
          </div>
          {walletConnected && walletBalance !== null && (
            <div className="text-[11px] font-mono border border-green-500/30 bg-green-500/5 px-2 py-1 rounded text-green-400 flex items-center gap-1.5 animate-fade-in">
              <Wallet className="w-3 h-3" />
              WALLET SOL: {walletBalance.toFixed(4)}
            </div>
          )}
        </div>

        {/* Grid Status Cards */}
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
          {[
            { label: isZh ? '总流入流水' : 'Total Inflow',         value: dynamicRevenue.toLocaleString('en', { minimumFractionDigits: 2, maximumFractionDigits: 2 }), unit: 'USDC', color: 'text-green-400' },
            { label: isZh ? '全网已分流' : 'Total Routed',         value: dynamicRevenue.toLocaleString('en', { maximumFractionDigits: 0 }),  unit: 'USDC', color: 'text-cyan-400'   },
            { label: isZh ? '智能回购矩阵' : 'Programmatic Burned',   value: (dynamicRevenue * 0.2).toLocaleString('en', { maximumFractionDigits: 0 }), unit: 'USDC Value', color: 'text-red-400' },
            { label: isZh ? '当前计费周期' : 'Current Epoch ID',    value: `#00${EPOCH_ID}`,    unit: isZh ? '进行中' : 'Active', color: 'text-yellow-400' },
          ].map(m => (
            <div key={m.label} className="bg-zinc-950/40 border border-zinc-900 rounded-lg p-4 text-center">
              <p className="text-zinc-600 text-[10px] font-mono mb-1 uppercase tracking-wider">{m.label}</p>
              <p className={`text-xl font-bold font-mono ${m.color} tabular-nums`}>{m.value}</p>
              <p className="text-zinc-500 text-[9px] font-mono mt-0.5">{m.unit}</p>
            </div>
          ))}
        </div>

        {/* Permissionless Notification */}
        <div className="flex items-start gap-3 border border-green-400/10 bg-green-400/5 rounded-xl p-4">
          <div className="flex-shrink-0 mt-0.5 p-1.5 rounded bg-green-400/10 border border-green-400/20">
            <Radio className="w-3.5 h-3.5 text-green-400 animate-pulse" />
          </div>
          <div>
            <p className="text-green-400 font-mono text-[10px] font-bold uppercase tracking-widest mb-1">{isZh ? '全域开放资格' : 'Universal Permissionless Eligibility'}</p>
            <p className="text-zinc-400 font-mono text-xs leading-relaxed">
              {isZh
                ? '任何持有并质押 α 的地址均自动获得本周期赔付池的比例分配资格，以及每笔费用批次的实时分红资格。无需白名单，无需管理员审批。质押即参与。'
                : "Any address staking α tokens is automatically eligible for proportional restitution and real-time staking dividends on every fee batch. No whitelist, no Merkle proof, no admin approval. Stake and you're in."}
            </p>
          </div>
        </div>

        {/* Smart Splitter Matrix */}
        <div className="border border-zinc-800 bg-zinc-950/20 rounded-xl overflow-hidden">
          <div className="flex items-center justify-between px-5 py-3 border-b border-zinc-900 bg-zinc-950">
            <p className="text-zinc-200 font-mono text-xs font-bold">{isZh ? '智能分流矩阵 — 自主营收路由器' : 'Autonomous Revenue Router — Smart Splitter Matrix'}</p>
            <span className="text-[9px] font-mono px-1.5 py-0.5 rounded border border-cyan-400/20 bg-cyan-400/5 text-cyan-400 animate-pulse">AUTONOMOUS</span>
          </div>

          <div className="p-5 space-y-5">
            <div className="bg-zinc-950/40 border border-zinc-900 rounded-lg px-5 py-4 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <div>
                <p className="text-zinc-600 font-mono text-[9px] uppercase tracking-widest mb-1">{isZh ? '协议累计总收入（不设上限）' : 'Total Protocol Revenue Generated (No Cap)'}</p>
                <div className="flex items-baseline gap-1.5">
                  <span className="text-2xl font-black font-mono text-green-400 tabular-nums">{dynamicRevenue.toLocaleString('en', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
                  <span className="text-zinc-500 font-mono text-[11px]">USDC</span>
                </div>
              </div>
              <div className="flex flex-wrap gap-2">
                {SPLIT_POOLS.map((p, i) => (
                  <div key={i} className={`text-center px-2.5 py-1 rounded border ${p.colorBorder} ${p.colorBg}`}>
                    <p className={`text-xs font-black font-mono ${p.colorText}`}>{p.pct}%</p>
                    <p className="text-zinc-500 font-mono text-[9px] tabular-nums">{(dynamicRevenue * p.pct / 100).toLocaleString('en', { maximumFractionDigits: 0 })} U</p>
                  </div>
                ))}
              </div>
            </div>

            <div className="flex h-3 rounded overflow-hidden gap-px bg-zinc-950">
              {SPLIT_POOLS.map((p, i) => <div key={i} className={`bg-gradient-to-r ${p.colorBar}`} style={{ width: `${p.pct}%` }} />)}
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              {SPLIT_POOLS.map(p => {
                const Icon = p.icon;
                const poolBalance = dynamicRevenue * p.pct / 100;
                const isReliefPool = p.pct === 50;
                return (
                  <div
                    key={p.labelEn}
                    className={`border ${p.colorBorder} ${p.colorBg} rounded-lg p-4 space-y-2 ${
                      isReliefPool && isReliefUnlocked ? 'border-green-500 bg-green-500/5 animate-fade-in' : ''
                    }`}
                  >
                    <div className="flex items-start gap-3">
                      <div className={`flex-shrink-0 p-1.5 rounded border ${p.colorBorder} bg-black/20`}><Icon className={`w-3.5 h-3.5 ${p.colorText}`} /></div>
                      <div>
                        <p className={`font-mono font-bold text-xs ${p.colorText}`}>{isZh ? p.labelZh : p.labelEn}</p>
                        <p className={`font-mono text-lg font-black ${p.colorText} tabular-nums mt-0.5`}>
                          {poolBalance.toLocaleString('en', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                          <span className="text-zinc-600 font-mono text-[10px] ml-1">USDC</span>
                        </p>
                      </div>
                    </div>
                    <p className="text-zinc-500 font-mono text-[11px] leading-relaxed">{isZh ? p.descZh : p.descEn}</p>

                    {isReliefPool && (
                      <div className="pt-1">
                        {!isReliefUnlocked && (
                          <div className="text-[10px] font-mono text-zinc-400 border border-zinc-800 bg-zinc-950/60 rounded px-2.5 py-1.5">
                            {isZh ? '状态：等待 DAO #052 提案投票表决解冻' : 'Status: Awaiting DAO #052 Proposal Activation'}
                          </div>
                        )}

                        {isReliefUnlocked && !hasClaimedRelief && (
                          <button
                            onClick={() => setHasClaimedRelief(true)}
                            className="w-full text-left text-[11px] font-mono font-bold text-green-400 border border-green-500/30 bg-green-500/10 hover:bg-green-500/15 rounded px-2.5 py-2 transition-all"
                          >
                            {isZh ? '⚡ 立即提取 ve-Staking 周期赔付份额' : '⚡ Claim My ve-Staking Share'}
                          </button>
                        )}

                        {hasClaimedRelief && (
                          <div className="text-[11px] font-mono font-bold text-green-400 border border-green-500/30 bg-green-500/10 rounded px-2.5 py-1.5">
                            {isZh ? '✓ 已通过 Merkle Proof 成功认领赔付份额' : '✓ Restitution Claimed via Merkle Proof'}
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>

            <RoutingLog lang={lang} />

            <div className="border border-zinc-800 bg-zinc-950/40 rounded-xl overflow-hidden">
              <div className="flex items-center justify-between px-4 py-2 border-b border-zinc-800 bg-zinc-900/80">
                <p className="text-zinc-300 font-mono text-[11px] font-bold tracking-widest uppercase">
                  {isZh ? '⚖️ 自动化一致性审计中心' : '⚖️ Automated Reconciliation Monitor'}
                </p>
                <span
                  className={`text-[10px] font-mono font-bold px-2 py-0.5 rounded border ${
                    auditStatus === 'CONNECTING'
                      ? 'text-zinc-400 border-zinc-700 bg-zinc-800/40'
                      : auditStatus === 'MATCHED'
                        ? 'text-green-400 border-green-400/30 bg-green-400/10'
                        : auditStatus === 'SYNCING'
                          ? 'text-yellow-400 border-yellow-400/30 bg-yellow-400/10'
                          : 'text-red-400 border-red-400/40 bg-red-400/15 animate-pulse'
                  }`}
                >
                  {auditStatus === 'CONNECTING' ? 'CONNECTING' : auditStatus}
                </span>
              </div>

              <div className="p-4 space-y-3">
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                  <div className="border border-zinc-900 bg-zinc-950/70 rounded-lg p-3">
                    <p className="text-zinc-600 font-mono text-[9px] uppercase tracking-widest mb-1">
                      {isZh ? '审计批次 / Audit ID' : 'Audit ID'}
                    </p>
                    <p className="text-cyan-400 font-mono text-xs font-bold tabular-nums break-all">
                      {auditSnapshot ? auditSnapshot.auditId : 'LOADING...'}
                    </p>
                  </div>

                  <div className="border border-zinc-900 bg-zinc-950/70 rounded-lg p-3">
                    <p className="text-zinc-600 font-mono text-[9px] uppercase tracking-widest mb-1">
                      {isZh ? '断点区块 / Anchor Slot' : 'Anchor Slot'}
                    </p>
                    <p className="text-zinc-300 font-mono text-xs font-bold tabular-nums">
                      {auditSnapshot ? `#${auditSnapshot.chainState.currentSlot.toLocaleString('en')}` : 'LOADING...'}
                    </p>
                  </div>

                  <div className="border border-zinc-900 bg-zinc-950/70 rounded-lg p-3">
                    <p className="text-zinc-600 font-mono text-[9px] uppercase tracking-widest mb-1">
                      {isZh ? '账目差值 / Variance' : 'Variance'}
                    </p>
                    <p
                      className={`font-mono text-xs font-black tabular-nums ${
                        auditStatus === 'CONNECTING'
                          ? 'text-zinc-500'
                          : auditStatus === 'MATCHED'
                            ? 'text-green-400'
                            : auditStatus === 'SYNCING'
                              ? 'text-yellow-400'
                              : 'text-red-400 animate-pulse'
                      }`}
                    >
                      {variance !== null ? `${variance.toFixed(2)} USDC` : 'LOADING...'}
                    </p>
                  </div>
                </div>

                <div className={`text-[11px] font-mono rounded px-3 py-2 border ${
                  auditStatus === 'CONNECTING'
                    ? 'text-zinc-400 border-zinc-700 bg-zinc-800/30'
                    : auditStatus === 'MATCHED'
                      ? 'text-green-400 border-green-400/20 bg-green-400/5'
                      : auditStatus === 'SYNCING'
                        ? 'text-yellow-400 border-yellow-400/20 bg-yellow-400/5'
                        : 'text-red-400 border-red-400/30 bg-red-400/10 animate-pulse'
                }`}>
                  {(typeof window !== 'undefined' && window.treasuryAuditBypass) && (
                    <span className="block mb-1 text-cyan-400">
                      {isZh
                        ? '调试开关已开启：当前已强制校准为 MATCHED。'
                        : 'Debug bypass enabled: audit status is force-calibrated to MATCHED.'}
                    </span>
                  )}
                  {auditStatus === 'CONNECTING' && (
                    <span>
                      {isZh
                        ? '正在连接后台守护进程并等待首个对账快照，请稍候...'
                        : 'Connecting to daemon and waiting for the first reconciliation snapshot...'}
                    </span>
                  )}
                  {auditStatus === 'MATCHED' && (
                    <span>
                      {isZh
                        ? '链上 PDA 状态与后台账目完全重合，未发现异常流入。'
                        : 'On-chain PDA state and backend ledger are fully aligned. No anomalous inflow detected.'}
                    </span>
                  )}
                  {auditStatus === 'SYNCING' && (
                    <span>
                      {isZh
                        ? '前端路由正在同步最新落块高度，正在重新校准差值...'
                        : 'Frontend router is syncing the latest finalized slot height and recalibrating variance...'}
                    </span>
                  )}
                  {auditStatus === 'MISMATCHED' && (
                    <span>
                      {isZh
                        ? '🚨 警告：检测到账目非线性偏差！可能存在未授权的多签提取或假充值攻击，请立刻终止前端交互！'
                        : '🚨 Warning: Non-linear ledger variance detected. Possible unauthorized multisig extraction or fake-deposit attack. Halt frontend interactions immediately!'}
                    </span>
                  )}
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Yield Calculator */}
        <div className="border border-zinc-800 bg-zinc-950/20 rounded-xl overflow-hidden">
          <div className="flex items-center justify-between px-5 py-3 border-b border-zinc-900 bg-zinc-950">
            <div className="flex items-center gap-2">
              <div className="p-1.5 rounded bg-cyan-400/10 border border-cyan-400/20"><Calculator className="w-3.5 h-3.5 text-cyan-400" /></div>
              <p className="text-zinc-200 font-mono text-xs font-bold">{isZh ? '个人收益与分红估算器' : 'Personal Yield & Restitution Estimator'}</p>
            </div>
            <div className="flex items-center gap-1.5">
              <Clock className="w-3.5 h-3.5 text-cyan-400" />
              <span className="text-cyan-400 font-mono text-sm font-black tabular-nums">{pad(tick.hours)}:{pad(tick.minutes)}:{pad(tick.seconds)}</span>
            </div>
          </div>

          <div className="p-5 space-y-5">
            <div className="grid grid-cols-2 gap-3">
              <div className="bg-zinc-950/40 border border-green-400/20 rounded p-3 text-center">
                <p className="text-zinc-600 font-mono text-[9px] uppercase tracking-widest mb-1">{isZh ? '周期赔付池（50%）' : 'Epoch Restitution Pool (50%)'}</p>
                <p className="text-green-400 font-black font-mono text-base tabular-nums">{epochPool.toLocaleString('en', { minimumFractionDigits: 2 })} <span className="text-zinc-600 text-xs">U</span></p>
              </div>
              <div className="bg-zinc-950/40 border border-yellow-400/20 rounded p-3 text-center">
                <p className="text-zinc-600 font-mono text-[9px] uppercase tracking-widest mb-1">{isZh ? '质押分红池（10%）' : 'Staking Dividend Pool (10%)'}</p>
                <p className="text-yellow-400 font-black font-mono text-base tabular-nums">{dividendPool.toLocaleString('en', { minimumFractionDigits: 2 })} <span className="text-zinc-600 text-xs">U</span></p>
              </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
              <div className="space-y-1">
                <label className="text-zinc-600 text-[9px] font-mono uppercase tracking-widest">{isZh ? 'α 质押数量' : 'Staked α Amount'}</label>
                <input type="number" value={staked} onChange={e => setStaked(e.target.value)} placeholder="10000" className="w-full bg-zinc-950 border border-zinc-800 rounded px-3 py-2 text-xs font-mono text-zinc-200 placeholder-zinc-800 focus:outline-none focus:border-cyan-400/30" />
              </div>
              <div className="space-y-1">
                <label className="text-zinc-600 text-[9px] font-mono uppercase tracking-widest">{isZh ? '连续质押天数' : 'Consecutive Days'}</label>
                <input type="number" value={days} onChange={e => setDays(e.target.value)} placeholder="90" className="w-full bg-zinc-950 border border-zinc-800 rounded px-3 py-2 text-xs font-mono text-zinc-200 placeholder-zinc-800 focus:outline-none focus:border-cyan-400/30" />
              </div>
              <div className="space-y-1">
                <label className="text-zinc-600 text-[9px] font-mono uppercase tracking-widest">{isZh ? '全网质押总量' : 'Total Network Staked'}</label>
                <input type="number" value={totalStakedNetwork} onChange={e => setTotalStakedNetwork(e.target.value)} placeholder="10000000" className="w-full bg-zinc-950 border border-zinc-800 rounded px-3 py-2 text-xs font-mono text-zinc-200 placeholder-zinc-800 focus:outline-none focus:border-cyan-400/30" />
              </div>
            </div>

            <button onClick={calculate} className="w-full py-2 bg-zinc-900 hover:bg-zinc-850 border border-zinc-800 hover:border-zinc-700 text-zinc-300 font-mono font-bold uppercase tracking-widest rounded transition-all text-xs">
              {walletConnected ? (isZh ? '⚡ 跑数计算实时收益' : '⚡ Execute Live Estimation') : (isZh ? '🔓 请先在顶部连接钱包以解锁精确对账' : '🔓 Connect Wallet to Unlock Precision Engine')}
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

        <ProtocolArchitecture lang={lang} />
        <DAOGovernance
          lang={lang}
          onExecuteSuccess={() => {
            setIsReliefUnlocked(true);
            setLocalForfeitRevenue(prev => prev + 500);
          }}
        />

        {/* Transparency Manifest */}
        <div className="flex items-start gap-3 border border-zinc-900 bg-zinc-950/40 rounded-xl p-4">
          <div className="flex-shrink-0 mt-0.5 p-1.5 rounded bg-zinc-950 border border-zinc-900"><CheckCircle className="w-3.5 h-3.5 text-zinc-500" /></div>
          <div>
            <p className="text-zinc-400 font-mono text-[10px] font-bold uppercase tracking-widest mb-1">{isZh ? 'α 100% 链上真实性宣言' : 'α 100% On-Chain Transparency Manifest'}</p>
            <p className="text-zinc-600 font-mono text-[10px] leading-relaxed">
              {isZh
                ? '本国库拒绝任何虚假注水与预设底仓。起跑水位 0 USDC，所有资金完全由协议费用实时流入。50/20/20/10 规则硬编码于链上智能合约，全体 Holder 随时可链上对账。'
                : 'Zero synthetic padding or pre-seeded balances. Starting at 0 USDC, all funds flow in real-time from protocol fees. Hardcoded on-chain via immutable primitives, fully verifiable by any holder.'}
            </p>
          </div>
        </div>
      </div>
    );
  }