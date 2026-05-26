import { useState } from 'react';
import { AlertTriangle, Calculator, Zap, TrendingUp, TrendingDown, ShieldCheck, Activity, Layers } from 'lucide-react';
import { t, Lang } from '../translations';
import { TOTAL_NETWORK_POINTS, getDaysMultiplier } from '../utils/staking';

interface Props {
  lang: Lang;
  walletConnected: boolean;
}

const EPOCH_POOL_TOTAL = 5000;

const tiers = [
  { minPoints: 0,       multiplier: 1,  label: 'tier1' as const, color: 'text-gray-400',   border: 'border-gray-400/30',   bg: 'bg-gray-400/5',   barColor: 'bg-gray-400'   },
  { minPoints: 5000,    multiplier: 2,  label: 'tier2' as const, color: 'text-cyan-400',   border: 'border-cyan-400/30',   bg: 'bg-cyan-400/5',   barColor: 'bg-cyan-400'   },
  { minPoints: 25000,   multiplier: 4,  label: 'tier3' as const, color: 'text-yellow-400', border: 'border-yellow-400/30', bg: 'bg-yellow-400/5', barColor: 'bg-yellow-400' },
  { minPoints: 100000,  multiplier: 8,  label: 'tier4' as const, color: 'text-red-400',    border: 'border-red-400/30',   bg: 'bg-red-400/5',    barColor: 'bg-red-400'    },
];

const TIERS_DESC = [...tiers].reverse();

