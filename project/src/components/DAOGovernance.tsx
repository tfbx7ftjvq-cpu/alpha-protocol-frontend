import { useState } from 'react';
import { Shield, Users, Clock, Vote, Zap, Lock, CheckCircle, XCircle, UserX, RefreshCw, AlertTriangle, ChevronRight, SlidersHorizontal } from 'lucide-react';
import { Lang } from '../translations';

interface Props {
  lang: Lang;
}

interface Proposal {
  id: string;
  tag: string;
  name: string;
  type: string;
  typeTag: string;
  description: string;
  approved: number;
  target: number;
  status: string;
  timeRemaining: string;
  track: 1 | 2;
}

// ─── Shared sub-components ────────────────────────────────────────────────────
function ThresholdBar({ pct, color, label }: { pct: number; color: string; label: string }) {
  return (
    <>
      <div className={`relative h-2 bg-zinc-800 rounded-full overflow-hidden border border-zinc-700/40`}>
        <div className={`absolute top-0 bottom-0 w-0.5 ${color} z-10`} style={{ left: `${pct}%`, opacity: 0.8 }} />
        <div className={`h-full rounded-full`} style={{ width: `${pct}%`, background: `color-mix(in srgb, currentColor 20%, transparent)` }} />
      </div>
      <p className="text-[10px] text-zinc-600 font-mono text-right">{label}</p>
    </>
  );
}

function VoteBar({
  label, value, threshold, passed, barClass, labelClass,
}: {
  label: string; value: number; threshold?: number; passed?: boolean;
  barClass: string; labelClass: string;
}) {
  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between font-mono text-[11px]">
        <span className={`font-bold ${passed ? 'text-emerald-400' : labelClass}`}>{label}</span>
        <span className={`font-bold tabular-nums ${passed ? 'text-emerald-400' : labelClass}`}>{value.toFixed(1)}%</span>
      </div>
      <div className={`relative bg-zinc-800 rounded-full overflow-hidden border border-zinc-700/50 ${threshold !== undefined ? 'h-4' : 'h-2.5'}`}>
        {threshold !== undefined && (
          <div className="absolute top-0 bottom-0 w-0.5 bg-emerald-400/60 z-10" style={{ left: `${threshold}%` }} />
        )}
        <div
          className={`h-full rounded-full transition-all duration-500 relative overflow-hidden ${barClass}`}
          style={{ width: `${Math.min(value, 100)}%` }}
        >
          {threshold !== undefined && (
            <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent animate-shimmer" />
          )}
        </div>
        {threshold !== undefined && (
          <span className="absolute right-2 top-1/2 -translate-y-1/2 text-[8px] font-mono text-zinc-600 z-10">
            {threshold}% ——
          </span>
        )}
      </div>
    </div>
  );
}

