import { useState, useEffect, useRef } from 'react';
import { AlertTriangle, CheckCircle, Clock, ExternalLink, Shield, Gavel, FileText, ChevronRight, X, Users, Timer, ThumbsUp, BadgeCheck, ArrowUpRight } from 'lucide-react';
import { t, Lang } from '../translations';

interface Props {
  lang: Lang;
}

const certifiedData = [
  { name: 'SolNexus Matrix',    chain: 'SOL', contribution: '2,500', date: '2026-05-20', rating: 'AAA', ratingZh: '极佳' },
  { name: 'Aegis Liquidity',    chain: 'SOL', contribution: '1,800', date: '2026-05-24', rating: 'AA',  ratingZh: '良好' },
  { name: 'ClearVault Protocol', chain: 'SOL', contribution: '3,200', date: '2026-05-18', rating: 'AAA', ratingZh: '极佳' },
  { name: 'ProofStake Finance', chain: 'SOL', contribution: '950',   date: '2026-05-22', rating: 'A',   ratingZh: '合格' },
];

const fraudData = [
  { project: 'ShadowSwap Finance', chain: 'ETH', loss: '2,340,000', date: '2024-01-15', statusKey: 'statusFlagged' as const },
  { project: 'MoonRug Protocol', chain: 'BSC', loss: '890,000', date: '2024-02-08', statusKey: 'statusHearing' as const },
  { project: 'InvisibleVault DAO', chain: 'SOL', loss: '4,100,000', date: '2024-03-22', statusKey: 'statusConvicted' as const },
  { project: 'GhostLiquidity', chain: 'ARB', loss: '560,000', date: '2024-04-01', statusKey: 'statusVerified' as const },
  { project: 'DarkMatter Exchange', chain: 'ETH', loss: '1,230,000', date: '2024-04-18', statusKey: 'statusFlagged' as const },
  { project: 'PhantomYield', chain: 'MATIC', loss: '330,000', date: '2024-05-02', statusKey: 'statusConvicted' as const },
];

interface ImpeachState {
  project: string;
  votes: number;
  success: boolean;
  animating: boolean;
}

