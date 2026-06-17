import { useEffect, useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import {
  ArrowRight,
  BadgeCheck,
  Banknote,
  ExternalLink,
  FileText,
  Flag,
  Gavel,
  HandHeart,
  PlusCircle,
  Shield,
  Users,
  Vote,
  Wallet,
  X,
  Zap,
} from 'lucide-react';
import { type Lang } from '../translations';

interface Props {
  lang: Lang;
}

interface ExposureProject {
  id: string;
  project: string;
  rugDate: string;
  amountUsd: string;
  coSigners: number;
  chain: string;
  status: 'exposed' | 'hearing' | 'dao_pending';
}

const INITIAL_EXPOSURES: ExposureProject[] = [
  {
    id: 'proj-x',
    project: 'Project X',
    rugDate: '2026-05',
    amountUsd: '$12,000,000',
    coSigners: 1240,
    chain: 'SOL',
    status: 'dao_pending',
  },
  {
    id: 'shadow-swap',
    project: 'ShadowSwap Finance',
    rugDate: '2024-01',
    amountUsd: '$2,340,000',
    coSigners: 892,
    chain: 'ETH',
    status: 'hearing',
  },
  {
    id: 'invisible-vault',
    project: 'InvisibleVault DAO',
    rugDate: '2024-03',
    amountUsd: '$4,100,000',
    coSigners: 2103,
    chain: 'SOL',
    status: 'exposed',
  },
  {
    id: 'moon-rug',
    project: 'MoonRug Protocol',
    rugDate: '2024-02',
    amountUsd: '$890,000',
    coSigners: 456,
    chain: 'BSC',
    status: 'exposed',
  },
];

const certifiedData = [
  { name: 'SolNexus Matrix', chain: 'SOL', contribution: '2,500', date: '2026-05-20', rating: 'AAA', ratingZh: '优秀' },
  { name: 'Aegis Liquidity', chain: 'SOL', contribution: '1,800', date: '2026-05-24', rating: 'AA', ratingZh: '良好' },
];

const DAO_USES = [
  '赔付申请审核',
  '绿标认证',
  '风险项目曝光',
  '重大国库支出',
  '协议参数升级',
  '贡献者激励审批',
];

const PROPOSAL_FLOW = [
  'Draft 草案',
  'Active 投票中',
  'Passed 已通过',
  'Rejected 已拒绝',
  'Queued 待执行',
  'Executed 已执行',
];

const MOCK_DAO_PROPOSALS = [
  {
    title: '赔付申请：某项目跑路受害者赔付申请',
    type: '赔付申请',
    status: 'Active 投票中',
    icon: HandHeart,
    color: 'text-emerald-400 border-emerald-400/25 bg-emerald-400/5',
  },
  {
    title: '绿标认证：某项目申请 Alpha Green Label',
    type: '绿标认证',
    status: 'Draft 草案',
    icon: BadgeCheck,
    color: 'text-cyan-400 border-cyan-400/25 bg-cyan-400/5',
  },
  {
    title: '风险曝光：某高风险项目社区曝光提案',
    type: '风险曝光',
    status: 'Passed 已通过',
    icon: Flag,
    color: 'text-red-400 border-red-400/25 bg-red-400/5',
  },
  {
    title: '国库支出：安全审计费用支出提案',
    type: '国库支出',
    status: 'Queued 待执行',
    icon: Banknote,
    color: 'text-blue-400 border-blue-400/25 bg-blue-400/5',
  },
  {
    title: '贡献者激励：前端开发任务赏金提案',
    type: '贡献者激励',
    status: 'Draft 草案',
    icon: Users,
    color: 'text-violet-400 border-violet-400/25 bg-violet-400/5',
  },
];

const BUILDER_USES = [
  '协议开发',
  '安全审计',
  '前端设计',
  '社区运营',
  '风险项目调查',
  '赔付案件资料整理',
  '绿标认证尽调',
  '文档与多语言建设',
  '社区治理工具建设',
];

const BUILDER_FLOW = ['任务发布', '社区认领', '里程碑验收', 'DAO 审核', '激励发放'];

const GREEN_LABEL_FLOW = ['项目提交申请', '社区尽调', 'DAO 投票', '获得绿标', '持续监督', '必要时撤销认证'];

function statusBadge(status: ExposureProject['status']) {
  const map = {
    exposed: { label: '已曝光', className: 'text-red-400 border-red-400/40 bg-red-400/10' },
    hearing: { label: '听证中', className: 'text-yellow-400 border-yellow-400/40 bg-yellow-400/10' },
    dao_pending: { label: '待 DAO 表决', className: 'text-green-400 border-green-400/40 bg-green-400/10' },
  };
  return map[status];
}

export default function WallOfShame({ lang }: Props) {
  const locale = lang === 'zh' ? 'zh-CN' : 'en-US';
  const { publicKey, connected } = useWallet();
  const [activeTab, setActiveTab] = useState<0 | 1>(0);
  const [exposures, setExposures] = useState<ExposureProject[]>(INITIAL_EXPOSURES);
  const [joinedIds, setJoinedIds] = useState<Set<string>>(new Set());
  const [showReportForm, setShowReportForm] = useState(false);
  const [newProject, setNewProject] = useState('');
  const [newLoss, setNewLoss] = useState('');
  const [newChain, setNewChain] = useState('SOL');

  const shortAddress = publicKey
    ? `${publicKey.toBase58().slice(0, 4)}...${publicKey.toBase58().slice(-4)}`
    : null;

  useEffect(() => {
    document.body.style.overflow = showReportForm ? 'hidden' : '';
    return () => {
      document.body.style.overflow = '';
    };
  }, [showReportForm]);

  function joinClassAction(projectId: string) {
    if (!connected || joinedIds.has(projectId)) return;
    setExposures((prev) =>
      prev.map((p) => (p.id === projectId ? { ...p, coSigners: p.coSigners + 1 } : p))
    );
    setJoinedIds((prev) => new Set(prev).add(projectId));
  }

  function submitNewFraud() {
    if (!newProject.trim()) return;

    const now = new Date();
    const newExposure: ExposureProject = {
      id: `${newProject.trim().toLowerCase().replace(/[^a-z0-9]+/g, '-')}-${now.getTime()}`,
      project: newProject.trim(),
      rugDate: `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`,
      amountUsd: newLoss.trim() ? `$${newLoss.trim()}` : '$0',
      coSigners: connected ? 1 : 0,
      chain: newChain.trim() || 'SOL',
      status: 'dao_pending',
    };

    setExposures((prev) => [newExposure, ...prev]);
    setJoinedIds((prev) => connected ? new Set(prev).add(newExposure.id) : prev);
    setNewProject('');
    setNewLoss('');
    setNewChain('SOL');
    setShowReportForm(false);
  }

  return (
    <>
      <div className="space-y-6">
        <div className="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-4">
          <div className="flex items-center gap-3 min-w-0">
            <Gavel className="w-6 h-6 text-green-400 flex-shrink-0" />
            <div>
              <h2 className="text-xl font-black text-zinc-100 font-mono tracking-wide uppercase">
                链上法庭与 DAO 治理
              </h2>
              <p className="text-zinc-600 font-mono text-[10px] mt-0.5 uppercase tracking-widest">
                WALL_OF_SHAME · DAO_GOVERNANCE_ROADMAP · {new Date().toLocaleDateString(locale)}
              </p>
            </div>
          </div>

          <button
            type="button"
            onClick={() => connected && setShowReportForm(true)}
            disabled={!connected}
            className={`flex-shrink-0 inline-flex items-center justify-center gap-2 px-5 py-2.5 rounded-lg border text-xs font-mono font-bold uppercase tracking-wider transition-all duration-200 ${
              connected
                ? 'border-green-500/50 bg-green-500/15 text-green-400 hover:bg-green-500/25 hover:shadow-green-500/20 hover:shadow-lg'
                : 'border-zinc-700/50 bg-zinc-900/50 text-zinc-500 cursor-not-allowed opacity-60'
            }`}
          >
            <PlusCircle className="w-4 h-4" />
            {!connected ? '请先连接钱包' : '发起风险项目曝光'}
          </button>
        </div>

        <div
          className={`flex items-center gap-2 px-4 py-2.5 rounded-lg border font-mono text-xs transition-colors ${
            connected ? 'border-green-400/30 bg-green-400/5 text-green-400' : 'border-zinc-800 bg-zinc-950 text-zinc-500'
          }`}
        >
          <Wallet className="w-3.5 h-3.5 flex-shrink-0" />
          {connected && shortAddress ? (
            <span>
              当前钱包：<span className="font-bold tabular-nums">{shortAddress}</span>
            </span>
          ) : (
            <span>钱包未连接</span>
          )}
        </div>

        <div className="border border-green-400/30 bg-green-400/5 rounded-xl px-4 py-3 relative overflow-hidden">
          <div className="absolute top-0 left-0 w-1 h-full bg-green-400/60" />
          <p className="text-green-400 font-mono text-xs leading-relaxed pl-2">
            当前版本为 Devnet Alpha 测试网原型。风险曝光、受害者联署、绿标名录和 DAO 提案卡片均为路线图展示，不代表已开放真实链上 DAO 投票交易。
          </p>
        </div>

        <div className="border border-cyan-400/20 bg-cyan-400/5 rounded-xl overflow-hidden">
          <div className="px-5 py-4 border-b border-cyan-400/10 bg-zinc-950/70 flex items-center gap-2">
            <Vote className="w-4 h-4 text-cyan-400" />
            <div>
              <h3 className="text-zinc-100 font-mono text-sm font-black uppercase tracking-widest">
                DAO Governance / 治理中心
              </h3>
              <p className="text-zinc-500 font-mono text-[10px] mt-0.5">
                DAO 是 Alpha Protocol 的核心公共决策层。
              </p>
            </div>
          </div>
          <div className="p-5 space-y-4">
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
              {DAO_USES.map((item) => (
                <div key={item} className="rounded-lg border border-zinc-800 bg-zinc-950/70 px-3 py-2 text-xs font-mono font-bold text-zinc-300">
                  {item}
                </div>
              ))}
            </div>

            <div className="rounded-lg border border-yellow-400/25 bg-yellow-400/5 px-4 py-3 text-xs font-mono leading-relaxed text-yellow-100">
              质押分红是自动机制，不需要 DAO 逐笔投票。DAO 最多负责未来调整质押规则或协议参数。
            </div>
            <div className="rounded-lg border border-cyan-400/25 bg-cyan-400/5 px-4 py-3 text-xs font-mono leading-relaxed text-cyan-100">
              DAO 链上投票模块将在后续合约版本中开放。
            </div>

            <div className="flex flex-wrap items-center gap-2">
              {PROPOSAL_FLOW.map((status, index) => (
                <div key={status} className="flex items-center gap-2">
                  <span className="rounded border border-zinc-800 bg-zinc-950/80 px-2.5 py-1 text-[11px] font-mono font-bold text-zinc-300">
                    {status}
                  </span>
                  {index < PROPOSAL_FLOW.length - 1 && <ArrowRight className="h-3.5 w-3.5 text-zinc-700" />}
                </div>
              ))}
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {MOCK_DAO_PROPOSALS.map((proposal) => {
                const Icon = proposal.icon;
                return (
                  <article key={proposal.title} className={`rounded-lg border p-4 ${proposal.color}`}>
                    <div className="flex items-start gap-3">
                      <div className="rounded border border-current/30 bg-black/20 p-2">
                        <Icon className="w-4 h-4" />
                      </div>
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-2 mb-2">
                          <span className="rounded border border-current/30 bg-black/20 px-2 py-0.5 text-[10px] font-mono font-bold">
                            {proposal.type}
                          </span>
                          <span className="rounded border border-zinc-700 bg-zinc-950/70 px-2 py-0.5 text-[10px] font-mono font-bold text-zinc-300">
                            {proposal.status}
                          </span>
                        </div>
                        <h4 className="text-sm font-black text-zinc-100 font-mono">{proposal.title}</h4>
                        <button
                          type="button"
                          disabled
                          className="mt-3 w-full rounded border border-zinc-700 bg-zinc-900/70 px-3 py-2 text-xs font-mono font-bold text-zinc-500 disabled:cursor-not-allowed"
                        >
                          DAO 治理合约开发中
                        </button>
                      </div>
                    </div>
                  </article>
                );
              })}
            </div>
          </div>
        </div>

        <div className="flex gap-1 bg-zinc-950 p-1 rounded-xl border border-zinc-800 w-full max-w-xl">
          {[
            { label: '黑客曝光墙 / 受害者联署', i: 0 as const },
            { label: 'Alpha Green Label 绿标名录', i: 1 as const },
          ].map(({ label, i }) => (
            <button
              key={i}
              type="button"
              onClick={() => setActiveTab(i)}
              className={`flex-1 py-2.5 px-3 rounded-lg text-xs font-mono font-bold transition-all duration-200 ${
                activeTab === i
                  ? 'bg-green-500/15 text-green-400 border border-green-500/40'
                  : 'text-zinc-600 hover:text-zinc-400'
              }`}
            >
              {label}
            </button>
          ))}
        </div>

        {activeTab === 0 && (
          <div className="space-y-4">
            <div className="flex items-center gap-2 text-[10px] font-mono text-green-400/70 uppercase tracking-[0.2em]">
              <Zap className="w-3 h-3" />
              <span>Devnet Alpha 风险项目展示</span>
              <span className="ml-auto text-zinc-600 tabular-nums">{exposures.length} CASES</span>
            </div>

            <div className="hidden md:block border border-green-400/20 rounded-xl overflow-hidden bg-zinc-950/80 backdrop-blur-sm">
              <div className="overflow-x-auto">
                <table className="w-full text-sm font-mono">
                  <thead>
                    <tr className="border-b border-green-400/15 bg-green-400/5">
                      {['曝光项目', '事件时间', '涉及金额', '联署人数', '链', '状态', '操作'].map((col) => (
                        <th
                          key={col}
                          className="text-left px-4 py-3 text-green-400/80 uppercase tracking-widest text-[10px] font-bold whitespace-nowrap"
                        >
                          {col}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {exposures.map((row) => {
                      const joined = joinedIds.has(row.id);
                      const badge = statusBadge(row.status);
                      const isFeatured = row.id === 'proj-x';
                      return (
                        <tr
                          key={row.id}
                          className={`border-b border-zinc-800/60 transition-colors duration-150 ${
                            isFeatured ? 'bg-green-400/5 hover:bg-green-400/8' : 'hover:bg-zinc-900/50'
                          } ${joined ? 'ring-1 ring-inset ring-green-400/30' : ''}`}
                        >
                          <td className="px-4 py-3">
                            <span className="text-zinc-100 font-bold">{row.project}</span>
                            {isFeatured && (
                              <span className="ml-2 text-[9px] text-green-400 border border-green-400/40 px-1.5 py-0.5 rounded">
                                DEMO
                              </span>
                            )}
                          </td>
                          <td className="px-4 py-3 text-zinc-500">{row.rugDate}</td>
                          <td className="px-4 py-3 text-red-400 font-bold tabular-nums">{row.amountUsd}</td>
                          <td className="px-4 py-3">
                            <span className={`text-green-400 font-bold tabular-nums transition-all duration-300 ${joined ? 'scale-105 text-green-300' : ''}`}>
                              {row.coSigners.toLocaleString()} 人
                            </span>
                          </td>
                          <td className="px-4 py-3">
                            <span className="text-cyan-400 text-xs px-2 py-0.5 rounded border border-cyan-400/30 bg-cyan-400/5">
                              {row.chain}
                            </span>
                          </td>
                          <td className="px-4 py-3">
                            <span className={`text-[10px] px-2 py-0.5 rounded border font-bold ${badge.className}`}>
                              {badge.label}
                            </span>
                          </td>
                          <td className="px-4 py-3 min-w-[200px]">
                            <ExposureActions
                              joined={joined}
                              connected={connected}
                              onJoin={() => joinClassAction(row.id)}
                            />
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </div>

            <div className="md:hidden space-y-3">
              {exposures.map((row) => {
                const joined = joinedIds.has(row.id);
                const badge = statusBadge(row.status);
                return (
                  <div
                    key={row.id}
                    className={`border rounded-xl p-4 space-y-3 bg-zinc-950/80 ${
                      joined ? 'border-green-400/50 bg-green-400/5' : 'border-zinc-800'
                    }`}
                  >
                    <div className="flex items-start justify-between gap-2">
                      <p className="text-zinc-100 font-bold font-mono">{row.project}</p>
                      <span className={`text-[10px] px-2 py-0.5 rounded border ${badge.className}`}>{badge.label}</span>
                    </div>
                    <div className="grid grid-cols-2 gap-2 text-[11px] font-mono">
                      <div>
                        <p className="text-zinc-600">事件时间</p>
                        <p className="text-zinc-400">{row.rugDate}</p>
                      </div>
                      <div>
                        <p className="text-zinc-600">涉及金额</p>
                        <p className="text-red-400 font-bold">{row.amountUsd}</p>
                      </div>
                      <div>
                        <p className="text-zinc-600">联署人数</p>
                        <p className="text-green-400 font-bold tabular-nums">{row.coSigners.toLocaleString()}</p>
                      </div>
                      <div>
                        <p className="text-zinc-600">链</p>
                        <p className="text-cyan-400">{row.chain}</p>
                      </div>
                    </div>
                    <ExposureActions joined={joined} connected={connected} onJoin={() => joinClassAction(row.id)} />
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {activeTab === 1 && (
          <div className="border border-green-400/20 rounded-xl overflow-hidden bg-zinc-950/80">
            <div className="px-4 py-3 border-b border-green-400/15 bg-green-400/5 flex items-center gap-2">
              <BadgeCheck className="w-4 h-4 text-green-400" />
              <span className="text-green-400 font-mono text-xs font-bold uppercase tracking-widest">
                Alpha Green Label 绿标认证路线图
              </span>
            </div>
            <div className="p-4 space-y-4">
              <p className="text-zinc-400 font-mono text-xs leading-relaxed">
                项目方未来可申请 Alpha Protocol 绿标认证。绿标认证由 DAO 治理投票决定，当前显示为路线图功能。
              </p>
              <div className="flex flex-wrap items-center gap-2">
                {GREEN_LABEL_FLOW.map((step, index) => (
                  <div key={step} className="flex items-center gap-2">
                    <span className="rounded border border-zinc-800 bg-zinc-950 px-2.5 py-1 text-[11px] font-mono font-bold text-zinc-300">
                      {step}
                    </span>
                    {index < GREEN_LABEL_FLOW.length - 1 && <ArrowRight className="h-3.5 w-3.5 text-zinc-700" />}
                  </div>
                ))}
              </div>
            </div>
            <div className="overflow-x-auto border-t border-zinc-800">
              <table className="w-full text-sm font-mono">
                <thead>
                  <tr className="border-b border-zinc-800">
                    {['项目', '链', '贡献记录', '日期', '评级'].map((col) => (
                      <th key={col} className="text-left px-4 py-2 text-zinc-500 text-[10px] uppercase font-bold">
                        {col}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {certifiedData.map((row) => (
                    <tr key={row.name} className="border-b border-zinc-800/50 hover:bg-green-400/5">
                      <td className="px-4 py-3 text-zinc-200 font-bold">{row.name}</td>
                      <td className="px-4 py-3 text-cyan-400">{row.chain}</td>
                      <td className="px-4 py-3 text-green-400">{row.contribution} USDC</td>
                      <td className="px-4 py-3 text-zinc-500">{row.date}</td>
                      <td className="px-4 py-3 text-green-400">{row.ratingZh}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <div className="p-4 border-t border-zinc-800">
              <button
                type="button"
                disabled
                className="w-full py-2.5 rounded-lg border border-zinc-700 text-zinc-500 bg-zinc-900/70 cursor-not-allowed text-xs font-mono font-bold uppercase tracking-wider"
              >
                绿标认证模块开发中
              </button>
            </div>
          </div>
        )}

        <div className="border border-blue-400/20 bg-blue-400/5 rounded-xl overflow-hidden">
          <div className="px-5 py-4 border-b border-blue-400/10 bg-zinc-950/70 flex items-center gap-2">
            <Users className="w-4 h-4 text-blue-400" />
            <div>
              <h3 className="text-zinc-100 font-mono text-sm font-black uppercase tracking-widest">
                Community Builders / 社区建设者
              </h3>
              <p className="text-zinc-500 font-mono text-[10px] mt-0.5">
                DAO 贡献者 / 生态建设池来自协议收入的 20%，用于支持 Alpha Protocol 的长期建设。
              </p>
            </div>
          </div>
          <div className="p-5 space-y-4">
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
              {BUILDER_USES.map((item) => (
                <div key={item} className="rounded-lg border border-zinc-800 bg-zinc-950/70 px-3 py-2 text-xs font-mono font-bold text-zinc-300">
                  {item}
                </div>
              ))}
            </div>

            <div className="rounded-lg border border-zinc-800 bg-zinc-950/70 p-4">
              <p className="mb-3 text-zinc-100 font-mono text-xs font-black uppercase tracking-widest">
                未来共建流程
              </p>
              <div className="flex flex-wrap items-center gap-2">
                {BUILDER_FLOW.map((step, index) => (
                  <div key={step} className="flex items-center gap-2">
                    <span className="rounded border border-zinc-800 bg-zinc-950 px-2.5 py-1 text-[11px] font-mono font-bold text-zinc-300">
                      {step}
                    </span>
                    {index < BUILDER_FLOW.length - 1 && <ArrowRight className="h-3.5 w-3.5 text-zinc-700" />}
                  </div>
                ))}
              </div>
            </div>

            <button
              type="button"
              disabled
              className="w-full md:w-auto rounded border border-zinc-700 bg-zinc-900/70 px-4 py-2 text-xs font-mono font-bold text-zinc-500 disabled:cursor-not-allowed"
            >
              社区共建入口即将开放
            </button>

            <div className="rounded-lg border border-blue-400/25 bg-blue-400/5 px-4 py-3 text-xs font-mono leading-relaxed text-blue-100">
              Alpha Protocol 将采用渐进式去中心化路线，早期由创始团队完成基础框架，随后通过 DAO 治理和贡献者激励池逐步开放社区共建。
            </div>
          </div>
        </div>
      </div>

      {showReportForm && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center p-4"
          onClick={(e) => {
            if (e.target === e.currentTarget) setShowReportForm(false);
          }}
        >
          <div className="absolute inset-0 bg-black/85 backdrop-blur-sm" />
          <div className="relative w-full max-w-md border border-green-400/40 bg-zinc-950 rounded-2xl overflow-hidden shadow-2xl shadow-green-400/10">
            <div className="h-1 w-full bg-gradient-to-r from-green-600 via-green-400 to-cyan-400" />
            <div className="p-6 space-y-4">
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-2">
                  <FileText className="w-5 h-5 text-green-400" />
                  <h3 className="text-green-400 font-mono font-bold uppercase tracking-wider text-sm">
                    发起风险项目曝光
                  </h3>
                </div>
                <button type="button" onClick={() => setShowReportForm(false)} className="text-zinc-600 hover:text-zinc-300">
                  <X className="w-5 h-5" />
                </button>
              </div>
              <p className="text-zinc-500 font-mono text-[11px] leading-relaxed">
                当前表单仅保存到前端演示列表。真实链上 DAO 提案提交将在后续合约版本中开放。
              </p>
              <input
                type="text"
                value={newProject}
                onChange={(e) => setNewProject(e.target.value)}
                placeholder="项目名称"
                className="w-full bg-zinc-950 border border-green-400/25 rounded-lg px-3 py-2.5 text-sm font-mono text-zinc-200 focus:outline-none focus:border-green-400/60"
              />
              <div className="grid grid-cols-2 gap-3">
                <input
                  type="text"
                  value={newLoss}
                  onChange={(e) => setNewLoss(e.target.value)}
                  placeholder="损失金额"
                  className="w-full bg-zinc-950 border border-green-400/25 rounded-lg px-3 py-2.5 text-sm font-mono text-zinc-200 focus:outline-none focus:border-green-400/60"
                />
                <input
                  type="text"
                  value={newChain}
                  onChange={(e) => setNewChain(e.target.value)}
                  placeholder="链（SOL/ETH）"
                  className="w-full bg-zinc-950 border border-green-400/25 rounded-lg px-3 py-2.5 text-sm font-mono text-zinc-200 focus:outline-none focus:border-green-400/60"
                />
              </div>
              <button
                type="button"
                onClick={submitNewFraud}
                disabled={!newProject.trim()}
                className="w-full py-3 bg-green-500/20 hover:bg-green-500/30 border border-green-500/50 text-green-400 font-mono font-bold uppercase tracking-wider rounded-lg text-sm transition-all disabled:cursor-not-allowed disabled:opacity-50"
              >
                加入演示列表
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

function ExposureActions({
  joined,
  connected,
  onJoin,
}: {
  joined: boolean;
  connected: boolean;
  onJoin: () => void;
}) {
  return (
    <div className="space-y-2">
      <div className="flex flex-wrap items-center gap-2">
        <button
          type="button"
          onClick={onJoin}
          disabled={!connected || joined}
          className={`inline-flex items-center gap-1.5 text-xs px-3 py-1.5 rounded-lg border font-bold transition-all duration-200 ${
            joined
              ? 'border-green-400/60 bg-green-400/20 text-green-300 cursor-default'
              : connected
                ? 'border-green-500/50 bg-green-500/10 text-green-400 hover:bg-green-500/20 hover:border-green-500/70'
                : 'border-zinc-700 text-zinc-500 cursor-not-allowed opacity-50'
          }`}
        >
          <Users className="w-3 h-3" />
          {joined ? '已联署' : connected ? '参与受害者联署' : '请先连接钱包'}
        </button>
        <button
          type="button"
          disabled
          className="inline-flex items-center gap-1 text-xs text-zinc-500 cursor-not-allowed"
        >
          <ExternalLink className="w-3 h-3" />
          证据入口开发中
        </button>
      </div>
      {joined && (
        <p className="text-[10px] font-mono text-green-400 leading-snug flex items-start gap-1 animate-pulse">
          <Shield className="w-3 h-3 flex-shrink-0 mt-0.5" />
          当前钱包已加入 Devnet Alpha 演示联署列表，真实链上提案将在后续开放。
        </p>
      )}
    </div>
  );
}