// ─── DAO Governance content ───────────────────────────────────────────────────
const content = {
  zh: {
    heading: 'α 协议双轨制去中心化治理看板',
    matrixTitle: '治理权限分摊矩阵',
    matrixSubtitle: 'Governance Authorization Matrix',
    track1Title: '轨道一：陪审团日常裁决提案',
    track1Subtitle: 'Jury Verdict Registry',
    track1Scope: '适用场景：绿标合规审计评级、涉嫌 Rug 资产定性、链上清算纠纷初审。',
    track1Threshold: '法定通过门槛：硬编码锁定 51% 相对多数赞成票 (51% Consensus Line)。',
    track2Title: '轨道二：国库调拨与最高弹劾提案',
    track2Subtitle: 'Supreme Veto & Treasury Allocation',
    track2Scope: '适用场景：动用多签蓄水池资金理赔、修改底层风控参数、强制废除当前多签签署路径。',
    track2Threshold: '法定通过门槛：硬编码锁定 66% 绝对多数赞成票 (66% DAO Supreme Veto Line)。',
    proposalsTitle: '实时活跃提案对账',
    proposalsSubtitle: '支持模拟投票交互与占比计算',
    verifiabilityTitle: '链上治理对账提示',
    verifiabilityText: '连接钱包后，系统将自动通过 RPC 接口读取您在 Realms (SPL-Governance) 中的 ve-lock 锁仓权重。您的实时投票占比 = (实际 α 锁仓量 * 时间锁仓红利) / 全网总算力。',
    approveBtn: '赞成',
    rejectBtn: '反对',
    approvedLabel: '已投赞成票',
    targetLabel: '法定目标',
    proposals: [
      {
        id: '002',
        tag: '提案 #002',
        name: '陪审团关于 SOL-X 项目绿色合规标签初审评级公示',
        type: '陪审团日常裁决',
        typeTag: 'Jury Verdict',
        description: '陪审团已完成对该项目方代码仓库与团队锁仓地址的真实性审计，提议授予 α 绿标认证，现提交 DAO 进行二级表决。',
        approved: 54.2,
        target: 51.0,
        status: '超过通过线',
        timeRemaining: '剩余 14小时20分钟',
        track: 1 as const,
      },
      {
        id: '003',
        tag: '提案 #003',
        name: '动用 Squads 国库多签进行第一阶段受害者资产重组清算',
        type: '国库调拨与最高弹劾',
        typeTag: 'Treasury & Veto',
        description: '申请从多签账户中提取 5,000 USDC，按照前端阶梯理赔积分榜对首批 45 个合格受害者地址进行自动化线性释放。',
        approved: 48.8,
        target: 66.0,
        status: '计票中',
        timeRemaining: '剩余 32小时10分钟',
        track: 2 as const,
      },
    ] as Proposal[],
  },
  en: {
    heading: 'α Protocol Dual-Track DAO Governance Dashboard',
    matrixTitle: 'Governance Authorization Matrix',
    matrixSubtitle: '',
    track1Title: 'Track 1: Jury Verdict Registry',
    track1Subtitle: 'Daily Operational Governance',
    track1Scope: 'Scope: Green-label compliance audit ratings, protocol-level vulnerability determination, compromised asset assessment.',
    track1Threshold: 'Statutory Threshold: Hardcoded at 51% relative majority consensus (51% Consensus Line).',
    track2Title: 'Track 2: Supreme Veto & Treasury Allocation',
    track2Subtitle: 'Critical Risk Control',
    track2Scope: 'Scope: Disbursing liquidity from the Squads multi-sig reservoir, modifying core tokenomics parameters, or revoking administrative credentials.',
    track2Threshold: 'Statutory Threshold: Hardcoded at a strict 66% supermajority (66% DAO Supreme Veto Line).',
    proposalsTitle: 'Live Active Proposals',
    proposalsSubtitle: 'With Mock Interactivity & Share Calibration',
    verifiabilityTitle: 'On-Chain Verifiability Log',
    verifiabilityText: 'Upon wallet connection, the interface fetches your ve-lock weight natively from Realms via RPC. Your Live Voting Power Share = (Actual α Staked * Lockup Period Multiplier) / Total Protocol Voting Depth.',
    approveBtn: 'APPROVE',
    rejectBtn: 'REJECT',
    approvedLabel: 'Approved',
    targetLabel: 'Target',
    proposals: [
      {
        id: '002',
        tag: 'Proposal #002',
        name: 'Jury Verdict on Green-Label Compliance Rating for SOL-X Project',
        type: 'Jury Verdict Registry',
        typeTag: 'Jury Verdict',
        description: 'The Jury has finalized its source-code and liquidity lockup audit for the target project. Proposed to grant α green-label certification; now submitted to the DAO for secondary validation.',
        approved: 54.2,
        target: 51.0,
        status: 'Threshold Passed',
        timeRemaining: '14h 20m Remaining',
        track: 1 as const,
      },
      {
        id: '003',
        tag: 'Proposal #003',
        name: 'Allocation of Squads Treasury for Phase 1 Asset Reorganization Liquidation',
        type: 'Supreme Veto & Treasury Allocation',
        typeTag: 'Treasury & Veto',
        description: 'Requesting the disbursement of 5,000 USDC from the multi-sig account to execute linear restitution for the first batch of 45 verified wallets based on laddered points.',
        approved: 48.8,
        target: 66.0,
        status: 'Active Voting',
        timeRemaining: '32h 10m Remaining',
        track: 2 as const,
      },
    ] as Proposal[],
  },
};

