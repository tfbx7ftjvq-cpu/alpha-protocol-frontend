import { useEffect, useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import {
  BadgeCheck,
  ExternalLink,
  FileText,
  Gavel,
  PlusCircle,
  Shield,
  Users,
  Wallet,
  X,
  Zap,
} from 'lucide-react';
import { t, Lang } from '../translations';

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
  { name: 'SolNexus Matrix', chain: 'SOL', contribution: '2,500', date: '2026-05-20', rating: 'AAA', ratingZh: '极佳' },
  { name: 'Aegis Liquidity', chain: 'SOL', contribution: '1,800', date: '2026-05-24', rating: 'AA', ratingZh: '良好' },
];

function statusBadge(status: ExposureProject['status'], zh: boolean) {
  const map = {
    exposed: { label: zh ? '已曝光' : 'EXPOSED', class: 'text-red-400 border-red-400/40 bg-red-400/10' },
    hearing: { label: zh ? '听证中' : 'HEARING', class: 'text-yellow-400 border-yellow-400/40 bg-yellow-400/10' },
    dao_pending: { label: zh ? '待 DAO 表决' : 'DAO PENDING', class: 'text-green-400 border-green-400/40 bg-green-400/10' },
  };
  return map[status];
}

export default function WallOfShame({ lang }: Props) {
  const tr = t[lang];
  const zh = lang === 'zh';
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

  const connectReportHint = zh
    ? '连接钱包后方可发起链上欺诈归集提案'
    : 'Connect wallet to submit on-chain fraud aggregation proposal';

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
    if (!connected || !newProject.trim()) return;
    const id = `custom-${Date.now()}`;
    setExposures((prev) => [
      {
        id,
        project: newProject.trim(),
        rugDate: new Date().toISOString().slice(0, 7),
        amountUsd: newLoss.trim() ? `$${newLoss.trim()}` : '$0',
        coSigners: 1,
        chain: newChain,
        status: 'exposed',
      },
      ...prev,
    ]);
    if (connected) setJoinedIds((prev) => new Set(prev).add(id));
    setNewProject('');
    setNewLoss('');
    setShowReportForm(false);
  }

  return (
    <>
      <div className="space-y-6">
        {/* Header row */}
        <div className="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-4">
          <div className="flex items-center gap-3 min-w-0">
            <Gavel className="w-6 h-6 text-green-400 flex-shrink-0" />
            <div>
              <h2 className="text-xl font-black text-zinc-100 font-mono tracking-wide uppercase">
                {zh ? '链上欺诈弹劾归集面板' : 'On-Chain Fraud Impeachment Registry'}
              </h2>
              <p className="text-zinc-600 font-mono text-[10px] mt-0.5 uppercase tracking-widest">
                WALL_OF_SHAME · CLASS_ACTION_AGGREGATOR
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
            {!connected ? connectReportHint : zh ? '发起新项目欺诈清算' : 'Initiate Fraud Liquidation'}
          </button>
        </div>

        {/* Wallet strip */}
        <div
          className={`flex items-center gap-2 px-4 py-2.5 rounded-lg border font-mono text-xs transition-colors ${
            connected ? 'border-green-400/30 bg-green-400/5 text-green-400' : 'border-zinc-800 bg-zinc-950 text-zinc-500'
          }`}
        >
          <Wallet className="w-3.5 h-3.5 flex-shrink-0" />
          {connected && shortAddress ? (
            <span>
              {zh ? '链上身份' : 'On-chain ID'}: <span className="font-bold tabular-nums">{shortAddress}</span>
            </span>
          ) : (
            <span>{connectReportHint}</span>
          )}
        </div>

        {/* Global notice */}
        <div className="border border-green-400/30 bg-green-400/5 rounded-xl px-4 py-3 relative overflow-hidden">
          <div className="absolute top-0 left-0 w-1 h-full bg-green-400/60" />
          <p className="text-green-400 font-mono text-xs leading-relaxed pl-2">
            ⚖️{' '}
            {zh
              ? '互助法庭前哨：此墙所列之欺诈项目，一旦通过社区联署并经 DAO 投票表决（10% 常驻治理），将自动激活 50% 国库公池实施精准定向救济分发。'
              : 'Mutual Court Outpost: fraud listings here, once co-signed and passed by DAO vote (10% governance), trigger 50% treasury pool for targeted victim relief.'}
          </p>
        </div>

        {/* Tabs */}
        <div className="flex gap-1 bg-zinc-950 p-1 rounded-xl border border-zinc-800 w-full max-w-xl">
          {[
            { label: zh ? '🔴 黑客曝光墙' : '🔴 Exposure Wall', i: 0 as const },
            { label: zh ? '🟢 绿标合规名录' : '🟢 Certified', i: 1 as const },
          ].map(({ label, i }) => (
            <button
              key={i}
              type="button"
              onClick={() => setActiveTab(i)}
              className={`flex-1 py-2.5 px-3 rounded-lg text-xs font-mono font-bold transition-all duration-200 ${
                activeTab === i
                  ? i === 0
                    ? 'bg-green-500/15 text-green-400 border border-green-500/40'
                    : 'bg-green-500/15 text-green-400 border border-green-500/40'
                  : 'text-zinc-600 hover:text-zinc-400'
              }`}
            >
              {label}
            </button>
          ))}
        </div>

        {/* Tab 0: Exposure Wall */}
        {activeTab === 0 && (
          <div className="space-y-4">
            {/* Matrix scanline header */}
            <div className="flex items-center gap-2 text-[10px] font-mono text-green-400/70 uppercase tracking-[0.2em]">
              <Zap className="w-3 h-3 animate-pulse" />
              <span>{zh ? '实时链上证据归集流' : 'LIVE ON-CHAIN EVIDENCE STREAM'}</span>
              <span className="ml-auto text-zinc-600 tabular-nums">{exposures.length} NODES</span>
            </div>

            {/* Desktop table */}
            <div className="hidden md:block border border-green-400/20 rounded-xl overflow-hidden bg-zinc-950/80 backdrop-blur-sm">
              <div className="overflow-x-auto">
                <table className="w-full text-sm font-mono">
                  <thead>
                    <tr className="border-b border-green-400/15 bg-green-400/5">
                      {(zh
                        ? ['曝光项目', 'Rug 时间', '涉案金额', '联署人数', '链', '状态', '操作']
                        : ['Project', 'Rug Date', 'Amount', 'Co-signers', 'Chain', 'Status', 'Action']
                      ).map((col) => (
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
                      const badge = statusBadge(row.status, zh);
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
                                DAO 003
                              </span>
                            )}
                          </td>
                          <td className="px-4 py-3 text-zinc-500">{row.rugDate}</td>
                          <td className="px-4 py-3 text-red-400 font-bold tabular-nums">{row.amountUsd}</td>
                          <td className="px-4 py-3">
                            <span
                              className={`text-green-400 font-bold tabular-nums transition-all duration-300 ${
                                joined ? 'scale-105 text-green-300' : ''
                              }`}
                            >
                              {row.coSigners.toLocaleString()}
                              {zh ? ' 人' : ''}
                            </span>
                          </td>
                          <td className="px-4 py-3">
                            <span className="text-cyan-400 text-xs px-2 py-0.5 rounded border border-cyan-400/30 bg-cyan-400/5">
                              {row.chain}
                            </span>
                          </td>
                          <td className="px-4 py-3">
                            <span className={`text-[10px] px-2 py-0.5 rounded border font-bold ${badge.class}`}>
                              {badge.label}
                            </span>
                          </td>
                          <td className="px-4 py-3 min-w-[200px]">
                            <ExposureActions
                              joined={joined}
                              connected={connected}
                              connectHint={zh ? '请先连接钱包' : 'Connect wallet'}
                              zh={zh}
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

            {/* Mobile cards */}
            <div className="md:hidden space-y-3">
              {exposures.map((row) => {
                const joined = joinedIds.has(row.id);
                const badge = statusBadge(row.status, zh);
                return (
                  <div
                    key={row.id}
                    className={`border rounded-xl p-4 space-y-3 bg-zinc-950/80 ${
                      joined ? 'border-green-400/50 bg-green-400/5' : 'border-zinc-800'
                    }`}
                  >
                    <div className="flex items-start justify-between gap-2">
                      <p className="text-zinc-100 font-bold font-mono">{row.project}</p>
                      <span className={`text-[10px] px-2 py-0.5 rounded border ${badge.class}`}>{badge.label}</span>
                    </div>
                    <div className="grid grid-cols-2 gap-2 text-[11px] font-mono">
                      <div>
                        <p className="text-zinc-600">{zh ? 'Rug 时间' : 'Rug'}</p>
                        <p className="text-zinc-400">{row.rugDate}</p>
                      </div>
                      <div>
                        <p className="text-zinc-600">{zh ? '涉案金额' : 'Loss'}</p>
                        <p className="text-red-400 font-bold">{row.amountUsd}</p>
                      </div>
                      <div>
                        <p className="text-zinc-600">{zh ? '联署' : 'Co-signers'}</p>
                        <p className="text-green-400 font-bold tabular-nums">{row.coSigners.toLocaleString()}</p>
                      </div>
                      <div>
                        <p className="text-zinc-600">{zh ? '链' : 'Chain'}</p>
                        <p className="text-cyan-400">{row.chain}</p>
                      </div>
                    </div>
                    <ExposureActions
                      joined={joined}
                      connected={connected}
                      connectHint={zh ? '请先连接钱包' : 'Connect wallet'}
                      zh={zh}
                      onJoin={() => joinClassAction(row.id)}
                    />
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* Tab 1: Green registry (compact) */}
        {activeTab === 1 && (
          <div className="border border-green-400/20 rounded-xl overflow-hidden bg-zinc-950/80">
            <div className="px-4 py-3 border-b border-green-400/15 bg-green-400/5 flex items-center gap-2">
              <BadgeCheck className="w-4 h-4 text-green-400" />
              <span className="text-green-400 font-mono text-xs font-bold uppercase tracking-widest">
                {zh ? '绿标合规名录' : 'Certified Whitelist'}
              </span>
            </div>
            <div className="overflow-x-auto">
              <table className="w-full text-sm font-mono">
                <thead>
                  <tr className="border-b border-zinc-800">
                    {(zh ? ['项目', '链', '贡献', '评级'] : ['Project', 'Chain', 'Fee', 'Rating']).map((col) => (
                      <th key={col} className="text-left px-4 py-2 text-zinc-500 text-[10px] uppercase font-bold">
                        {col}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {certifiedData.map((row, i) => (
                    <tr key={i} className="border-b border-zinc-800/50 hover:bg-green-400/5">
                      <td className="px-4 py-3 text-zinc-200 font-bold">
                        {row.name}
                      </td>
                      <td className="px-4 py-3 text-cyan-400">{row.chain}</td>
                      <td className="px-4 py-3 text-green-400">{row.contribution} USDC</td>
                      <td className="px-4 py-3 text-green-400">{zh ? row.ratingZh : row.rating}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <div className="p-4 border-t border-zinc-800">
              <button
                type="button"
                disabled={!connected}
                className={`w-full py-2.5 rounded-lg border text-xs font-mono font-bold uppercase tracking-wider transition-all ${
                  connected
                    ? 'border-green-500/50 text-green-400 bg-green-500/10 hover:bg-green-500/20'
                    : 'border-zinc-700 text-zinc-500 cursor-not-allowed opacity-50'
                }`}
              >
                {connected ? tr.card1Btn : connectReportHint}
              </button>
            </div>
          </div>
        )}
      </div>

      {/* New fraud report modal */}
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
                    {zh ? '发起链上欺诈归集提案' : 'Submit Fraud Aggregation'}
                  </h3>
                </div>
                <button type="button" onClick={() => setShowReportForm(false)} className="text-zinc-600 hover:text-zinc-300">
                  <X className="w-5 h-5" />
                </button>
              </div>
              <p className="text-zinc-500 font-mono text-[11px] leading-relaxed">{tr.reportDesc}</p>
              <input
                type="text"
                value={newProject}
                onChange={(e) => setNewProject(e.target.value)}
                placeholder={tr.projectPlaceholder}
                className="w-full bg-zinc-950 border border-green-400/25 rounded-lg px-3 py-2.5 text-sm font-mono text-zinc-200 focus:outline-none focus:border-green-400/60"
              />
              <div className="grid grid-cols-2 gap-3">
                <input
                  type="text"
                  value={newLoss}
                  onChange={(e) => setNewLoss(e.target.value)}
                  placeholder={tr.lossPlaceholder}
                  className="w-full bg-zinc-950 border border-green-400/25 rounded-lg px-3 py-2.5 text-sm font-mono text-zinc-200 focus:outline-none focus:border-green-400/60"
                />
                <input
                  type="text"
                  value={newChain}
                  onChange={(e) => setNewChain(e.target.value)}
                  placeholder={zh ? '链 (SOL/ETH)' : 'Chain'}
                  className="w-full bg-zinc-950 border border-green-400/25 rounded-lg px-3 py-2.5 text-sm font-mono text-zinc-200 focus:outline-none focus:border-green-400/60"
                />
              </div>
              <button
                type="button"
                onClick={submitNewFraud}
                disabled={!newProject.trim()}
                className="w-full py-3 bg-green-500/20 hover:bg-green-500/30 border border-green-500/50 text-green-400 font-mono font-bold uppercase tracking-wider rounded-lg text-sm transition-all"
              >
                {tr.reportBtn}
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
  connectHint,
  zh,
  onJoin,
}: {
  joined: boolean;
  connected: boolean;
  connectHint: string;
  zh: boolean;
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
          {joined
            ? zh
              ? '已联署'
              : 'Joined'
            : !connected
              ? connectHint
              : zh
                ? '参与受害者联署'
                : 'Join Class Action'}
        </button>
        <button
          type="button"
          className="inline-flex items-center gap-1 text-xs text-zinc-500 hover:text-cyan-400 transition-colors"
        >
          <ExternalLink className="w-3 h-3" />
          {zh ? '证据' : 'Evidence'}
        </button>
      </div>
      {joined && (
        <p className="text-[10px] font-mono text-green-400 leading-snug flex items-start gap-1 animate-pulse">
          <Shield className="w-3 h-3 flex-shrink-0 mt-0.5" />
          {zh ? '已用当前钱包地址参与链上证据留存' : 'Wallet recorded on-chain for evidence co-signing'}
        </p>
      )}
    </div>
  );
}