export default function VictimRelief({ lang, walletConnected }: Props) {
  const tr = t[lang];
  const [staked, setStaked] = useState('');
  const [days, setDays] = useState('');
  const [result, setResult] = useState<{ stakingPower: number; epochQuota: number; sharePct: string; tier: typeof tiers[number] } | null>(null);
  const [isPriceDropped, setIsPriceDropped] = useState(true);

  function calculate() {
    const s = parseFloat(staked) || 0;
    const d = parseFloat(days) || 0;
    const daysMultiplier = getDaysMultiplier(d);
    const stakingPower = s * daysMultiplier;
    const tier = TIERS_DESC.find((t) => stakingPower >= t.minPoints) ?? tiers[0];
    const epochQuota = (EPOCH_POOL_TOTAL * stakingPower) / TOTAL_NETWORK_POINTS;
    const sharePct = ((stakingPower / TOTAL_NETWORK_POINTS) * 100).toFixed(4);
    setResult({ stakingPower, epochQuota, sharePct, tier });
  }

  return (
    <div className="space-y-8">
      {/* Section header */}
      <div className="flex items-center gap-3">
        <Zap className="text-yellow-400 w-6 h-6" />
        <h2 className="text-2xl font-bold text-yellow-400 tracking-widest uppercase font-mono">
          {tr.reliefTitle}
        </h2>
      </div>

      {/* Eligibility notice — replaces Merkle gate */}
      <div className="flex items-start gap-3 border border-green-400/30 bg-green-400/5 rounded-xl p-4 relative overflow-hidden">
        <div className="absolute top-0 left-0 w-1 h-full bg-green-400/60 rounded-l-xl" />
        <div className="flex-shrink-0 mt-0.5 p-1.5 rounded-lg bg-green-400/10 border border-green-400/30">
          <ShieldCheck className="w-4 h-4 text-green-400" />
        </div>
        <div className="pl-1">
          <p className="text-green-400 font-mono text-xs font-bold uppercase tracking-widest mb-1">
            {lang === 'zh' ? '通用质押资格' : 'Universal Staking Eligibility'}
          </p>
          <p className="text-green-200/80 font-mono text-xs leading-relaxed">
            {lang === 'zh'
              ? '任何持有并质押 α 代币的地址均自动享有周期性赔付池的比例分配权。资格无需白名单或 Merkle 验证 — 质押即参与。'
              : 'Any address staking α tokens is automatically eligible for a proportional share of each Epoch Restitution Pool. No whitelist or Merkle proof required — stake and you\'re in.'}
          </p>
        </div>
      </div>

      {/* Price Protection */}
      <div className="border border-gray-700/60 bg-gray-900/50 rounded-xl overflow-hidden backdrop-blur-sm">
        <div className="flex items-center justify-between px-5 py-3 border-b border-gray-700/50 bg-gray-900/70">
          <div className="flex items-center gap-2">
            <Activity className="w-4 h-4 text-gray-400" />
            <span className="text-gray-400 font-mono text-xs font-bold uppercase tracking-widest">{tr.marketStatus}</span>
          </div>
          <button
            onClick={() => setIsPriceDropped((v) => !v)}
            className={`flex items-center gap-2 text-xs font-mono font-bold px-3 py-1.5 rounded-lg border transition-all duration-200 ${
              isPriceDropped
                ? 'border-red-400/50 text-red-400 bg-red-400/10 hover:bg-red-400/15'
                : 'border-green-400/40 text-green-400 bg-green-400/8 hover:bg-green-400/15'
            }`}
          >
            {isPriceDropped ? (
              <><TrendingDown className="w-3 h-3" />{tr.priceDropToggleOff}</>
            ) : (
              <><TrendingUp className="w-3 h-3" />{tr.priceDropToggleOn}</>
            )}
          </button>
        </div>
        <div className="p-5">
          {isPriceDropped ? (
            <div
              className="relative border border-red-400/50 rounded-xl p-4 overflow-hidden"
              style={{ background: 'linear-gradient(135deg, rgba(127,29,29,0.25) 0%, rgba(120,53,15,0.20) 50%, rgba(127,29,29,0.15) 100%)' }}
            >
              <div className="absolute inset-0 rounded-xl pointer-events-none" style={{ boxShadow: '0 0 20px rgba(248,113,113,0.15), inset 0 0 20px rgba(248,113,113,0.05)' }} />
              <div className="absolute top-0 left-0 w-full h-0.5 bg-gradient-to-r from-red-500 via-orange-400 to-red-500" />
              <div className="flex items-start gap-3">
                <div className="flex-shrink-0 mt-0.5 p-1.5 rounded-lg bg-red-400/20 border border-red-400/40">
                  <TrendingDown className="w-4 h-4 text-red-400 animate-pulse" />
                </div>
                <div className="flex-1 space-y-2">
                  <p className="text-red-200 font-mono text-sm leading-relaxed font-bold">{tr.priceDropAlert}</p>
                  <div className="flex flex-wrap gap-2 pt-1">
                    <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded border border-red-400/50 bg-red-400/15 text-red-300 text-xs font-mono font-bold animate-pulse">
                      <Zap className="w-3 h-3" />{tr.buybackActive}
                    </span>
                    <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded border border-orange-400/50 bg-orange-400/15 text-orange-300 text-xs font-mono font-bold animate-pulse">
                      <ShieldCheck className="w-3 h-3" />{tr.stakingBoost}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          ) : (
            <div className="flex items-center gap-3 py-2">
              <div className="p-2 rounded-lg bg-green-400/10 border border-green-400/30">
                <ShieldCheck className="w-5 h-5 text-green-400" />
              </div>
              <div>
                <p className="text-green-400 font-mono font-bold text-sm">{tr.priceNormal}</p>
                <p className="text-gray-600 font-mono text-xs mt-0.5">{tr.priceStableNote}</p>
              </div>
              <span className="ml-auto text-xs font-mono px-2 py-1 rounded border border-green-400/30 text-green-400 bg-green-400/10">
                NOMINAL
              </span>
            </div>
          )}
        </div>
      </div>

      {/* Staking multiplier tiers */}
      <div className="border border-gray-700/50 rounded-xl overflow-hidden bg-gray-900/40 backdrop-blur-sm">
        <div className="px-4 py-3 border-b border-gray-700/50 bg-gray-800/60">
          <p className="text-gray-400 font-mono text-xs font-bold uppercase tracking-widest">
            {lang === 'zh' ? '连续质押天数加成倍率' : 'Consecutive Staking Days Multiplier Tiers'}
          </p>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-sm font-mono">
            <thead>
              <tr className="border-b border-gray-700/50 bg-gray-800/40">
                {[
                  lang === 'zh' ? '档位' : tr.poolLabel,
                  lang === 'zh' ? '最低质押算力' : 'Min Staking Power',
                  lang === 'zh' ? '天数门槛' : 'Days Threshold',
                  tr.tierMultiplier,
                ].map((col) => (
                  <th key={col} className="text-left px-4 py-3 text-gray-400 uppercase tracking-widest text-xs font-bold">
                    {col}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {[
                { tier: tiers[0], daysThreshold: '< 30d',   daysMultiplier: '1x' },
                { tier: tiers[1], daysThreshold: '≥ 30d',   daysMultiplier: '2x' },
                { tier: tiers[2], daysThreshold: '≥ 90d',   daysMultiplier: '4x' },
                { tier: tiers[3], daysThreshold: '≥ 180d',  daysMultiplier: '8x' },
              ].map(({ tier, daysThreshold, daysMultiplier }, i) => {
                const isActive = result?.tier.label === tier.label;
                return (
                  <tr key={i} className={`border-b border-gray-800/40 transition-colors duration-150 ${isActive ? tier.bg : 'hover:bg-gray-800/30'}`}>
                    <td className={`px-4 py-3 font-bold ${tier.color}`}>{tr[tier.label]}</td>
                    <td className="px-4 py-3 text-gray-300">{tier.minPoints.toLocaleString()}</td>
                    <td className="px-4 py-3 text-gray-400">{daysThreshold}</td>
                    <td className="px-4 py-3">
                      <span className={`px-2 py-0.5 rounded border text-xs ${tier.color} ${tier.border} ${tier.bg}`}>
                        {daysMultiplier}
                      </span>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
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

      {/* Calculator */}
      <div className="border border-yellow-400/20 bg-yellow-400/5 rounded-xl p-6 space-y-5">
        <div className="flex items-center gap-2">
          <Calculator className="text-yellow-400 w-5 h-5" />
          <h3 className="text-yellow-400 font-mono font-bold uppercase tracking-wider">
            {lang === 'zh' ? '质押奖励估算器' : 'Staking Rewards Estimator'}
          </h3>
        </div>

        {/* Formula */}
        <div className="bg-gray-900/70 border border-zinc-700/50 rounded-lg px-4 py-3 space-y-1.5">
          <p className="text-zinc-500 font-mono text-[10px] uppercase tracking-widest">
            {lang === 'zh' ? '透明公式' : 'Formula'}
          </p>
          <div className="flex flex-wrap items-center gap-x-2 gap-y-1 font-mono text-sm">
            <span className="text-cyan-400 font-bold">{lang === 'zh' ? '质押算力' : 'Staking Power'}</span>
            <span className="text-zinc-600">=</span>
            <span className="text-green-400 font-bold">{lang === 'zh' ? '质押数量' : 'Staked α'}</span>
            <span className="text-zinc-600">×</span>
            <span className="text-yellow-400 font-bold">{lang === 'zh' ? '天数倍率' : 'Days Multiplier'}</span>
          </div>
          <div className="flex flex-wrap items-center gap-x-2 gap-y-1 font-mono text-sm">
            <span className="text-cyan-400 font-bold">{lang === 'zh' ? '周期配额' : 'Epoch Quota'}</span>
            <span className="text-zinc-600">=</span>
            <span className="text-yellow-400 font-bold">{lang === 'zh' ? '本期总池' : 'Epoch Pool'}</span>
            <span className="text-zinc-600">×</span>
            <span className="text-zinc-400">(</span>
            <span className="text-cyan-400 font-bold">{lang === 'zh' ? '质押算力' : 'Staking Power'}</span>
            <span className="text-zinc-600">/</span>
            <span className="text-zinc-300 font-bold">{lang === 'zh' ? '全网总算力' : 'Total Network Power'}</span>
            <span className="text-zinc-400">)</span>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="space-y-1.5">
            <label className="text-gray-500 text-xs font-mono uppercase tracking-wider">{tr.stakedAmt}</label>
            <input
              type="number"
              value={staked}
              onChange={(e) => setStaked(e.target.value)}
              placeholder={tr.stakedPlaceholder}
              className="w-full bg-gray-900/80 border border-yellow-400/20 rounded-lg px-3 py-2.5 text-sm font-mono text-gray-300 placeholder-gray-600 focus:outline-none focus:border-yellow-400/60 transition-colors"
            />
          </div>
          <div className="space-y-1.5">
            <label className="text-gray-500 text-xs font-mono uppercase tracking-wider">{tr.stakeDays}</label>
            <input
              type="number"
              value={days}
              onChange={(e) => setDays(e.target.value)}
              placeholder={tr.daysPlaceholder}
              className="w-full bg-gray-900/80 border border-yellow-400/20 rounded-lg px-3 py-2.5 text-sm font-mono text-gray-300 placeholder-gray-600 focus:outline-none focus:border-yellow-400/60 transition-colors"
            />
          </div>
        </div>

        <button
          onClick={calculate}
          className="w-full py-3 bg-yellow-500/20 hover:bg-yellow-500/30 border border-yellow-500/50 text-yellow-400 font-mono font-bold uppercase tracking-wider rounded-lg transition-all duration-200 hover:shadow-yellow-500/20 hover:shadow-lg text-sm"
        >
          {tr.calcPoints}
        </button>

        {result && (
          <div className="space-y-4 pt-1">
            {/* Staking power + tier */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="bg-gray-900/80 border border-gray-700/50 rounded-lg p-4 text-center">
                <p className="text-gray-500 text-xs font-mono uppercase tracking-wider mb-1">
                  {lang === 'zh' ? '质押算力' : 'Staking Power'}
                </p>
                <p className="text-2xl font-bold font-mono text-cyan-400">
                  {Math.round(result.stakingPower).toLocaleString()}
                </p>
                <p className={`text-xs font-mono mt-1 ${result.tier.color}`}>{tr[result.tier.label]}</p>
              </div>

              <div className="bg-gray-900/80 border border-gray-700/50 rounded-lg p-4 text-center">
                <p className="text-gray-500 text-xs font-mono uppercase tracking-wider mb-1">
                  {lang === 'zh' ? '全网占比' : 'Network Share'}
                </p>
                <p className="text-2xl font-bold font-mono text-green-400">
                  {result.sharePct}%
                </p>
                <p className="text-zinc-600 font-mono text-[10px] mt-1">
                  {lang === 'zh' ? '占全网总算力' : 'of total network power'}
                </p>
              </div>

              <div className="bg-gray-900/80 border border-gray-700/50 rounded-lg p-4 text-center">
                <p className="text-gray-500 text-xs font-mono uppercase tracking-wider mb-1">
                  {lang === 'zh' ? '本期可领配额' : 'Epoch Claimable Quota'}
                </p>
                <p className="text-2xl font-bold font-mono text-yellow-400">
                  {result.epochQuota.toFixed(4)} <span className="text-sm text-zinc-500">USDC</span>
                </p>
              </div>
            </div>

            {/* Epoch cap proof */}
            <div className="flex items-start gap-3 border border-zinc-700/50 bg-zinc-900/60 rounded-lg p-4">
              <Layers className="w-4 h-4 text-zinc-500 mt-0.5 flex-shrink-0" />
              <div className="space-y-1">
                <p className="text-zinc-400 font-mono text-[11px] font-bold uppercase tracking-widest">
                  {lang === 'zh' ? '周期配额上限验证' : 'Epoch Cap Enforcement'}
                </p>
                <p className="text-zinc-500 font-mono text-[10px] leading-relaxed">
                  {lang === 'zh'
                    ? `本期总池 ${EPOCH_POOL_TOTAL.toLocaleString()} USDC × (${Math.round(result.stakingPower).toLocaleString()} / ${TOTAL_NETWORK_POINTS.toLocaleString()}) = ${result.epochQuota.toFixed(4)} USDC 硬顶。智能合约层面强制执行，任何地址均无法超额提取，金库永不被单点耗尽。`
                    : `Pool ${EPOCH_POOL_TOTAL.toLocaleString()} USDC × (${Math.round(result.stakingPower).toLocaleString()} / ${TOTAL_NETWORK_POINTS.toLocaleString()}) = ${result.epochQuota.toFixed(4)} USDC hard cap. Enforced at contract level — no single address can overdraw, vault drain is structurally impossible.`}
                </p>
              </div>
            </div>

            {/* Tax notice */}
            <div className="border border-yellow-400/30 bg-yellow-400/5 rounded-lg px-4 py-3">
              <p className="text-yellow-300 font-mono text-xs leading-relaxed">{tr.taxWarning}</p>
            </div>
          </div>
        )}
      </div>

      {/* Submit claim */}
      <div className="border border-green-400/20 bg-green-400/5 rounded-xl p-6 space-y-4">
        <div className="flex items-center gap-2">
          <TrendingUp className="w-4 h-4 text-green-400" />
          <h3 className="text-green-400 font-mono font-bold uppercase tracking-wider text-sm">{tr.submitClaim}</h3>
        </div>
        {!walletConnected && (
          <p className="text-zinc-500 font-mono text-xs border border-zinc-700/50 rounded-lg px-3 py-2">
            {lang === 'zh' ? '⚠ 请先连接钱包以自动填充您的链上质押数据。' : '⚠ Connect your wallet to auto-populate your on-chain staking data.'}
          </p>
        )}
        <div className="space-y-3">
          <input
            type="text"
            placeholder={tr.walletPlaceholder}
            className="w-full bg-gray-900/80 border border-green-400/20 rounded-lg px-3 py-2.5 text-sm font-mono text-gray-300 placeholder-gray-600 focus:outline-none focus:border-green-400/60 transition-colors"
          />
          <textarea
            placeholder={tr.lossDocPlaceholder}
            rows={3}
            className="w-full bg-gray-900/80 border border-green-400/20 rounded-lg px-3 py-2.5 text-sm font-mono text-gray-300 placeholder-gray-600 focus:outline-none focus:border-green-400/60 transition-colors resize-none"
          />
        </div>
        <button className="w-full py-3 bg-green-500/20 hover:bg-green-500/30 border border-green-500/50 text-green-400 font-mono font-bold uppercase tracking-wider rounded-lg transition-all duration-200 hover:shadow-green-500/20 hover:shadow-lg text-sm">
          {tr.submitClaim}
        </button>
      </div>
    </div>
  );
}