export default function WallOfShame({ lang }: Props) {
  const tr = t[lang];
  const [activeTab, setActiveTab] = useState<0 | 1 | 2>(0);
  const [projectInput, setProjectInput] = useState('');
  const [lossInput, setLossInput] = useState('');
  const [impeachModal, setImpeachModal] = useState<ImpeachState | null>(null);
  const rafRef = useRef<number>(0);

  // Lock body scroll when modal open
  useEffect(() => {
    document.body.style.overflow = impeachModal ? 'hidden' : '';
    return () => { document.body.style.overflow = ''; };
  }, [!!impeachModal]);

  function closeModal() {
    cancelAnimationFrame(rafRef.current);
    setImpeachModal(null);
  }

  function openImpeach(project: string) {
    setImpeachModal({ project, votes: 61.4, success: false, animating: false });
  }

  function castVote() {
    if (!impeachModal || impeachModal.animating || impeachModal.success) return;
    setImpeachModal((prev) => prev ? { ...prev, animating: true } : prev);
    const start = 61.4;
    const end = 66.5;
    const duration = 1200;
    const startTime = performance.now();
    function step(now: number) {
      const progress = Math.min((now - startTime) / duration, 1);
      const eased = 1 - Math.pow(1 - progress, 3);
      const current = start + (end - start) * eased;
      setImpeachModal((prev) => prev ? { ...prev, votes: parseFloat(current.toFixed(1)) } : prev);
      if (progress < 1) {
        rafRef.current = requestAnimationFrame(step);
      } else {
        setImpeachModal((prev) => prev ? { ...prev, votes: end, success: true, animating: false } : prev);
      }
    }
    rafRef.current = requestAnimationFrame(step);
  }

  const statusConfig = {
    statusFlagged: { labelKey: 'statusFlagged' as const, color: 'text-red-400', border: 'border-red-400/40', bg: 'bg-red-400/10', icon: AlertTriangle },
    statusVerified: { labelKey: 'statusVerified' as const, color: 'text-green-400', border: 'border-green-400/40', bg: 'bg-green-400/10', icon: CheckCircle },
    statusHearing: { labelKey: 'statusHearing' as const, color: 'text-yellow-400', border: 'border-yellow-400/40', bg: 'bg-yellow-400/10', icon: Clock },
    statusConvicted: { labelKey: 'statusConvicted' as const, color: 'text-orange-400', border: 'border-orange-400/40', bg: 'bg-orange-400/10', icon: Gavel },
  };

  const requiredPct = 66;

  return (
    <>
      <div className="space-y-6">
        {/* Tabs */}
        <div className="flex gap-1 bg-gray-900/80 p-1 rounded-xl border border-gray-700/50 w-full">
          {[
            { label: tr.tab1, activeColor: 'bg-red-500/20 text-red-400 border border-red-500/40' },
            { label: lang === 'zh' ? '🟢 绿标合规名录' : '🟢 Certified Whitelist', activeColor: 'bg-green-500/20 text-green-400 border border-green-500/40' },
            { label: tr.tab2, activeColor: 'bg-yellow-500/20 text-yellow-400 border border-yellow-500/40' },
          ].map(({ label, activeColor }, i) => (
            <button
              key={i}
              onClick={() => setActiveTab(i as 0 | 1 | 2)}
              className={`flex-1 py-2.5 px-3 rounded-lg text-xs font-mono font-bold transition-all duration-200 ${
                activeTab === i ? activeColor : 'text-gray-500 hover:text-gray-300'
              }`}
            >
              {label}
            </button>
          ))}
        </div>

        {/* Tab 0: Wall of Shame */}
        {activeTab === 0 && (
          <div className="space-y-6">
            {/* Table */}
            <div className="border border-red-400/20 rounded-xl overflow-hidden bg-gray-900/40 backdrop-blur-sm">
              <div className="overflow-x-auto">
                <table className="w-full text-sm font-mono">
                  <thead>
                    <tr className="border-b border-red-400/20 bg-red-400/5">
                      {[tr.colProject, tr.colChain, tr.colLoss, tr.colDate, tr.colStatus, tr.colAction].map((col) => (
                        <th key={col} className="text-left px-4 py-3 text-red-400 uppercase tracking-widest text-xs font-bold">
                          {col}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {fraudData.map((row, i) => {
                      const sc = statusConfig[row.statusKey];
                      const Icon = sc.icon;
                      const isConvicted = row.statusKey === 'statusConvicted';
                      return (
                        <tr
                          key={i}
                          className="border-b border-gray-800/60 hover:bg-red-400/5 transition-colors duration-150 group"
                        >
                          <td className="px-4 py-3 text-white font-bold">{row.project}</td>
                          <td className="px-4 py-3 text-cyan-400">{row.chain}</td>
                          <td className="px-4 py-3 text-red-400 font-bold">{row.loss} USDC</td>
                          <td className="px-4 py-3 text-gray-500">{row.date}</td>
                          <td className="px-4 py-3">
                            <span className={`inline-flex items-center gap-1.5 px-2 py-1 rounded border text-xs ${sc.color} ${sc.border} ${sc.bg}`}>
                              <Icon className="w-3 h-3" />
                              {tr[sc.labelKey]}
                            </span>
                          </td>
                          <td className="px-4 py-3">
                            <div className="flex items-center gap-2 flex-wrap">
                              <button className="inline-flex items-center gap-1 text-xs text-gray-500 hover:text-cyan-400 transition-colors">
                                <ExternalLink className="w-3 h-3" />
                                {tr.viewEvidence}
                              </button>
                              {isConvicted && (
                                <button
                                  onClick={() => openImpeach(row.project)}
                                  className="inline-flex items-center gap-1 text-xs px-2 py-1 rounded border border-orange-400/50 text-orange-400 bg-orange-400/10 hover:bg-orange-400/20 transition-all duration-150 font-bold whitespace-nowrap"
                                >
                                  <Gavel className="w-3 h-3" />
                                  {tr.impeachBtn}
                                </button>
                              )}
                            </div>
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </div>

            {/* Submit report */}
            <div className="border border-red-400/20 bg-red-400/5 rounded-xl p-6 space-y-4">
              <div className="flex items-center gap-2 mb-2">
                <FileText className="text-red-400 w-5 h-5" />
                <h3 className="text-red-400 font-mono font-bold uppercase tracking-wider">{tr.submitReport}</h3>
              </div>
              <p className="text-gray-500 text-xs font-mono">{tr.reportDesc}</p>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <input
                  type="text"
                  value={projectInput}
                  onChange={(e) => setProjectInput(e.target.value)}
                  placeholder={tr.projectPlaceholder}
                  className="bg-gray-900/80 border border-red-400/20 rounded-lg px-3 py-2.5 text-sm font-mono text-gray-300 placeholder-gray-600 focus:outline-none focus:border-red-400/60 transition-colors"
                />
                <input
                  type="text"
                  value={lossInput}
                  onChange={(e) => setLossInput(e.target.value)}
                  placeholder={tr.lossPlaceholder}
                  className="bg-gray-900/80 border border-red-400/20 rounded-lg px-3 py-2.5 text-sm font-mono text-gray-300 placeholder-gray-600 focus:outline-none focus:border-red-400/60 transition-colors"
                />
              </div>
              <button className="w-full py-2.5 bg-red-500/20 hover:bg-red-500/30 border border-red-500/50 text-red-400 font-mono font-bold uppercase tracking-wider rounded-lg transition-all duration-200 hover:shadow-red-500/20 hover:shadow-lg text-sm">
                {tr.reportBtn}
              </button>
            </div>
          </div>
        )}

        {/* Tab 1: Green-Label Compliance Registry */}
        {activeTab === 1 && (
          <div className="space-y-5">
            {/* Eco loop info banner */}
            <div className="flex items-start gap-3 border border-green-400/20 bg-green-400/5 rounded-xl px-4 py-3 relative overflow-hidden">
              <div className="absolute top-0 left-0 w-1 h-full bg-green-400/50 rounded-l-xl" />
              <BadgeCheck className="w-4 h-4 text-green-400 flex-shrink-0 mt-0.5" />
              <p className="text-green-300/80 font-mono text-[11px] leading-relaxed pl-1">
                <span className="text-green-400 font-bold">[AUDIT-INFO]</span>
                {lang === 'zh'
                  ? '：绿标合规项目贡献的 100% 服务费已单向路由至协议国库：80% 沉淀至散户回购与救济池，20% 动态流向社区 DAO 投票锁定的核心技术与运营地址。'
                  : ': 100% of service fees contributed by green-label certified projects are routed one-way to the protocol treasury: 80% flows into the retail buyback & relief pool, 20% dynamically routes to the DAO-governance-locked core ops addresses.'}
              </p>
            </div>

            {/* Certified projects table */}
            <div className="border border-green-400/20 rounded-xl overflow-hidden bg-gray-900/40 backdrop-blur-sm">
              <div className="px-4 py-3 border-b border-green-400/15 bg-green-400/5 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <BadgeCheck className="w-4 h-4 text-green-400" />
                  <span className="text-green-400 font-mono text-xs font-bold uppercase tracking-widest">
                    {lang === 'zh' ? '绿标合规名录' : 'Green-Label Compliance Registry'}
                  </span>
                </div>
                <span className="text-green-500/60 font-mono text-[10px] border border-green-400/20 px-2 py-0.5 rounded-full">
                  {certifiedData.length} {lang === 'zh' ? '个认证项目' : 'Certified Projects'}
                </span>
              </div>
              <div className="overflow-x-auto">
                <table className="w-full text-sm font-mono">
                  <thead>
                    <tr className="border-b border-green-400/10 bg-green-400/5">
                      {(lang === 'zh'
                        ? ['项目名称', '生态链', '国库造血服务费贡献', '审计通过日期', '合规评级', '安全证明']
                        : ['Project', 'Chain', 'Treasury Fee Contributed', 'Audit Date', 'Compliance Rating', 'Proof']
                      ).map((col) => (
                        <th key={col} className="text-left px-4 py-3 text-green-500 uppercase tracking-widest text-xs font-bold whitespace-nowrap">
                          {col}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {certifiedData.map((row, i) => {
                      const isAAA = row.rating === 'AAA';
                      const isAA  = row.rating === 'AA';
                      const ratingColor = isAAA ? 'text-green-400 border-green-400/50 bg-green-400/10'
                        : isAA ? 'text-cyan-400 border-cyan-400/40 bg-cyan-400/8'
                        : 'text-yellow-400 border-yellow-400/40 bg-yellow-400/8';
                      return (
                        <tr key={i} className="border-b border-gray-800/50 hover:bg-green-400/5 transition-colors duration-150">
                          <td className="px-4 py-3 text-white font-bold whitespace-nowrap">{row.name}</td>
                          <td className="px-4 py-3">
                            <span className="text-cyan-400 font-bold text-xs px-2 py-0.5 rounded border border-cyan-400/30 bg-cyan-400/8">{row.chain}</span>
                          </td>
                          <td className="px-4 py-3 text-green-400 font-bold whitespace-nowrap">{row.contribution} USDC</td>
                          <td className="px-4 py-3 text-gray-500 whitespace-nowrap">{row.date}</td>
                          <td className="px-4 py-3">
                            <span className={`inline-flex items-center gap-1.5 px-2 py-1 rounded border text-xs font-bold whitespace-nowrap ${ratingColor}`}>
                              <span>🟢</span>
                              {lang === 'zh' ? row.ratingZh : row.rating} ({row.rating})
                            </span>
                          </td>
                          <td className="px-4 py-3">
                            <button className="inline-flex items-center gap-1.5 text-xs px-2.5 py-1.5 rounded border border-green-400/40 text-green-400 bg-green-400/8 hover:bg-green-400/15 transition-all duration-150 font-bold whitespace-nowrap">
                              <ArrowUpRight className="w-3 h-3" />
                              {lang === 'zh' ? '查看审计报告' : 'IPFS Proof'}
                            </button>
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </div>

            {/* Apply for green label CTA */}
            <div className="border border-green-400/20 bg-green-400/5 rounded-xl p-5 flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
              <div className="flex items-center gap-3">
                <div className="p-2.5 rounded-xl bg-green-400/10 border border-green-400/30 flex-shrink-0">
                  <Shield className="w-5 h-5 text-green-400" />
                </div>
                <div>
                  <p className="text-green-400 font-mono font-bold text-sm">{tr.card1Title}</p>
                  <p className="text-green-500/60 font-mono text-xs mt-0.5">{tr.card1Fee}</p>
                </div>
              </div>
              <button className="flex-shrink-0 inline-flex items-center gap-2 py-2.5 px-5 bg-green-500/20 hover:bg-green-500/30 border border-green-500/50 text-green-400 font-mono font-bold rounded-lg transition-all duration-200 hover:shadow-green-500/20 hover:shadow-lg text-sm whitespace-nowrap">
                <CheckCircle className="w-4 h-4" />
                {tr.card1Btn}
                <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* Tab 2: Service & Application Details */}
        {activeTab === 2 && (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* Card 1 */}
            <div className="border border-green-400/30 bg-green-400/5 rounded-xl p-6 space-y-4 relative overflow-hidden group hover:border-green-400/60 transition-all duration-300">
              <div className="absolute top-0 right-0 w-24 h-24 bg-green-400/5 rounded-bl-full" />
              <div className="flex items-center gap-3">
                <div className="p-2.5 rounded-xl bg-green-400/10 border border-green-400/30">
                  <Shield className="w-6 h-6 text-green-400" />
                </div>
                <div>
                  <h3 className="text-green-400 font-mono font-bold text-base">{tr.card1Title}</h3>
                  <p className="text-green-500/60 text-xs font-mono">{tr.card1Fee}</p>
                </div>
              </div>
              <p className="text-gray-400 text-sm font-mono leading-relaxed">{tr.card1Desc}</p>
              <button className="w-full py-3 bg-green-500/20 hover:bg-green-500/30 border border-green-500/50 text-green-400 font-mono font-bold rounded-lg transition-all duration-200 hover:shadow-green-500/20 hover:shadow-lg flex items-center justify-center gap-2 text-sm">
                <CheckCircle className="w-4 h-4" />
                {tr.card1Btn}
                <ChevronRight className="w-4 h-4" />
              </button>
            </div>

            {/* Card 2 */}
            <div className="border border-yellow-400/30 bg-yellow-400/5 rounded-xl p-6 space-y-4 relative overflow-hidden group hover:border-yellow-400/60 transition-all duration-300">
              <div className="absolute top-0 right-0 w-24 h-24 bg-yellow-400/5 rounded-bl-full" />
              <div className="flex items-center gap-3">
                <div className="p-2.5 rounded-xl bg-yellow-400/10 border border-yellow-400/30">
                  <Gavel className="w-6 h-6 text-yellow-400" />
                </div>
                <div>
                  <h3 className="text-yellow-400 font-mono font-bold text-base">{tr.card2Title}</h3>
                  <p className="text-yellow-500/60 text-xs font-mono">{tr.card2Fee}</p>
                </div>
              </div>
              <p className="text-gray-400 text-sm font-mono leading-relaxed">{tr.card2Desc}</p>
              <button className="w-full py-3 bg-yellow-500/20 hover:bg-yellow-500/30 border border-yellow-500/50 text-yellow-400 font-mono font-bold rounded-lg transition-all duration-200 hover:shadow-yellow-500/20 hover:shadow-lg flex items-center justify-center gap-2 text-sm">
                <Gavel className="w-4 h-4" />
                {tr.card2Btn}
                <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}
      </div>

      {/* DAO Impeachment Modal */}
      {impeachModal && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center p-4"
          onClick={(e) => { if (e.target === e.currentTarget) closeModal(); }}
        >
          {/* Backdrop */}
          <div className="absolute inset-0 bg-black/80 backdrop-blur-sm" />

          {/* Modal panel */}
          <div className="relative w-full max-w-lg border border-orange-400/40 bg-gray-950 rounded-2xl overflow-hidden shadow-2xl shadow-orange-400/10 animate-modal-in">
            {/* Top accent bar */}
            <div className="h-1 w-full bg-gradient-to-r from-red-500 via-orange-400 to-yellow-400" />

            <div className="p-6 space-y-5">
              {/* Header */}
              <div className="flex items-start justify-between">
                <div>
                  <div className="flex items-center gap-2 mb-1">
                    <Gavel className="text-orange-400 w-5 h-5" />
                    <h3 className="text-orange-400 font-mono font-black text-lg uppercase tracking-widest">
                      {tr.impeachModalTitle}
                    </h3>
                  </div>
                  <p className="text-gray-500 text-xs font-mono">{tr.impeachModalSub}</p>
                  <p className="text-gray-600 text-xs font-mono mt-0.5">
                    {lang === 'en' ? 'Target: ' : '目标项目：'}
                    <span className="text-orange-300 font-bold">{impeachModal.project}</span>
                  </p>
                </div>
                <button
                  onClick={closeModal}
                  className="text-gray-600 hover:text-gray-300 transition-colors p-1"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>

              {/* Stats row */}
              <div className="grid grid-cols-2 gap-3">
                <div className="bg-gray-900/70 border border-gray-700/50 rounded-lg p-3 text-center">
                  <Users className="w-4 h-4 text-gray-500 mx-auto mb-1" />
                  <p className="text-gray-500 text-xs font-mono">{tr.impeachVoters}</p>
                  <p className="text-white font-mono font-bold text-lg">14,728</p>
                </div>
                <div className="bg-gray-900/70 border border-gray-700/50 rounded-lg p-3 text-center">
                  <Timer className="w-4 h-4 text-gray-500 mx-auto mb-1" />
                  <p className="text-gray-500 text-xs font-mono">{tr.impeachTimeLeft}</p>
                  <p className="text-yellow-400 font-mono font-bold text-lg">47:23:11</p>
                </div>
              </div>

              {/* Vote progress */}
              <div className="space-y-2">
                <div className="flex justify-between text-xs font-mono">
                  <span className="text-gray-400">{tr.impeachVotesLabel}</span>
                  <span className={`font-bold ${impeachModal.votes >= requiredPct ? 'text-green-400' : 'text-orange-400'}`}>
                    {impeachModal.votes.toFixed(1)}%
                  </span>
                </div>

                {/* Track */}
                <div className="relative h-6 bg-gray-800 rounded-full overflow-hidden border border-gray-700/50">
                  {/* Required threshold marker */}
                  <div
                    className="absolute top-0 bottom-0 w-0.5 bg-white/30 z-10"
                    style={{ left: `${requiredPct}%` }}
                  />

                  {/* Fill bar */}
                  <div
                    className={`h-full rounded-full transition-all duration-300 relative overflow-hidden ${
                      impeachModal.votes >= requiredPct
                        ? 'bg-gradient-to-r from-green-700 to-green-400'
                        : 'bg-gradient-to-r from-red-800 to-orange-400'
                    }`}
                    style={{ width: `${Math.min(impeachModal.votes, 100)}%` }}
                  >
                    <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/15 to-transparent animate-shimmer" />
                  </div>
                </div>

                <div className="flex justify-between text-xs font-mono text-gray-600">
                  <span>0%</span>
                  <span className="text-gray-400">
                    {tr.impeachRequired}: <span className="text-white font-bold">{requiredPct}%</span>
                  </span>
                  <span>100%</span>
                </div>
              </div>

              {/* Success state */}
              {impeachModal.success ? (
                <div className="border border-green-400/40 bg-green-400/10 rounded-xl p-4 space-y-2">
                  <div className="flex items-center gap-2">
                    <div className="w-7 h-7 rounded-full bg-green-400/20 border border-green-400/50 flex items-center justify-center flex-shrink-0">
                      <ThumbsUp className="w-4 h-4 text-green-400" />
                    </div>
                    <span className="text-green-400 font-mono font-black text-sm uppercase tracking-wider">
                      {lang === 'en' ? 'Impeachment Passed' : '弹劾通过'}
                    </span>
                  </div>
                  <p className="text-green-200 font-mono text-xs leading-relaxed">{tr.impeachSuccess}</p>
                </div>
              ) : (
                <button
                  onClick={castVote}
                  disabled={impeachModal.animating}
                  className="w-full py-3 bg-orange-500/20 hover:bg-orange-500/30 disabled:opacity-60 disabled:cursor-not-allowed border border-orange-500/50 text-orange-400 font-mono font-black uppercase tracking-wider rounded-lg transition-all duration-200 hover:shadow-orange-500/20 hover:shadow-lg text-sm flex items-center justify-center gap-2"
                >
                  <Gavel className="w-4 h-4" />
                  {tr.impeachCastBtn}
                </button>
              )}

              <button
                onClick={closeModal}
                className="w-full py-2 text-gray-600 hover:text-gray-400 font-mono text-xs transition-colors"
              >
                {tr.impeachClose}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}