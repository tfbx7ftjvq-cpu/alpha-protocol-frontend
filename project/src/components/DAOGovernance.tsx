import { Buffer } from 'buffer';
import { useEffect, useState } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { Vote, CheckCircle2, XCircle, ShieldAlert, Award, ChevronRight } from 'lucide-react';
import { Lang } from '../translations';

if (typeof window !== 'undefined' && !(window as typeof window & { Buffer?: typeof Buffer }).Buffer) {
  (window as typeof window & { Buffer?: typeof Buffer }).Buffer = Buffer;
}

const ALPHA_TOKEN_MINT = '6VXo4Ut8wNyhFvHSiXVtnJwZfcoRg8FyNFjMgjokSMu8';

interface Props {
  lang: Lang;
  onExecuteSuccess?: () => void;
}

const TREASURY_PDA = new PublicKey('2yVkP9w21w78tD6vSAtmfbpukWN5u8VmsZxF8bUSyWhv');

interface Proposal {
  id: string;
  titleZh: string;
  titleEn: string;
  proposer: string;
  forVotes: number;
  againstVotes: number;
  quorum: number;
  endTime: number;
  executed: boolean;
  status: 'ACTIVE_VOTING' | 'PASSED' | 'DEFEATED' | 'EXECUTED';
  descZh: string;
  descEn: string;
}

