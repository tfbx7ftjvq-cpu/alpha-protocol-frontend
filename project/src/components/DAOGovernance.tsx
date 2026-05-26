import { useEffect, useMemo, useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import {
  Shield,
  Users,
  Clock,
  Vote,
  Zap,
  Lock,
  CheckCircle,
  XCircle,
  RefreshCw,
  SlidersHorizontal,
  Wallet,
  PlusCircle,
} from 'lucide-react';
import { Lang } from '../translations';

interface Props {
  lang: Lang;
}

const BASE_YES_VOTES = 4500;
const BASE_NO_VOTES = 1200;

function parsePositive(raw: string) {
  const n = parseFloat(raw);
  return Number.isFinite(n) && n > 0 ? n : 0;
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

function voteButtonClass(connected: boolean, active: boolean, side: 'yes' | 'no') {
  if (!connected) {
    return 'border-zinc-700/50 bg-zinc-900/50 text-zinc-500 cursor-not-allowed opacity-50';
  }
  if (active) {
    return side === 'yes'
      ? 'border-green-400/60 bg-green-400/20 text-green-400'
      : 'border-red-400/60 bg-red-400/20 text-red-400';
  }
  return side === 'yes'
    ? 'border-green-500/40 bg-green-500/10 text-green-400 hover:bg-green-500/20 hover:border-green-500/60 hover:shadow-green-500/10 hover:shadow-lg'
    : 'border-red-500/40 bg-red-500/10 text-red-400 hover:bg-red-500/20 hover:border-red-500/60 hover:shadow-red-500/10 hover:shadow-lg';
}

// ─── Quadratic Voting Panel (DAO 003) ─────────────────────────────────────────
function QuadraticVotingPanel({
  lang,
  connected,
  connectHint,
  voteAmount,
  setVoteAmount,
}: {
  lang: Lang;
  connected: boolean;
  connectHint: string;
  voteAmount: string;
  setVoteAmount: (v: string) => void;
}) {
  const zh = lang === 'zh';
  const [yesVotes, setYesVotes] = useState(BASE_YES_VOTES);
  const [noVotes, setNoVotes] = useState(BASE_NO_VOTES);
  const [lastVote, setLastVote] = useState<'yes' | 'no' | null>(null);

  const stake = useMemo(() => parsePositive(voteAmount), [voteAmount]);
  const effectiveVotes = useMemo(() => (stake > 0 ? Math.sqrt(stake) : 0), [stake]);

  const totalVotes = yesVotes + noVotes;
  const yesPct = totalVotes > 0 ? (yesVotes / totalVotes) * 100 : 0;
  const noPct = totalVotes > 0 ? (noVotes / totalVotes) * 100 : 0;

  const yesBump = useBump(yesVotes);
  const noBump = useBump(noVotes);
  const evBump = useBump(effectiveVotes);

  function castVote(side: 'yes' | 'no') {
    if (!connected || effectiveVotes <= 0) return;
    setLastVote(side);
    if (side === 'yes') setYesVotes((v) => v + effectiveVotes);
    else setNoVotes((v) => v + effectiveVotes);
  }

  return (
    <div className="border border-green-400/25 bg-green-400/5 rounded-xl overflow-hidden">
      <div className="px-5 py-4 border-b border-green-400/20 bg-zinc-950/60 flex flex-wrap items-center gap-3">
        <div className="flex items-center gap-2 min-w-0">
          <Vote className="w-4 h-4 text-green-400 flex-shrink-0" />
          <div>
            <p className="text-green-400 font-mono text-xs font-bold uppercase tracking-widest">
              {zh ? 'DAO 003 号提案 · 二次方投票' : 'DAO Proposal #003 · Quadratic Voting'}
            </p>
            <p className="text-zinc-400 font-mono text-[11px] mt-0.5 truncate">
              {zh
                ? '针对 X 项目受害者实施 150,000 USDC 救济分发'
                : '150,000 USDC relief distribution for X Protocol victims'}
            </p>
          </div>
        </div>
        <span className="ml-auto text-[10px] font-mono font-bold text-cyan-400 border border-cyan-400/30 px-2 py-0.5 rounded">
          {zh ? '计票中' : 'ACTIVE'}
        </span>
      </div>

      <div className="p-5 sm:p-6 space-y-5">
        <p className="text-zinc-400 font-mono text-xs leading-relaxed">
          {zh
            ? '本提案已由 DAO 陪审团表决通过事件定性，现进入链上二次方投票阶段。请投入 α 代币权重参与赞成或反对。'
            : 'Event qualification passed by jury. On-chain quadratic voting is live — stake α weight to vote FOR or AGAINST.'}
        </p>

        <div className="border border-cyan-400/30 bg-cyan-400/5 rounded-lg px-4 py-3">
          <p className="text-cyan-400 font-mono text-xs leading-relaxed">
            🛡️{' '}
            {zh
              ? '治理防线：本自治组织采用二次方投票算法（Votes = √Tokens）。大户资金具备严重边际递减效应，捍卫散户治理主权。'
              : 'Governance defense: quadratic voting (Votes = √Tokens). Whale capital faces severe diminishing returns — retail sovereignty protected.'}
          </p>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div className="space-y-2 min-w-0">
            <label className="text-zinc-500 text-xs font-mono uppercase tracking-wider">
              {zh ? '计划投出 α 数量' : 'α Tokens to Vote'}
            </label>
            <input
              type="number"
              min={0}
              step="any"
              value={voteAmount}
              onChange={(e) => setVoteAmount(e.target.value)}
              disabled={!connected}
              placeholder={zh ? '输入代币数量' : 'Enter token amount'}
              className="w-full bg-zinc-950 border border-green-400/25 rounded-lg px-3 py-2.5 text-sm font-mono text-zinc-200 focus:outline-none focus:border-green-400/60 transition-colors disabled:opacity-40"
            />
          </div>
          <div className="bg-zinc-950/80 border border-zinc-700/50 rounded-lg p-4 text-center min-w-0">
            <p className="text-zinc-500 text-[10px] font-mono uppercase tracking-wider mb-1">
              {zh ? '有效票数' : 'Effective Votes'}
            </p>
            <p
              className={`text-2xl sm:text-3xl font-bold font-mono text-cyan-400 tabular-nums transition-transform duration-150 ${
                evBump ? 'scale-[1.03]' : 'scale-100'
              }`}
            >
              {effectiveVotes.toFixed(4)}
            </p>
            <p className="text-zinc-600 font-mono text-[10px] mt-1">√(voteAmount)</p>
          </div>
        </div>

        {/* Live vote bars */}
        <div className="space-y-4">
          <div className="space-y-2">
            <div className="flex items-center justify-between font-mono text-xs">
              <span className="text-green-400 font-bold flex items-center gap-1">
                <CheckCircle className="w-3 h-3" />
                {zh ? '赞成票' : 'YES'}
              </span>
              <span
                className={`text-green-400 font-bold tabular-nums transition-transform duration-150 ${
                  yesBump ? 'scale-[1.03]' : ''
                }`}
              >
                {yesVotes.toLocaleString('en-US', { maximumFractionDigits: 2 })} {zh ? '票' : 'votes'}
              </span>
            </div>
            <div className="relative h-4 bg-zinc-800 rounded-full overflow-hidden border border-zinc-700/50">
              <div
                className="h-full rounded-full bg-gradient-to-r from-green-800 to-green-400 transition-all duration-500 relative overflow-hidden"
                style={{ width: `${yesPct}%` }}
              >
                <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent animate-shimmer" />
              </div>
            </div>
            <p className="text-[10px] font-mono text-zinc-600 text-right tabular-nums">{yesPct.toFixed(2)}%</p>
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between font-mono text-xs">
              <span className="text-red-400 font-bold flex items-center gap-1">
                <XCircle className="w-3 h-3" />
                {zh ? '反对票' : 'NO'}
              </span>
              <span
                className={`text-red-400 font-bold tabular-nums transition-transform duration-150 ${
                  noBump ? 'scale-[1.03]' : ''
                }`}
              >
                {noVotes.toLocaleString('en-US', { maximumFractionDigits: 2 })} {zh ? '票' : 'votes'}
              </span>
            </div>
            <div className="relative h-4 bg-zinc-800 rounded-full overflow-hidden border border-zinc-700/50">
              <div
                className="h-full rounded-full bg-gradient-to-r from-red-900 to-red-400 transition-all duration-500"
                style={{ width: `${noPct}%` }}
              />
            </div>
            <p className="text-[10px] font-mono text-zinc-600 text-right tabular-nums">{noPct.toFixed(2)}%</p>
          </div>
        </div>

        <div className="flex items-center gap-2 text-[10px] font-mono text-zinc-600">
          <Clock className="w-3 h-3" />
          {zh ? '剩余 32 小时 10 分钟 · 链上实时计票' : '32h 10m remaining · on-chain tally LIVE'}
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <button
            type="button"
            onClick={() => castVote('yes')}
            disabled={!connected || effectiveVotes <= 0}
            className={`flex items-center justify-center gap-2 px-4 py-3 rounded-lg border text-xs font-mono font-bold tracking-widest uppercase transition-all duration-200 ${voteButtonClass(
              connected,
              lastVote === 'yes',
              'yes'
            )}`}
          >
            <CheckCircle className="w-3.5 h-3.5" />
            {!connected ? connectHint : zh ? '投赞成票' : 'Vote YES'}
          </button>
          <button
            type="button"
            onClick={() => castVote('no')}
            disabled={!connected || effectiveVotes <= 0}
            className={`flex items-center justify-center gap-2 px-4 py-3 rounded-lg border text-xs font-mono font-bold tracking-widest uppercase transition-all duration-200 ${voteButtonClass(
              connected,
              lastVote === 'no',
              'no'
            )}`}
          >
            <XCircle className="w-3.5 h-3.5" />
            {!connected ? connectHint : zh ? '投反对票' : 'Vote NO'}
          </button>
        </div>

        <button
          type="button"
          disabled={!connected}
          className={`w-full flex items-center justify-center gap-2 py-3 rounded-lg border text-xs font-mono font-bold uppercase tracking-wider transition-all duration-200 ${
            connected
              ? 'border-green-500/50 bg-green-500/10 text-green-400 hover:bg-green-500/20 hover:border-green-500/60'
              : 'border-zinc-700/50 bg-zinc-900/50 text-zinc-500 cursor-not-allowed opacity-50'
          }`}
        >
          <PlusCircle className="w-4 h-4" />
          {!connected ? connectHint : zh ? '发起新提案' : 'Submit New Proposal'}
        </button>
      </div>
    </div>
  );
}

// ─── Payroll slot registry ────────────────────────────────────────────────────
interface PayrollSlot {
  id: number;
  roleEn: string;
  roleZh: string;
  tagEn: string;
  tagZh: string;
  poolPct: number;
  revenuePct: number;
  address: string;
  status: 'active' | 'impeachment_pending' | 'dismissed';
}

const INITIAL_SLOTS: PayrollSlot[] = [
  { id: 1, roleEn: 'Core Technical Infrastructure', roleZh: '核心技术流', tagEn: 'Core Dev', tagZh: '核心开发', poolPct: 40, revenuePct: 8.0, address: 'AnLi...mF9k', status: 'impeachment_pending' },
  { id: 2, roleEn: 'Global Growth & Community Operations', roleZh: '全球宣发流', tagEn: 'Global Ops', tagZh: '全球运营', poolPct: 30, revenuePct: 6.0, address: 'Ops7...XyZ2', status: 'active' },
  { id: 3, roleEn: 'Jury Pool', roleZh: '陪审团池', tagEn: 'Jury Pool', tagZh: '陪审团', poolPct: 20, revenuePct: 4.0, address: 'Jury...K3p1', status: 'active' },
  { id: 4, roleEn: 'Protocol Reserve', roleZh: '协议储备金', tagEn: 'Reserve', tagZh: '储备金', poolPct: 10, revenuePct: 2.0, address: 'Res9...7Wq0', status: 'active' },
];

const PAYROLL_LEGEND = [
  { label: { en: 'Core Dev 40%', zh: '核心开发 40%' }, dot: 'bg-cyan-500' },
  { label: { en: 'Global Ops 30%', zh: '全球运营 30%' }, dot: 'bg-green-500' },
  { label: { en: 'Jury Pool 20%', zh: '陪审团池 20%' }, dot: 'bg-yellow-500' },
  { label: { en: 'Reserve 10%', zh: '储备金 10%' }, dot: 'bg-zinc-500' },
];

function VoteBar({
  label,
  value,
  threshold,
  passed,
  barClass,
  labelClass,
}: {
  label: string;
  value: number;
  threshold?: number;
  passed?: boolean;
  barClass: string;
  labelClass: string;
}) {
  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between font-mono text-[11px]">
        <span className={`font-bold ${passed ? 'text-green-400' : labelClass}`}>{label}</span>
        <span className={`font-bold tabular-nums ${passed ? 'text-green-400' : labelClass}`}>{value.toFixed(1)}%</span>
      </div>
      <div className={`relative bg-zinc-800 rounded-full overflow-hidden border border-zinc-700/50 ${threshold !== undefined ? 'h-4' : 'h-2.5'}`}>
        {threshold !== undefined && (
          <div className="absolute top-0 bottom-0 w-0.5 bg-green-400/60 z-10" style={{ left: `${threshold}%` }} />
        )}
        <div
          className={`h-full rounded-full transition-all duration-500 relative overflow-hidden ${barClass}`}
          style={{ width: `${Math.min(value, 100)}%` }}
        >
          {threshold !== undefined && (
            <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent animate-shimmer" />
          )}
        </div>
      </div>
    </div>
  );
}

function RoleElectionDashboard({
  lang,
  connected,
  connectHint,
  voteAmount,
}: {
  lang: Lang;
  connected: boolean;
  connectHint: string;
  voteAmount: string;
}) {
  const zh = lang === 'zh';
  const [slots, setSlots] = useState<PayrollSlot[]>(INITIAL_SLOTS);
  const [yesVotes, setYesVotes] = useState(42.5);
  const [noVotes, setNoVotes] = useState(15.0);
  const [p004Voted, setP004Voted] = useState<'for' | 'against' | null>(null);
  const [p004Executed, setP004Executed] = useState(false);

  const effectiveVotes = useMemo(() => {
    const s = parsePositive(voteAmount);
    return s > 0 ? Math.sqrt(s) : 0;
  }, [voteAmount]);

  const THRESHOLD = 51.0;
  const p004Passed = yesVotes >= THRESHOLD;

  function vote(side: 'for' | 'against') {
    if (!connected || effectiveVotes <= 0 || p004Voted) return;
    setP004Voted(side);
    const delta = effectiveVotes * 0.4;
    if (side === 'for') setYesVotes((v) => parseFloat(Math.min(v + delta, 100).toFixed(1)));
    else setNoVotes((v) => parseFloat(Math.min(v + delta, 100).toFixed(1)));
  }

  function executeProposal() {
    if (!connected || !p004Passed || p004Executed) return;
    setP004Executed(true);
    setSlots((prev) => prev.map((s) => (s.id === 1 ? { ...s, address: 'NewD...8v7x', status: 'active' } : s)));
  }

  const visibleSlots = slots.filter((s) => s.status !== 'dismissed');

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-2 border-b border-zinc-800 pb-2">
        <RefreshCw className="w-4 h-4 text-zinc-500" />
        <h4 className="text-sm font-bold text-zinc-100 font-mono">
          {zh ? '3. 动态基础设施角色选举与弹劾看板' : '3. Dynamic Infrastructure Role Election & Impeachment'}
        </h4>
      </div>

      <div className="border border-zinc-700/50 bg-zinc-900/60 rounded-xl p-4 space-y-3">
        <p className="text-zinc-500 font-mono text-[10px] uppercase tracking-widest">
          {zh ? '贡献者薪资池分配（占协议总营收 20%）' : 'Contributor Payroll Pool — 20% of Protocol Revenue'}
        </p>
        <div className="flex h-3 rounded-full overflow-hidden gap-0.5">
          <div className="bg-cyan-500/80" style={{ width: '40%' }} />
          <div className="bg-green-500/80" style={{ width: '30%' }} />
          <div className="bg-yellow-500/80" style={{ width: '20%' }} />
          <div className="bg-zinc-500/80" style={{ width: '10%' }} />
        </div>
        <div className="flex flex-wrap gap-x-5 gap-y-1">
          {PAYROLL_LEGEND.map(({ label, dot }) => (
            <div key={label.en} className="flex items-center gap-1.5">
              <span className={`w-2 h-2 rounded-full ${dot}`} />
              <span className="text-[10px] font-mono text-zinc-400">{zh ? label.zh : label.en}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="border border-zinc-700/50 rounded-xl overflow-hidden bg-zinc-900/40">
        <div className="px-4 py-3 border-b border-zinc-800 bg-zinc-900/70 flex items-center gap-2">
          <Shield className="w-3.5 h-3.5 text-zinc-500" />
          <span className="text-zinc-300 font-mono text-xs font-bold uppercase tracking-widest">
            {zh ? '当前活跃地址登记册' : 'Current Active Registry'}
          </span>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-xs font-mono">
            <thead>
              <tr className="border-b border-zinc-800 bg-zinc-900/50">
                {[zh ? '槽位' : 'Slot', zh ? '职能' : 'Role', zh ? '池占比' : 'Pool', zh ? '地址' : 'Address', zh ? '状态' : 'Status'].map((col) => (
                  <th key={col} className="text-left px-4 py-2.5 text-zinc-500 uppercase tracking-widest text-[10px] font-bold whitespace-nowrap">
                    {col}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {visibleSlots.map((slot) => (
                <tr key={slot.id} className="border-b border-zinc-800/50 hover:bg-zinc-800/30">
                  <td className="px-4 py-3 text-zinc-500">#{slot.id}</td>
                  <td className="px-4 py-3 text-zinc-200 font-bold">{zh ? slot.roleZh : slot.roleEn}</td>
                  <td className="px-4 py-3 text-cyan-400 font-bold">{slot.poolPct}%</td>
                  <td className="px-4 py-3 text-zinc-300">{slot.address}</td>
                  <td className="px-4 py-3">
                    <span
                      className={`text-[10px] px-2 py-0.5 rounded border ${
                        slot.status === 'impeachment_pending'
                          ? 'border-red-400/40 bg-red-400/10 text-red-400'
                          : 'border-green-400/30 bg-green-400/5 text-green-400'
                      }`}
                    >
                      {slot.status === 'impeachment_pending' ? (zh ? '弹劾待决' : 'PENDING') : zh ? '活跃' : 'ACTIVE'}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      <div className="border border-cyan-400/25 bg-cyan-400/5 rounded-xl p-5 space-y-4">
        <p className="text-zinc-100 font-mono text-sm font-bold">
          {zh ? '[提案 #004 - 替换核心技术基础设施负责人]' : '[Proposal #004 - Replace Core Infrastructure Lead]'}
        </p>
        <VoteBar
          label={zh ? '赞成' : 'YES'}
          value={yesVotes}
          threshold={THRESHOLD}
          passed={p004Passed}
          barClass={p004Passed ? 'bg-gradient-to-r from-green-700 to-green-400' : 'bg-gradient-to-r from-zinc-600 to-zinc-400'}
          labelClass="text-zinc-300"
        />
        <VoteBar label={zh ? '反对' : 'NO'} value={noVotes} barClass="bg-gradient-to-r from-zinc-700 to-zinc-500" labelClass="text-zinc-500" />

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <button
            type="button"
            onClick={() => vote('for')}
            disabled={!connected || p004Voted !== null || effectiveVotes <= 0}
            className={`flex items-center justify-center gap-2 px-4 py-3 rounded-lg border text-xs font-mono font-bold uppercase transition-all duration-200 ${voteButtonClass(
              connected,
              p004Voted === 'for',
              'yes'
            )}`}
          >
            <CheckCircle className="w-3.5 h-3.5" />
            {!connected ? connectHint : zh ? '投赞成票' : 'Vote YES'}
          </button>
          <button
            type="button"
            onClick={() => vote('against')}
            disabled={!connected || p004Voted !== null || effectiveVotes <= 0}
            className={`flex items-center justify-center gap-2 px-4 py-3 rounded-lg border text-xs font-mono font-bold uppercase transition-all duration-200 ${voteButtonClass(
              connected,
              p004Voted === 'against',
              'no'
            )}`}
          >
            <XCircle className="w-3.5 h-3.5" />
            {!connected ? connectHint : zh ? '投反对票' : 'Vote NO'}
          </button>
        </div>

        {p004Passed && !p004Executed && (
          <button
            type="button"
            onClick={executeProposal}
            disabled={!connected}
            className={`w-full py-2.5 rounded-lg border text-xs font-mono font-bold uppercase tracking-widest transition-all duration-200 ${
              connected
                ? 'border-green-400/50 bg-green-400/15 text-green-400 hover:bg-green-400/25'
                : 'border-zinc-700/50 text-zinc-500 cursor-not-allowed opacity-50'
            }`}
          >
            {connected ? (zh ? '确认执行地址覆写' : 'Confirm Address Overwrite') : connectHint}
          </button>
        )}
      </div>
    </div>
  );
}

const GENESIS_SPLITS = [
  { pct: 50, labelZh: '散户赔付救济金池', labelEn: 'Retail Relief Pool', color: 'text-green-400', track: 'bg-green-400', border: 'border-green-400/30' },
  { pct: 20, labelZh: '自动回购销毁矩阵', labelEn: 'Buyback & Burn Matrix', color: 'text-red-400', track: 'bg-red-400', border: 'border-red-400/30' },
  { pct: 20, labelZh: 'DAO 建设者工资池', labelEn: 'DAO Contributor Payroll', color: 'text-blue-400', track: 'bg-blue-400', border: 'border-blue-400/30' },
  { pct: 10, labelZh: '纯代币质押分红池', labelEn: 'Pure Staking Dividend', color: 'text-yellow-400', track: 'bg-yellow-400', border: 'border-yellow-400/30' },
];

function GenesisParameterConsole({ lang }: { lang: Lang }) {
  const zh = lang === 'zh';
  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2 border-b border-zinc-800 pb-2">
        <SlidersHorizontal className="w-4 h-4 text-zinc-500" />
        <h4 className="text-sm font-bold text-zinc-100 font-mono">
          {zh ? '5. ⚙️ 创世参数微调控制台' : '5. ⚙️ Genesis Parameter Tuning Console'}
        </h4>
      </div>
      <div className="border border-zinc-700/40 bg-zinc-900/40 rounded-xl p-5 space-y-3">
        {GENESIS_SPLITS.map((s) => (
          <div key={s.labelEn} className="space-y-1">
            <div className="flex justify-between font-mono text-[11px]">
              <span className={`font-bold ${s.color}`}>{zh ? s.labelZh : s.labelEn}</span>
              <span className={`font-black ${s.color}`}>{s.pct}%</span>
            </div>
            <div className="h-4 bg-zinc-800 rounded-full overflow-hidden border border-zinc-700/40">
              <div className={`h-full ${s.track} opacity-40 rounded-full`} style={{ width: `${s.pct}%` }} />
            </div>
          </div>
        ))}
        <p className="text-zinc-600 font-mono text-[10px] pt-2 border-t border-zinc-800">
          {zh ? '创世参数已链上锁定至 Epoch 10' : 'Genesis parameters locked on-chain until Epoch 10'}
        </p>
      </div>
    </div>
  );
}

const content = {
  zh: {
    heading: 'α 协议双轨制去中心化治理看板',
    matrixTitle: '治理权限分摊矩阵',
    track1Title: '轨道一：陪审团日常裁决提案',
    track1Scope: '适用场景：绿标合规审计评级、涉嫌 Rug 资产定性、链上清算纠纷初审。',
    track1Threshold: '法定通过门槛：51% 相对多数赞成票。',
    track2Title: '轨道二：国库调拨与最高弹劾提案',
    track2Scope: '适用场景：动用多签蓄水池资金理赔、修改底层风控参数。',
    track2Threshold: '法定通过门槛：66% 绝对多数赞成票。',
    votingTitle: '二次方投票治理中心',
    verifiabilityTitle: '链上治理对账提示',
    verifiabilityText:
      '连接钱包后，系统将读取您在 Realms (SPL-Governance) 中的 ve-lock 权重。有效票数 = √(投入 α 数量)，大户边际效益递减。',
  },
  en: {
    heading: 'α Protocol Dual-Track DAO Governance',
    matrixTitle: 'Governance Authorization Matrix',
    track1Title: 'Track 1: Jury Verdict Registry',
    track1Scope: 'Scope: compliance ratings, rug determination, dispute review.',
    track1Threshold: 'Threshold: 51% relative majority.',
    track2Title: 'Track 2: Supreme Veto & Treasury',
    track2Scope: 'Scope: treasury disbursement, core parameter changes.',
    track2Threshold: 'Threshold: 66% supermajority.',
    votingTitle: 'Quadratic Voting Center',
    verifiabilityTitle: 'On-Chain Verifiability',
    verifiabilityText:
      'When connected, ve-lock weight is fetched from Realms. Effective Votes = √(α staked) — whale diminishing returns enforced.',
  },
};

export default function DAOGovernance({ lang }: Props) {
  const c = content[lang];
  const zh = lang === 'zh';
  const { publicKey, connected } = useWallet();
  const [voteAmount, setVoteAmount] = useState('10000');

  const connectHint = zh ? '请先连接钱包以激活投票权' : 'Connect wallet to activate voting';
  const shortAddress = publicKey
    ? `${publicKey.toBase58().slice(0, 4)}...${publicKey.toBase58().slice(-4)}`
    : null;

  const trackConfig = [
    {
      icon: Users,
      border: 'border-green-400/20 bg-green-400/5',
      titleColor: 'text-green-400',
      leftBorder: 'border-green-400/30',
      thresholdColor: 'text-green-300/90',
      barColor: 'bg-green-400',
      pct: 51,
      title: c.track1Title,
      scope: c.track1Scope,
      threshold: c.track1Threshold,
    },
    {
      icon: Zap,
      border: 'border-orange-400/20 bg-orange-400/5',
      titleColor: 'text-orange-400',
      leftBorder: 'border-orange-400/30',
      thresholdColor: 'text-orange-300/90',
      barColor: 'bg-orange-400',
      pct: 66,
      title: c.track2Title,
      scope: c.track2Scope,
      threshold: c.track2Threshold,
    },
  ];

  return (
    <section className="space-y-8">
      <div className="flex items-center gap-3">
        <Vote className="w-5 h-5 text-green-400" />
        <h3 className="text-lg font-bold text-zinc-200 font-mono tracking-wide">{c.heading}</h3>
      </div>

      <div
        className={`flex items-center gap-2 px-4 py-2.5 rounded-lg border font-mono text-xs transition-colors ${
          connected ? 'border-green-400/30 bg-green-400/5 text-green-400' : 'border-zinc-800 bg-zinc-950 text-zinc-500'
        }`}
      >
        <Wallet className="w-3.5 h-3.5 flex-shrink-0" />
        {connected && shortAddress ? (
          <span>
            {zh ? '治理账户' : 'Governance Account'}: <span className="font-bold tabular-nums">{shortAddress}</span>
          </span>
        ) : (
          <span>{connectHint}</span>
        )}
      </div>

      {/* 1. Matrix */}
      <div className="space-y-4">
        <div className="flex items-center gap-2 border-b border-zinc-800 pb-2">
          <Shield className="w-4 h-4 text-zinc-500" />
          <h4 className="text-sm font-bold text-zinc-100 font-mono">1. {c.matrixTitle}</h4>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
          {trackConfig.map((tr) => {
            const Icon = tr.icon;
            return (
              <div key={tr.title} className={`border ${tr.border} rounded-xl p-5 space-y-3`}>
                <div className="flex items-center gap-2">
                  <Icon className={`w-4 h-4 ${tr.titleColor}`} />
                  <p className={`text-sm font-bold ${tr.titleColor} font-mono`}>{tr.title}</p>
                </div>
                <div className={`pl-3 border-l-2 ${tr.leftBorder} space-y-2`}>
                  <p className="text-xs text-zinc-400 font-mono">{tr.scope}</p>
                  <p className={`text-xs ${tr.thresholdColor} font-mono font-semibold`}>{tr.threshold}</p>
                </div>
                <div className="relative h-2 bg-zinc-800 rounded-full overflow-hidden">
                  <div className={`absolute top-0 bottom-0 w-0.5 ${tr.barColor}/80`} style={{ left: `${tr.pct}%` }} />
                  <div className={`h-full ${tr.barColor}/20 rounded-full`} style={{ width: `${tr.pct}%` }} />
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* 2. Quadratic Voting — DAO 003 */}
      <div className="space-y-4">
        <div className="flex items-center gap-2 border-b border-zinc-800 pb-2">
          <Vote className="w-4 h-4 text-green-400" />
          <h4 className="text-sm font-bold text-zinc-100 font-mono">2. {c.votingTitle}</h4>
        </div>
        <QuadraticVotingPanel
          lang={lang}
          connected={connected}
          connectHint={connectHint}
          voteAmount={voteAmount}
          setVoteAmount={setVoteAmount}
        />
      </div>

      <RoleElectionDashboard lang={lang} connected={connected} connectHint={connectHint} voteAmount={voteAmount} />

      <GenesisParameterConsole lang={lang} />

      <div className="space-y-3">
        <div className="flex items-center gap-2 border-b border-zinc-800 pb-2">
          <Lock className="w-4 h-4 text-zinc-500" />
          <h4 className="text-sm font-bold text-zinc-100 font-mono">6. {c.verifiabilityTitle}</h4>
        </div>
        <div className="border border-zinc-700/50 bg-zinc-900/50 rounded-xl p-4 flex items-start gap-3">
          <Shield className="w-4 h-4 text-zinc-400 flex-shrink-0 mt-0.5" />
          <p className="text-xs text-zinc-400 font-mono leading-relaxed">{c.verifiabilityText}</p>
        </div>
      </div>
    </section>
  );
}