// ─── Proposal Card ────────────────────────────────────────────────────────────
function ProposalCard({ proposal, lang, c }: { proposal: Proposal; lang: Lang; c: (typeof content)['en'] }) {
  const [votes, setVotes] = useState(proposal.approved);
  const [voted, setVoted] = useState<'approve' | 'reject' | null>(null);

  const passed = votes >= proposal.target;
  const isTrack2 = proposal.track === 2;
  const typeColor = isTrack2 ? 'text-orange-400' : 'text-emerald-400';
  const typeBadge = isTrack2 ? 'border-orange-400/40 bg-orange-400/10' : 'border-emerald-400/40 bg-emerald-400/10';
  const borderBg  = isTrack2 ? 'border-orange-400/30 bg-orange-400/5' : 'border-emerald-400/30 bg-emerald-400/5';
  const barClass  = passed
    ? 'bg-gradient-to-r from-emerald-700 to-emerald-400'
    : isTrack2 ? 'bg-gradient-to-r from-orange-800 to-orange-400'
               : 'bg-gradient-to-r from-emerald-800 to-emerald-500';

  function handleVote(type: 'approve' | 'reject') {
    if (voted) return;
    setVoted(type);
    setVotes(v => parseFloat((type === 'approve' ? v + 2.3 : v - 1.1).toFixed(1)));
  }

  return (
    <div className={`border ${borderBg} rounded-xl p-5 space-y-4`}>
      <div className="flex items-start justify-between gap-3 flex-wrap">
        <p className="text-xs font-mono font-bold text-zinc-200 leading-snug flex-1 min-w-0">
          [{proposal.tag} - {proposal.name}]
        </p>
        <span className={`text-[10px] font-mono px-2 py-0.5 rounded border ${typeBadge} ${typeColor} whitespace-nowrap flex-shrink-0`}>
          {proposal.typeTag}
        </span>
      </div>

      <div className="space-y-1.5">
        <p className="text-[11px] font-mono text-zinc-500">
          {lang === 'zh' ? '提案类别' : 'Type'}: <span className={`${typeColor} font-semibold`}>{proposal.type}</span>
        </p>
        <p className="text-xs text-zinc-400 font-mono leading-relaxed">{proposal.description}</p>
      </div>

      <div className="space-y-2">
        <div className="flex items-center justify-between text-[10px] font-mono text-zinc-500">
          <span className="flex items-center gap-1">
            <Clock className="w-3 h-3" />
            {proposal.timeRemaining}
          </span>
          <span className={`font-bold ${passed ? 'text-emerald-400' : typeColor}`}>{proposal.status}</span>
        </div>

        <div className="relative h-5 bg-zinc-800 rounded-full overflow-hidden border border-zinc-700/50">
          <div
            className="absolute top-0 bottom-0 w-0.5 z-10"
            style={{ left: `${proposal.target}%`, background: isTrack2 ? 'rgba(251,146,60,0.7)' : 'rgba(52,211,153,0.7)' }}
          />
          <div
            className="absolute -top-0.5 z-10 text-[8px] font-mono font-bold"
            style={{ left: `${proposal.target}%`, transform: 'translateX(-50%)', color: isTrack2 ? 'rgba(251,146,60,0.9)' : 'rgba(52,211,153,0.9)' }}
          >
            {proposal.target}%
          </div>
          <div
            className={`h-full rounded-full transition-all duration-500 relative overflow-hidden ${barClass}`}
            style={{ width: `${Math.min(votes, 100)}%` }}
          >
            <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent animate-shimmer" />
          </div>
        </div>

        <div className="flex items-center justify-between text-[10px] font-mono">
          <span className={passed ? 'text-emerald-400 font-bold' : 'text-zinc-300'}>
            {c.approvedLabel} {votes.toFixed(1)}%
          </span>
          <span className="text-zinc-500">{c.targetLabel} {proposal.target}%</span>
        </div>
      </div>

      <div className="flex gap-3 pt-1">
        {(['approve', 'reject'] as const).map(type => {
          const isApprove = type === 'approve';
          const active = voted === type;
          const disabled = voted !== null;
          const activeClass   = isApprove ? 'border-emerald-400/60 bg-emerald-400/20 text-emerald-400' : 'border-red-400/60 bg-red-400/20 text-red-400';
          const idleClass     = isApprove ? 'border-emerald-500/40 bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20 hover:border-emerald-500/60' : 'border-red-500/40 bg-red-500/10 text-red-400 hover:bg-red-500/20 hover:border-red-500/60';
          return (
            <button
              key={type}
              onClick={() => handleVote(type)}
              disabled={disabled}
              className={`flex-1 flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg border text-xs font-mono font-bold tracking-wide transition-all duration-200 ${
                active ? activeClass : disabled ? 'border-zinc-700/50 text-zinc-600 cursor-not-allowed opacity-50' : idleClass
              }`}
            >
              {isApprove ? <CheckCircle className="w-3.5 h-3.5" /> : <XCircle className="w-3.5 h-3.5" />}
              {isApprove ? c.approveBtn : c.rejectBtn}
            </button>
          );
        })}
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
  { id: 1, roleEn: 'Core Technical Infrastructure',        roleZh: '核心技术流',  tagEn: 'Core Dev',   tagZh: '核心开发', poolPct: 40, revenuePct: 8.0, address: 'AnLi...mF9k', status: 'impeachment_pending' },
  { id: 2, roleEn: 'Global Growth & Community Operations', roleZh: '全球宣发流',  tagEn: 'Global Ops', tagZh: '全球运营', poolPct: 30, revenuePct: 6.0, address: 'Ops7...XyZ2', status: 'active' },
  { id: 3, roleEn: 'Jury Pool',                            roleZh: '陪审团池',    tagEn: 'Jury Pool',  tagZh: '陪审团',   poolPct: 20, revenuePct: 4.0, address: 'Jury...K3p1', status: 'active' },
  { id: 4, roleEn: 'Protocol Reserve',                     roleZh: '协议储备金',  tagEn: 'Reserve',    tagZh: '储备金',   poolPct: 10, revenuePct: 2.0, address: 'Res9...7Wq0', status: 'active' },
];

const PAYROLL_LEGEND = [
  { label: { en: 'Core Dev 40%',  zh: '核心开发 40%' }, dot: 'bg-cyan-500'   },
  { label: { en: 'Global Ops 30%', zh: '全球运营 30%' }, dot: 'bg-green-500' },
  { label: { en: 'Jury Pool 20%',  zh: '陪审团池 20%' }, dot: 'bg-yellow-500'},
  { label: { en: 'Reserve 10%',    zh: '储备金 10%'   }, dot: 'bg-zinc-500'  },
];

function RoleElectionDashboard({ lang }: { lang: Lang }) {
  const isZh = lang === 'zh';
  const [slots, setSlots] = useState<PayrollSlot[]>(INITIAL_SLOTS);
  const [yesVotes, setYesVotes] = useState(42.5);
  const [noVotes, setNoVotes]   = useState(15.0);
  const [p004Voted, setP004Voted] = useState<'for' | 'against' | null>(null);
  const [p004Executed, setP004Executed] = useState(false);

  const THRESHOLD = 51.0;
  const p004Passed = yesVotes >= THRESHOLD;

  function updateSlotStatus(id: number, status: PayrollSlot['status']) {
    setSlots(prev => prev.map(s => s.id === id ? { ...s, status } : s));
  }

  function vote(side: 'for' | 'against') {
    if (p004Voted) return;
    setP004Voted(side);
    if (side === 'for') setYesVotes(v => parseFloat(Math.min(v + 3.8, 100).toFixed(1)));
    else               setNoVotes(v =>  parseFloat(Math.min(v + 2.1, 100).toFixed(1)));
  }

  function executeProposal() {
    if (!p004Passed || p004Executed) return;
    setP004Executed(true);
    setSlots(prev => prev.map(s => s.id === 1 ? { ...s, address: 'NewD...8v7x', status: 'active' } : s));
  }

  const visibleSlots = slots.filter(s => s.status !== 'dismissed');

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-2 border-b border-zinc-800 pb-2">
        <RefreshCw className="w-4 h-4 text-zinc-500" />
        <h4 className="text-sm font-bold text-zinc-100 font-mono">
          {isZh ? '3. 动态基础设施角色选举与弹劾看板' : '3. Dynamic Infrastructure Role Election & Impeachment'}
        </h4>
      </div>

      {/* Payroll pool overview bar */}
      <div className="border border-zinc-700/50 bg-zinc-900/60 rounded-xl p-4 space-y-3">
        <p className="text-zinc-500 font-mono text-[10px] uppercase tracking-widest">
          {isZh ? '贡献者薪资池分配（占协议总营收 20%）' : 'Contributor Payroll Pool Allocation — 20% of Total Protocol Revenue'}
        </p>
        <div className="flex h-3 rounded-full overflow-hidden gap-0.5">
          <div className="bg-cyan-500/80"   style={{ width: '40%' }} title="Core Dev 40%" />
          <div className="bg-green-500/80"  style={{ width: '30%' }} title="Global Ops 30%" />
          <div className="bg-yellow-500/80" style={{ width: '20%' }} title="Jury Pool 20%" />
          <div className="bg-zinc-500/80"   style={{ width: '10%' }} title="Reserve 10%" />
        </div>
        <div className="flex flex-wrap gap-x-5 gap-y-1">
          {PAYROLL_LEGEND.map(({ label, dot }) => (
            <div key={label.en} className="flex items-center gap-1.5">
              <span className={`w-2 h-2 rounded-full ${dot}`} />
              <span className="text-[10px] font-mono text-zinc-400">{isZh ? label.zh : label.en}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Current Active Registry Table */}
      <div className="border border-zinc-700/50 rounded-xl overflow-hidden bg-zinc-900/40">
        <div className="px-4 py-3 border-b border-zinc-800 bg-zinc-900/70 flex items-center gap-2">
          <Shield className="w-3.5 h-3.5 text-zinc-500" />
          <span className="text-zinc-300 font-mono text-xs font-bold uppercase tracking-widest">
            {isZh ? '当前活跃地址登记册' : 'Current Active Registry'}
          </span>
          <span className="ml-auto text-[10px] font-mono text-zinc-600">
            {isZh ? '链上可变状态数组 · 实时' : 'On-chain mutable state array · LIVE'}
          </span>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-xs font-mono">
            <thead>
              <tr className="border-b border-zinc-800 bg-zinc-900/50">
                {[
                  isZh ? '槽位' : 'Slot',
                  isZh ? '职能角色' : 'Role',
                  isZh ? '薪资占比' : 'Pool Alloc.',
                  isZh ? '总营收占比' : 'Rev. Share',
                  isZh ? '当前绑定地址' : 'Assigned Address',
                  isZh ? '状态' : 'Status',
                  isZh ? '操作' : 'Action',
                ].map(col => (
                  <th key={col} className="text-left px-4 py-2.5 text-zinc-500 uppercase tracking-widest text-[10px] font-bold whitespace-nowrap">{col}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {visibleSlots.map(slot => {
                const isPending = slot.status === 'impeachment_pending';
                return (
                  <tr key={slot.id} className={`border-b border-zinc-800/50 transition-colors duration-200 ${isPending ? 'bg-red-950/20' : 'hover:bg-zinc-800/30'}`}>
                    <td className="px-4 py-3 text-zinc-500">#{slot.id}</td>
                    <td className="px-4 py-3">
                      <p className="text-zinc-200 font-bold">{isZh ? slot.roleZh : slot.roleEn}</p>
                      <p className="text-zinc-600 text-[10px]">{isZh ? slot.tagZh : slot.tagEn}</p>
                    </td>
                    <td className="px-4 py-3"><span className="text-cyan-400 font-bold">{slot.poolPct}%</span></td>
                    <td className="px-4 py-3"><span className="text-green-400 font-bold">{slot.revenuePct.toFixed(1)}%</span></td>
                    <td className="px-4 py-3">
                      <span className={`font-mono text-[11px] px-2 py-0.5 rounded border ${
                        p004Executed && slot.id === 1
                          ? 'border-green-400/40 bg-green-400/10 text-green-400'
                          : 'border-zinc-700 bg-zinc-800/60 text-zinc-300'
                      }`}>
                        {slot.address}
                      </span>
                      {p004Executed && slot.id === 1 && (
                        <span className="ml-2 text-[9px] font-mono text-green-400/70">{isZh ? '← 已替换' : '← replaced'}</span>
                      )}
                    </td>
                    <td className="px-4 py-3">
                      {isPending ? (
                        <span className="text-[10px] font-mono px-2 py-0.5 rounded border border-red-400/40 bg-red-400/10 text-red-400 animate-pulse">
                          {isZh ? '弹劾待决' : 'IMPEACHMENT PENDING'}
                        </span>
                      ) : (
                        <span className="text-[10px] font-mono px-2 py-0.5 rounded border border-emerald-400/30 bg-emerald-400/5 text-emerald-400">
                          {isZh ? '活跃' : 'ACTIVE'}
                        </span>
                      )}
                    </td>
                    <td className="px-4 py-3">
                      {!isPending ? (
                        <button
                          onClick={() => updateSlotStatus(slot.id, 'impeachment_pending')}
                          className="flex items-center gap-1 px-2.5 py-1 rounded border border-red-500/40 bg-red-500/10 text-red-400 text-[10px] font-bold hover:bg-red-500/20 hover:border-red-500/60 transition-all duration-150 whitespace-nowrap"
                        >
                          <UserX className="w-3 h-3" />
                          {isZh ? '弹劾' : 'Impeach'}
                        </button>
                      ) : (
                        <button
                          onClick={() => updateSlotStatus(slot.id, 'dismissed')}
                          className="flex items-center gap-1 px-2.5 py-1 rounded border border-zinc-600/40 bg-zinc-800/60 text-zinc-500 text-[10px] font-bold hover:text-zinc-400 transition-all duration-150 whitespace-nowrap"
                        >
                          {isZh ? '撤销' : 'Dismiss'}
                        </button>
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>

      {/* Proposal #004 election card */}
      <div className="border border-cyan-400/25 bg-cyan-400/5 rounded-xl overflow-hidden">
        <div className="flex items-center justify-between px-5 py-3 border-b border-cyan-400/20 bg-cyan-400/5">
          <div className="flex items-center gap-2">
            <Vote className="w-3.5 h-3.5 text-cyan-400" />
            <span className="text-cyan-400 font-mono text-xs font-bold uppercase tracking-widest">
              {isZh ? '选举提案 #004 · 活跃计票中' : 'Election Proposal #004 · Active Voting'}
            </span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-pulse" />
            <span className="text-[10px] font-mono text-cyan-400/70">{isZh ? '实时' : 'LIVE'}</span>
          </div>
        </div>

        <div className="p-5 space-y-5">
          <div className="space-y-2">
            <p className="text-zinc-100 font-mono text-sm font-bold leading-snug">
              {isZh ? '[提案 #004 - 替换核心技术基础设施负责人]' : '[Proposal #004 - Replace Core Technical Infrastructure Lead]'}
            </p>
            <p className="text-zinc-400 font-mono text-xs leading-relaxed">
              {isZh
                ? '社区已收到申请，拟将核心开发薪资流（8% 总营收）从当前地址 AnLi...mF9k 重新路由至新候选人地址 NewD...8v7x，依据为其开源代码贡献绩效审查报告。'
                : 'Community application received to route the 8% Core Dev payroll stream from AnLi...mF9k to new candidate address NewD...8v7x based on their open-source performance review.'}
            </p>
          </div>

          {/* Address routing */}
          <div className="flex items-center gap-3 bg-zinc-900/60 border border-zinc-700/40 rounded-lg px-4 py-3 flex-wrap gap-y-2">
            <div className="space-y-0.5">
              <p className="text-[9px] font-mono text-zinc-600 uppercase tracking-widest">{isZh ? '当前持有人' : 'Incumbent'}</p>
              <span className="font-mono text-xs px-2 py-0.5 rounded border border-red-400/30 bg-red-400/10 text-red-300">AnLi...mF9k</span>
            </div>
            <ChevronRight className="w-4 h-4 text-zinc-600 flex-shrink-0" />
            <div className="space-y-0.5">
              <p className="text-[9px] font-mono text-zinc-600 uppercase tracking-widest">{isZh ? '候选人' : 'Candidate'}</p>
              <span className="font-mono text-xs px-2 py-0.5 rounded border border-cyan-400/40 bg-cyan-400/10 text-cyan-300">NewD...8v7x</span>
            </div>
            <div className="ml-auto space-y-0.5 text-right">
              <p className="text-[9px] font-mono text-zinc-600 uppercase tracking-widest">{isZh ? '涉及薪资流' : 'Affected Stream'}</p>
              <span className="font-mono text-xs px-2 py-0.5 rounded border border-zinc-600 bg-zinc-800/60 text-zinc-300">8.0% {isZh ? '总营收' : 'rev.'}</span>
            </div>
          </div>

          {/* Voting bars */}
          <div className="space-y-3">
            <p className="text-zinc-500 font-mono text-[10px] uppercase tracking-widest">
              {isZh ? '实时投票状态 · 法定通过线 51%' : 'Live Voting Status · 51% Statutory Threshold'}
            </p>
            <VoteBar
              label={isZh ? '赞成（支持候选人）' : 'YES — Vote for New Candidate'}
              value={yesVotes}
              threshold={THRESHOLD}
              passed={p004Passed}
              barClass={p004Passed ? 'bg-gradient-to-r from-emerald-700 to-emerald-400' : 'bg-gradient-to-r from-zinc-600 to-zinc-400'}
              labelClass="text-zinc-300"
            />
            <VoteBar
              label={isZh ? '反对（保留现任）' : 'NO — Keep Incumbent'}
              value={noVotes}
              barClass="bg-gradient-to-r from-zinc-700 to-zinc-500"
              labelClass="text-zinc-500"
            />
            <p className="text-[10px] font-mono text-zinc-600">
              {isZh ? '弃权 / 未投票' : 'Abstain / Not Yet Voted'}:{' '}
              <span className="text-zinc-500">{Math.max(0, 100 - yesVotes - noVotes).toFixed(1)}%</span>
            </p>
          </div>

          {/* Vote buttons */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <button
              onClick={() => vote('for')}
              disabled={p004Voted !== null}
              className={`flex items-center justify-center gap-2 px-4 py-3 rounded-lg border text-xs font-mono font-bold tracking-widest uppercase transition-all duration-200 ${
                p004Voted === 'for'    ? 'border-emerald-400/60 bg-emerald-400/20 text-emerald-400'
                : p004Voted !== null  ? 'border-zinc-700/40 text-zinc-600 cursor-not-allowed opacity-40'
                : 'border-emerald-500/40 bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20 hover:border-emerald-500/60 hover:shadow-emerald-500/10 hover:shadow-lg'
              }`}
            >
              <CheckCircle className="w-3.5 h-3.5" />
              {isZh ? '投票支持新候选人' : 'Vote for New Candidate'}
            </button>
            <button
              onClick={() => vote('against')}
              disabled={p004Voted !== null}
              className={`flex items-center justify-center gap-2 px-4 py-3 rounded-lg border text-xs font-mono font-bold tracking-widest uppercase transition-all duration-200 ${
                p004Voted === 'against' ? 'border-red-400/60 bg-red-400/20 text-red-400'
                : p004Voted !== null   ? 'border-zinc-700/40 text-zinc-600 cursor-not-allowed opacity-40'
                : 'border-red-500/40 bg-red-500/10 text-red-400 hover:bg-red-500/20 hover:border-red-500/60 hover:shadow-red-500/10 hover:shadow-lg'
              }`}
            >
              <XCircle className="w-3.5 h-3.5" />
              {isZh ? '投票保留现任' : 'Vote to Keep Incumbent'}
            </button>
          </div>

          {p004Passed && !p004Executed && (
            <div className="border border-emerald-400/40 bg-emerald-400/8 rounded-lg p-4 space-y-3">
              <div className="flex items-start gap-2">
                <AlertTriangle className="w-4 h-4 text-emerald-400 mt-0.5 flex-shrink-0" />
                <div>
                  <p className="text-emerald-400 font-mono text-xs font-bold">
                    {isZh ? '51% 法定门槛已触达 — 后端执行脚本就绪' : '51% Statutory Threshold Crossed — Backend Execution Script Ready'}
                  </p>
                  <p className="text-zinc-500 font-mono text-[10px] mt-1 leading-relaxed">
                    {isZh
                      ? '确认执行将在下一周期自动将薪资流地址索引从 AnLi...mF9k 覆写为 NewD...8v7x。此操作不可逆。'
                      : 'Confirming will overwrite the payroll stream address index from AnLi...mF9k to NewD...8v7x at the start of the next revenue epoch. Irreversible.'}
                  </p>
                </div>
              </div>
              <button
                onClick={executeProposal}
                className="w-full flex items-center justify-center gap-2 py-2.5 rounded-lg border border-emerald-400/50 bg-emerald-400/15 text-emerald-400 text-xs font-mono font-bold uppercase tracking-widest hover:bg-emerald-400/25 transition-all duration-200"
              >
                <RefreshCw className="w-3.5 h-3.5" />
                {isZh ? '确认执行地址覆写' : 'Confirm & Execute Address Overwrite'}
              </button>
            </div>
          )}

          {p004Executed && (
            <div className="border border-emerald-400/30 bg-emerald-400/5 rounded-lg px-4 py-3 flex items-center gap-2">
              <CheckCircle className="w-4 h-4 text-emerald-400 flex-shrink-0" />
              <p className="text-emerald-400 font-mono text-xs font-bold">
                {isZh
                  ? '[执行完成] — 核心开发薪资流地址已于本周期覆写为 NewD...8v7x。'
                  : '[EXECUTED] — Core Dev payroll stream address overwritten to NewD...8v7x for this epoch.'}
              </p>
            </div>
          )}
        </div>
      </div>

      {/* Institutional Notice Footer */}
      <div className="border border-zinc-800 bg-zinc-950/80 rounded-lg px-4 py-3">
        <p className="text-zinc-600 font-mono text-[10px] leading-relaxed">
          <span className="text-zinc-500 font-bold">[SYSTEM-NOTICE]:</span>{' '}
          {isZh
            ? '所有人员分配均以可变状态数组形式原生存储于链上。提案跨越 51% 共识线后，自动化后端执行脚本将于下一营收周期自动覆写对应地址索引。不存在管理员私钥手动干预路径。'
            : 'All personnel assignments are stored as mutable state arrays natively on-chain. Upon a proposal crossing the 51% consensus marker, the automated backend execution script will automatically overwrite the recipient address index for the next revenue epoch. No administrative private key overrides possible.'}
        </p>
      </div>
    </div>
  );
}

// ─── Genesis Parameter Tuning Console ────────────────────────────────────────
const GENESIS_SPLITS = [
  { pct: 50, labelZh: '散户赔付救济金池',  labelEn: 'Retail Relief Pool',      color: 'text-green-400',  track: 'bg-green-400',  border: 'border-green-400/30'  },
  { pct: 20, labelZh: '自动回购销毁矩阵',  labelEn: 'Buyback & Burn Matrix',   color: 'text-red-400',    track: 'bg-red-400',    border: 'border-red-400/30'    },
  { pct: 20, labelZh: 'DAO 建设者工资池',  labelEn: 'DAO Contributor Payroll', color: 'text-blue-400',   track: 'bg-blue-400',   border: 'border-blue-400/30'   },
  { pct: 10, labelZh: '纯代币质押分红池',  labelEn: 'Pure Staking Dividend',   color: 'text-yellow-400', track: 'bg-yellow-400', border: 'border-yellow-400/30' },
];

function GenesisParameterConsole({ lang }: { lang: Lang }) {
  const isZh = lang === 'zh';
  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2 border-b border-zinc-800 pb-2">
        <SlidersHorizontal className="w-4 h-4 text-zinc-500" />
        <h4 className="text-sm font-bold text-zinc-100 font-mono">
          {isZh ? '5. ⚙️ 创世参数微调控制台' : '5. ⚙️ Genesis Parameter Tuning Console'}
        </h4>
        <span className="ml-auto text-[10px] text-zinc-600 font-mono">
          {isZh ? '创世参数微调' : 'Fiscal Realignment'}
        </span>
      </div>

      <div className="border border-zinc-700/40 bg-zinc-900/40 rounded-xl overflow-hidden">
        {/* Header bar */}
        <div className="flex items-center justify-between px-5 py-3 border-b border-zinc-800 bg-zinc-950/60">
          <div className="flex items-center gap-2">
            <div className="p-1.5 rounded-lg bg-zinc-700/40 border border-zinc-600/40">
              <Lock className="w-3.5 h-3.5 text-zinc-500" />
            </div>
            <div>
              <p className="text-zinc-300 font-mono text-xs font-bold">
                {isZh ? '国库分配比例 — 创世锁定参数' : 'Treasury Allocation Ratios — Genesis Locked Parameters'}
              </p>
              <p className="text-zinc-600 font-mono text-[10px]">
                {isZh ? '50 / 20 / 20 / 10 链上硬编码' : '50 / 20 / 20 / 10 hardcoded on-chain'}
              </p>
            </div>
          </div>
          <span className="text-[10px] font-mono px-2.5 py-1 rounded border border-zinc-600/50 bg-zinc-800/60 text-zinc-400 font-bold whitespace-nowrap">
            {isZh ? '状态: 固定至 Epoch 10' : 'STATUS: FIXED UNTIL EPOCH 10'}
          </span>
        </div>

        <div className="p-5 space-y-4">
          {/* Slider rows */}
          <div className="space-y-3">
            {GENESIS_SPLITS.map((s) => (
              <div key={s.labelEn} className="space-y-1.5">
                <div className="flex items-center justify-between font-mono text-[11px]">
                  <div className="flex items-center gap-2">
                    <Lock className="w-2.5 h-2.5 text-zinc-600 flex-shrink-0" />
                    <span className={`font-bold ${s.color}`}>
                      {isZh ? s.labelZh : s.labelEn}
                    </span>
                  </div>
                  <span className={`font-black tabular-nums ${s.color}`}>{s.pct}%</span>
                </div>
                {/* Locked slider track */}
                <div className="relative h-5 bg-zinc-800/80 rounded-full overflow-hidden border border-zinc-700/40 cursor-not-allowed">
                  {/* Filled portion */}
                  <div
                    className={`h-full rounded-full ${s.track} opacity-40`}
                    style={{ width: `${s.pct}%` }}
                  />
                  {/* Lock icon overlay at thumb position */}
                  <div
                    className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 z-10"
                    style={{ left: `${s.pct}%` }}
                  >
                    <div className={`w-4 h-4 rounded-full border-2 ${s.border} bg-zinc-900 flex items-center justify-center`}>
                      <Lock className="w-2 h-2 text-zinc-500" />
                    </div>
                  </div>
                  {/* "LOCKED" label */}
                  <span className="absolute right-2 top-1/2 -translate-y-1/2 text-[8px] font-mono text-zinc-600 font-bold tracking-widest select-none">
                    {isZh ? '已锁定' : 'LOCKED'}
                  </span>
                </div>
              </div>
            ))}
          </div>

          {/* Total check */}
          <div className="flex items-center justify-between border-t border-zinc-800 pt-3 font-mono text-[11px]">
            <span className="text-zinc-600">{isZh ? '合计' : 'Total'}</span>
            <span className="text-zinc-400 font-bold tabular-nums">
              {GENESIS_SPLITS.reduce((acc, s) => acc + s.pct, 0)}% = 100%
            </span>
          </div>

          {/* Governance footnote */}
          <div className="border border-zinc-700/30 bg-zinc-950/50 rounded-lg px-4 py-3">
            <p className="text-zinc-600 font-mono text-[10px] leading-relaxed">
              <span className="text-zinc-500 font-bold">[GOVERNANCE-RULE]:</span>{' '}
              {isZh
                ? 'Epoch 10 结束后，任何 $α 持有人均可锁定 ve 权重，发起核心财政重分配提案（Core Fiscal Realignment Proposal）。调整上述绝对百分比通道需满足 66% 法定绝对多数赞成票，并须经过 72 小时链上时间锁（On-Chain Timelock）方可执行。代码即治理，治理即法律。'
                : 'Post Epoch 10, any $α holder can lock ve-weight to initiate a Core Fiscal Realignment Proposal. Adjusting these absolute percentage channels requires a 66% statutory supermajority and a 72-hour on-chain timelock to execute. Code is governance, governance is law.'}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

// ─── Main export ──────────────────────────────────────────────────────────────
export default function DAOGovernance({ lang }: Props) {
  const c = content[lang];
  const isZh = lang === 'zh';

  const trackConfig = [
    {
      icon: Users, iconClass: 'bg-emerald-400/10 border-emerald-400/30', iconColor: 'text-emerald-400',
      border: 'border-emerald-400/20 bg-emerald-400/5',
      titleColor: 'text-emerald-400',
      leftBorder: 'border-emerald-400/30',
      thresholdColor: 'text-emerald-300/90',
      barColor: 'bg-emerald-400',
      pct: 51,
      title: c.track1Title, subtitle: c.track1Subtitle,
      scope: c.track1Scope, threshold: c.track1Threshold,
      barLabel: '51% Consensus Line',
    },
    {
      icon: Zap, iconClass: 'bg-orange-400/10 border-orange-400/30', iconColor: 'text-orange-400',
      border: 'border-orange-400/20 bg-orange-400/5',
      titleColor: 'text-orange-400',
      leftBorder: 'border-orange-400/30',
      thresholdColor: 'text-orange-300/90',
      barColor: 'bg-orange-400',
      pct: 66,
      title: c.track2Title, subtitle: c.track2Subtitle,
      scope: c.track2Scope, threshold: c.track2Threshold,
      barLabel: '66% DAO Supreme Veto Line',
    },
  ];

  return (
    <section className="space-y-8">
      <div className="flex items-center gap-3">
        <Vote className="w-5 h-5 text-zinc-400" />
        <h3 className="text-lg font-bold text-zinc-200 font-mono tracking-wide">{c.heading}</h3>
      </div>

      {/* 1. Governance Matrix */}
      <div className="space-y-4">
        <div className="flex items-center gap-2 border-b border-zinc-800 pb-2">
          <Shield className="w-4 h-4 text-zinc-500" />
          <h4 className="text-sm font-bold text-zinc-100 font-mono">1. {c.matrixTitle}</h4>
          {c.matrixSubtitle && <span className="text-[10px] text-zinc-600 font-mono ml-2">{c.matrixSubtitle}</span>}
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
          {trackConfig.map(tr => {
            const Icon = tr.icon;
            return (
              <div key={tr.title} className={`border ${tr.border} rounded-xl p-5 space-y-3`}>
                <div className="flex items-center gap-2">
                  <div className={`p-1.5 rounded-lg ${tr.iconClass}`}>
                    <Icon className={`w-4 h-4 ${tr.iconColor}`} />
                  </div>
                  <div>
                    <p className={`text-sm font-bold ${tr.titleColor} font-mono`}>{tr.title}</p>
                    {tr.subtitle && <p className="text-[10px] text-zinc-500 font-mono">{tr.subtitle}</p>}
                  </div>
                </div>
                <div className={`space-y-2 pl-3 border-l-2 ${tr.leftBorder}`}>
                  <p className="text-xs text-zinc-400 font-mono leading-relaxed">{tr.scope}</p>
                  <p className={`text-xs ${tr.thresholdColor} font-mono leading-relaxed font-semibold`}>{tr.threshold}</p>
                </div>
                <div className={`relative h-2 bg-zinc-800 rounded-full overflow-hidden border border-zinc-700/40`}>
                  <div className={`absolute top-0 bottom-0 w-0.5 ${tr.barColor}/80 z-10`} style={{ left: `${tr.pct}%` }} />
                  <div className={`h-full ${tr.barColor}/20 rounded-full`} style={{ width: `${tr.pct}%` }} />
                </div>
                <p className="text-[10px] text-zinc-600 font-mono text-right">{tr.barLabel}</p>
              </div>
            );
          })}
        </div>
      </div>

      {/* 2. Live Active Proposals */}
      <div className="space-y-4">
        <div className="flex items-center gap-2 border-b border-zinc-800 pb-2">
          <Users className="w-4 h-4 text-zinc-500" />
          <h4 className="text-sm font-bold text-zinc-100 font-mono">2. {c.proposalsTitle}</h4>
          <span className="text-[10px] text-zinc-600 font-mono ml-2">{c.proposalsSubtitle}</span>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
          {c.proposals.map(proposal => (
            <ProposalCard key={proposal.id} proposal={proposal} lang={lang} c={c} />
          ))}
        </div>
      </div>

      {/* 3. Role Election & Impeachment */}
      <RoleElectionDashboard lang={lang} />

      {/* 5. Genesis Parameter Tuning Console */}
      <GenesisParameterConsole lang={lang} />

      {/* 6. On-Chain Verifiability */}
      <div className="space-y-3">
        <div className="flex items-center gap-2 border-b border-zinc-800 pb-2">
          <Lock className="w-4 h-4 text-zinc-500" />
          <h4 className="text-sm font-bold text-zinc-100 font-mono">6. {c.verifiabilityTitle}</h4>
        </div>
        <div className="border border-zinc-700/50 bg-zinc-900/50 rounded-xl p-4 flex items-start gap-3">
          <div className="flex-shrink-0 mt-0.5 p-1.5 rounded-lg bg-zinc-700/30 border border-zinc-600/40">
            <Shield className="w-4 h-4 text-zinc-400" />
          </div>
          <p className="text-xs text-zinc-400 font-mono leading-relaxed">{c.verifiabilityText}</p>
        </div>
        {isZh && <p className="text-zinc-700 font-mono text-[10px] pl-1">{/* verifiability note */}</p>}
      </div>
    </section>
  );
}