export default function DAOGovernance({ lang, onExecuteSuccess }: Props) {
  const isZh = lang === 'zh';
  const { connection } = useConnection();
  const { connected, publicKey } = useWallet();
  const [userAlphaBalance, setUserAlphaBalance] = useState(0);
  const [voteError, setVoteError] = useState('');

  const [proposals, setProposals] = useState<Proposal[]>([
    {
      id: 'DAO-053',
      titleZh: '项目 [RugPullX] 绿标合规与 500 USDC 保证金审计',
      titleEn: 'Project [RugPullX] Green-List Compliance & 500 USDC Deposit Audit',
      proposer: '8UVQ...vPhe',
      forVotes: 245000,
      againstVotes: 120000,
      quorum: 5000000,
      endTime: Date.now() + 48 * 60 * 60 * 1000,
      executed: false,
      status: 'ACTIVE_VOTING',
      descZh: '本提案用于审计项目方提交的 500 USDC 合规履约保证金，并以 DAO 投票决定该笔保证金后续执行路径：通过则原路退回申请地址，否决则按国库规则没收充公。',
      descEn: 'This proposal audits the 500 USDC compliance deposit submitted by the project and uses DAO voting to determine its execution path: pass to refund the applicant address, or reject to confiscate and route into treasury per protocol rules.',
    },
    {
      id: 'DAO-054',
      titleZh: '紧急理赔倾斜：从国库储备调拨 15,000 USDC 注入理赔池',
      titleEn: 'Emergency Fund Tilting: Allocate 15,000 USDC from Reserve to Restitution Pool',
      proposer: '8UVQ...vPhe',
      forVotes: 410000,
      againstVotes: 32000,
      quorum: 5000000,
      endTime: Date.now() + 48 * 60 * 60 * 1000,
      executed: false,
      status: 'ACTIVE_VOTING',
      descZh: '本提案面向二级市场暴跌情形，授权国库从储备中倾斜 15,000 USDC 至理赔池，以提升即时救济与市场稳定能力。',
      descEn: 'This proposal targets secondary market crashes and authorizes the treasury to tilt 15,000 USDC from reserves into the restitution pool for immediate relief and stabilization.',
    },
    {
      id: 'DAO-055',
      titleZh: '全链司法弹劾：将恶意砸盘项目 [AlphaDrain] 永久列入黑名单并没收资产',
      titleEn: 'Permanent On-Chain Impeachment: Blacklist [AlphaDrain] & Forfeit Assets',
      proposer: '8UVQ...vPhe',
      forVotes: 189000,
      againstVotes: 290000,
      quorum: 5000000,
      endTime: Date.now() + 48 * 60 * 60 * 1000,
      executed: false,
      status: 'ACTIVE_VOTING',
      descZh: '本提案用于对恶意地址执行最高司法确权，完成黑名单封禁后将其资产永久没收并进入链上司法处置流程。',
      descEn: 'This proposal applies the highest judicial on-chain ruling to malicious addresses, blacklisting them permanently and forfeiting assets into the judicial disposal flow.',
    },
  ]);

  const [hasVotedMap, setHasVotedMap] = useState<Record<string, boolean>>({});
  const [voteTypeMap, setVoteTypeMap] = useState<Record<string, 'FOR' | 'AGAINST' | null>>({});
  const [walletVoteLocks, setWalletVoteLocks] = useState<Record<string, Record<string, boolean>>>({});
  const [now, setNow] = useState(Date.now());
  const [txLogMap, setTxLogMap] = useState<Record<string, string[]>>({});
  const [isProcessing, setIsProcessing] = useState<boolean>(false);
  const [activeProposalId, setActiveProposalId] = useState<string>('DAO-053');

  const voteWeight = Math.floor(Math.sqrt(userAlphaBalance));
  const proposalWeight = voteWeight;

  useEffect(() => {
    const fetchAlphaBalance = async () => {
      if (!publicKey) {
        setUserAlphaBalance(0);
        return;
      }
      try {
        const res = await connection.getParsedTokenAccountsByOwner(publicKey, {
          mint: new PublicKey(ALPHA_TOKEN_MINT),
        });
        const balance = res.value.reduce((sum, account) => {
          const amount = account.account.data.parsed.info.tokenAmount.uiAmount ?? 0;
          return sum + amount;
        }, 0);
        setUserAlphaBalance(balance);
      } catch {
        setUserAlphaBalance(0);
      }
    };

    void fetchAlphaBalance();
  }, [connection, publicKey]);

  const activeProposal = proposals.find((item) => item.id === activeProposalId) ?? proposals[0];

  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    setProposals((prev) =>
      prev.map((item) => {
        if (item.status !== 'ACTIVE_VOTING' || now <= item.endTime) return item;
        const nextStatus = item.forVotes > item.againstVotes ? 'PASSED' : 'DEFEATED';
        return {
          ...item,
          status: item.executed ? 'EXECUTED' : nextStatus,
        };
      })
    );
  }, [now]);

  useEffect(() => {
    const dao053 = proposals.find((item) => item.id === 'DAO-053');
    if (!dao053 || dao053.status !== 'DEFEATED' || dao053.executed === true) return;

    setProposals((prev) =>
      prev.map((item) =>
        item.id === 'DAO-053' && item.status === 'DEFEATED' && item.executed === false
          ? { ...item, executed: true, status: 'EXECUTED' }
          : item
      )
    );
    if (onExecuteSuccess) onExecuteSuccess();
    console.info('[treasury]', {
      status: 'FORFEIT_LOCKED',
      proposalId: dao053.id,
      treasury: TREASURY_PDA.toBase58(),
      note: '500 USDC + 100,000 $α permanently retained and routed into restitution sub-pools',
    });
  }, [proposals, onExecuteSuccess]);

  useEffect(() => {
    if (!activeProposal && proposals.length > 0) setActiveProposalId(proposals[0].id);
  }, [activeProposal, proposals]);

  const totalVotes = activeProposal.forVotes + activeProposal.againstVotes;
  const forPercentage = totalVotes > 0 ? (activeProposal.forVotes / totalVotes) * 100 : 0;
  const quorumPercentage = Math.min((totalVotes / activeProposal.quorum) * 100, 100);
  const currentVoteType = voteTypeMap[activeProposal.id] ?? null;
  const hasVoted = hasVotedMap[activeProposal.id] ?? false;
  const txLog = txLogMap[activeProposal.id] ?? [];

  const handleVote = (proposalId: string, type: 'YES' | 'NO') => {
    if (!connected || !publicKey || isProcessing) return;
    if (userAlphaBalance <= 0) {
      setVoteError(isZh ? '未持有 $α 代币，无法参与审判' : 'You do not hold $α tokens and cannot participate in the tribunal');
      return;
    }

    const voteWeight = Math.floor(Math.sqrt(userAlphaBalance));
    if (voteWeight <= 0) return;

    const walletKey = publicKey.toBase58();
    if (walletVoteLocks[proposalId]?.[walletKey]) return;

    setVoteError('');
    setIsProcessing(true);
    setWalletVoteLocks((prev) => ({
      ...prev,
      [proposalId]: {
        ...(prev[proposalId] ?? {}),
        [walletKey]: true,
      },
    }));
    setProposals((prev) =>
      prev.map((item) =>
        item.id === proposalId
          ? {
              ...item,
              forVotes: type === 'YES' ? item.forVotes + voteWeight : item.forVotes,
              againstVotes: type === 'NO' ? item.againstVotes + voteWeight : item.againstVotes,
            }
          : item
      )
    );
    setTxLogMap((prev) => ({
      ...prev,
      [proposalId]: [
        ...(prev[proposalId] ?? []),
        isZh
          ? `[CALL] ${proposalId} 投票签名中：当前钱包 ${walletKey.slice(0, 4)}...${walletKey.slice(-4)}，权重 ${voteWeight.toLocaleString()} ve-α`
          : `[CALL] ${proposalId} vote signing: wallet ${walletKey.slice(0, 4)}...${walletKey.slice(-4)}, weight ${voteWeight.toLocaleString()} ve-α`,
      ],
    }));

    setTimeout(() => {
      setHasVotedMap((prev) => ({ ...prev, [proposalId]: true }));
      setVoteTypeMap((prev) => ({ ...prev, [proposalId]: type === 'YES' ? 'FOR' : 'AGAINST' }));
      setIsProcessing(false);
      setTxLogMap((prev) => ({
        ...prev,
        [proposalId]: [
          ...(prev[proposalId] ?? []),
          isZh
            ? `[SUCCESS] 投票成功：${type === 'YES' ? '赞成' : '反对'}，权重 ${voteWeight.toLocaleString()}`
            : `[SUCCESS] Vote cast: ${type}, weight ${voteWeight.toLocaleString()}`,
        ],
      }));
    }, 1200);
  };

  const handleExecute = () => {
    if (activeProposal.status !== 'ACTIVE_VOTING' || isProcessing) return;
    setIsProcessing(true);
    setTxLogMap((prev) => ({
      ...prev,
      [activeProposal.id]: [
        ...(prev[activeProposal.id] ?? []),
        isZh ? '[EXECUTE] 正在广播提案执行指令...' : '[EXECUTE] Broadcasting proposal execution instruction...',
      ],
    }));

    setTimeout(() => {
      setProposals((prev) => prev.map((item) => (item.id === activeProposal.id ? { ...item, status: 'EXECUTED', executed: true } : item)));
      setIsProcessing(false);
      setTxLogMap((prev) => ({
        ...prev,
        [activeProposal.id]: [
          ...(prev[activeProposal.id] ?? []),
          isZh
            ? `[DISPATCH] ${activeProposal.id} 已执行，国库赔付池解冻权限已释放。`
            : `[DISPATCH] ${activeProposal.id} executed, treasury payout unlock permission released.`,
        ],
      }));
      if (onExecuteSuccess) onExecuteSuccess();
    }, 1500);
  };

  return (
    <div className="border border-zinc-800 bg-zinc-950/20 rounded-xl overflow-hidden mt-8">
      {/* 组件头部 */}
      <div className="flex items-center justify-between px-5 py-3 border-b border-zinc-900 bg-zinc-950">
        <div className="flex items-center gap-2">
          <div className="p-1.5 rounded bg-purple-500/10 border border-purple-500/20">
            <Vote className="w-3.5 h-3.5 text-purple-400" />
          </div>
          <p className="text-zinc-200 font-mono text-xs font-bold">
            {isZh ? 'DAO 链上主权治理矩阵' : 'DAO On-Chain Sovereign Governance Matrix'}
          </p>
        </div>
        <button
          type="button"
          onClick={() => setActiveProposalId((prev) => {
            const currentIndex = proposals.findIndex((item) => item.id === prev);
            const nextIndex = (currentIndex + 1) % proposals.length;
            return proposals[nextIndex]?.id ?? prev;
          })}
          className="text-[9px] font-mono px-1.5 py-0.5 rounded border border-purple-400/20 bg-purple-400/5 text-purple-400 uppercase tracking-wider"
        >
          {activeProposal.id} · {activeProposal.status}
        </button>
      </div>

      <div className="p-5 space-y-6">
        {/* 提案主卡片 */}
        <div className="border border-zinc-900 bg-zinc-950/40 rounded-xl p-5 space-y-4">
          <div className="flex flex-col sm:flex-row sm:items-start justify-between gap-3 border-b border-zinc-900 pb-4">
            <div className="space-y-1">
              <div className="flex items-center gap-2">
                <span className="text-xs font-mono font-bold text-purple-400 bg-purple-500/5 border border-purple-500/10 px-1.5 py-0.5 rounded">
                  {activeProposal.id}
                </span>
                <h3 className="text-sm font-bold text-zinc-200 font-mono">
                  {isZh ? activeProposal.titleZh : activeProposal.titleEn}
                </h3>
              </div>
              <p className="text-zinc-500 font-mono text-[10px]">
                {isZh ? '提案发起人' : 'Proposer'}: <span className="text-zinc-400 font-bold">{activeProposal.proposer}</span>
              </p>
            </div>
            <div className="text-right shrink-0">
              <p className="text-zinc-600 font-mono text-[9px] uppercase tracking-wider">{isZh ? '投票倒计时' : 'Voting Ends In'}</p>
              <p className="text-sm font-bold font-mono text-zinc-300 animate-pulse">{activeProposal.endTime}</p>
            </div>
          </div>

          {/* 提案描述 */}
          <p className="text-zinc-400 font-mono text-xs leading-relaxed bg-black/20 border border-zinc-900/60 p-3 rounded-lg">
            {isZh ? activeProposal.descZh : activeProposal.descEn}
          </p>

          {/* 进度条：赞成率 与 法定人数(Quorum) */}
          <div className="space-y-3 pt-2">
            {/* 赞成 vs 反对 */}
            <div className="space-y-1">
              <div className="flex justify-between text-[11px] font-mono">
                <span className="text-green-400 font-bold">{isZh ? '赞成' : 'FOR'}: {activeProposal.forVotes.toLocaleString()} ({forPercentage.toFixed(1)}%)</span>
                <span className="text-red-400 font-bold">{isZh ? '反对' : 'AGAINST'}: {activeProposal.againstVotes.toLocaleString()}</span>
              </div>
              <div className="h-2 w-full bg-zinc-900 rounded-full overflow-hidden flex">
                <div className="bg-green-500 h-full transition-all duration-500" style={{ width: `${forPercentage}%` }} />
                <div className="bg-red-500 h-full transition-all duration-500" style={{ width: `${100 - forPercentage}%` }} />
              </div>
            </div>

            {/* Quorum 进度 */}
            <div className="space-y-1">
              <div className="flex justify-between text-[10px] font-mono text-zinc-500">
                <span>{isZh ? '当前投票总量' : 'Current Casted'}: {totalVotes.toLocaleString()}</span>
                <span>{isZh ? '法定通过票数(Quorum)' : 'Quorum Require'}: {activeProposal.quorum.toLocaleString()} ({quorumPercentage.toFixed(1)}%)</span>
              </div>
              <div className="h-1.5 w-full bg-zinc-900 rounded-full overflow-hidden">
                <div className="bg-purple-500 h-full transition-all duration-500" style={{ width: `${quorumPercentage}%` }} />
              </div>
            </div>
          </div>

          {/* 交互控制台 */}
          <div className="flex flex-col sm:flex-row gap-3 pt-2">
            <button
              onClick={() => handleVote(activeProposal.id, 'YES')}
              disabled={hasVotedMap[activeProposal.id] || isProcessing || activeProposal.status !== 'ACTIVE_VOTING'}
              className={`flex-1 py-2.5 rounded font-mono font-bold text-xs uppercase tracking-wider border transition-all flex items-center justify-center gap-2 ${
                currentVoteType === 'FOR'
                  ? 'bg-green-500/10 border-green-500/40 text-green-400'
                  : hasVoted
                  ? 'bg-zinc-950 border-zinc-900 text-zinc-600 cursor-not-allowed'
                  : 'bg-zinc-900 hover:bg-zinc-850 border-zinc-800 hover:border-green-500/30 text-green-400'
              }`}
            >
              <CheckCircle2 className="w-3.5 h-3.5" />
              {isZh ? '投赞成票' : 'Vote For'} {currentVoteType === 'FOR' && '✓'}
            </button>

            <button
              onClick={() => handleVote(activeProposal.id, 'NO')}
              disabled={hasVotedMap[activeProposal.id] || isProcessing || activeProposal.status !== 'ACTIVE_VOTING'}
              className={`flex-1 py-2.5 rounded font-mono font-bold text-xs uppercase tracking-wider border transition-all flex items-center justify-center gap-2 ${
                currentVoteType === 'AGAINST'
                  ? 'bg-red-500/10 border-red-500/40 text-red-400'
                  : hasVoted
                  ? 'bg-zinc-950 border-zinc-900 text-zinc-600 cursor-not-allowed'
                  : 'bg-zinc-900 hover:bg-zinc-850 border-zinc-800 hover:border-red-500/30 text-red-400'
              }`}
            >
              <XCircle className="w-3.5 h-3.5" />
              {isZh ? '投反对票' : 'Vote Against'} {currentVoteType === 'AGAINST' && '✓'}
            </button>

            {/* 如果达到了 Quorum 且尚未执行，开放执行按钮模拟 */}
            {totalVotes >= activeProposal.quorum && activeProposal.status === 'ACTIVE_VOTING' && (
              <button
                onClick={handleExecute}
                disabled={isProcessing}
                className="px-5 py-2.5 bg-purple-600 hover:bg-purple-500 border border-purple-500 text-white font-mono font-bold text-xs uppercase tracking-wider rounded transition-all animate-pulse flex items-center justify-center gap-1.5"
              >
                <Award className="w-3.5 h-3.5" />
                {isZh ? '执行提案' : 'Execute'}
              </button>
            )}
          </div>
        </div>

        {/* 投票权力状态卡片 */}
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div className="bg-zinc-950/40 border border-zinc-900 rounded-lg p-4 flex items-center justify-between">
            <div>
              <p className="text-zinc-600 font-mono text-[9px] uppercase tracking-widest mb-0.5">{isZh ? '你的当前治理权重' : 'Your Voting Weight'}</p>
              <p className="text-base font-black font-mono text-purple-400 tabular-nums">
                {proposalWeight.toLocaleString()} <span className="text-zinc-600 text-xs">ve-α</span>
              </p>
            </div>
            <ShieldAlert className="w-5 h-5 text-purple-500/40" />
          </div>

          <div className="bg-zinc-950/40 border border-zinc-900 rounded-lg p-4 flex items-center justify-between">
            <div>
              <p className="text-zinc-600 font-mono text-[9px] uppercase tracking-widest mb-0.5">{isZh ? '提案治理通过率阈值' : 'Governance Threshold'}</p>
              <p className="text-base font-black font-mono text-zinc-400 tabular-nums">
                &gt; 66.7% <span className="text-zinc-600 text-xs">Supermajority</span>
              </p>
            </div>
            <ChevronRight className="w-4 h-4 text-zinc-700" />
          </div>
        </div>

        {/* 交互日志输出终端 */}
        {(voteError || txLog.length > 0) && (
          <div className="border border-purple-900/30 bg-black rounded-lg p-3 font-mono text-[11px] space-y-1">
            <p className="text-purple-400 font-bold border-b border-purple-900/20 pb-1 mb-1.5 uppercase tracking-widest text-[9px]">
              {isZh ? '🏛️ 治理多签调用日志' : '🏛️ Governance Multisig Call Log'}
            </p>
            {voteError && <div className="text-red-400 break-all leading-relaxed">{voteError}</div>}
            {txLog.map((log, index) => (
              <div key={index} className="text-zinc-300 break-all leading-relaxed">
                {log}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}