import { useCallback, useEffect, useState } from 'react';
import { type WalletError } from '@solana/wallet-adapter-base';
import { ConnectionProvider, WalletProvider, useWallet, useConnection } from '@solana/wallet-adapter-react';
import { WalletModalProvider, WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { PhantomWalletAdapter } from '@solana/wallet-adapter-wallets';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';
import '@solana/wallet-adapter-react-ui/styles.css';
import { Globe, ShieldCheck, BookOpen, LayoutDashboard, Hexagon, Wallet, Coins, Home } from 'lucide-react';
import { type Lang } from './translations';
import TreasuryDashboard from './components/TreasuryDashboard';
import WallOfShame from './components/WallOfShame';
import VictimRelief from './components/VictimRelief';
import GreenLabelDashboard from './components/GreenLabelDashboard';
import TokenRevenueDashboard from './components/TokenRevenueDashboard';
import PublicLandingPage from './components/PublicLandingPage';

type Tab = 'home' | 'treasury' | 'shame' | 'relief' | 'greenLabel' | 'tokenRevenue';
type RpcStatus = 'checking' | 'ok' | 'error';

const endpoint = 'https://api.devnet.solana.com';
const wallets = [new PhantomWalletAdapter()];
const RPC_UNAVAILABLE_MESSAGE = 'Devnet RPC 暂时不可用';
const WALLET_REJECTED_MESSAGE = '用户取消连接';

const HERO = {
  en: {
    tagline: 'DAO 驱动的链上国库、赔付救济、自动质押分红、项目认证与社区共建协议',
    sub: '当前版本为 Devnet Alpha 测试网原型，链上国库分流账本已在 Devnet 验证。DAO、赔付、质押、绿标认证和社区共建模块将在后续合约版本中逐步开放。',
    tabTreasury: '国库分流账本',
    tabShame: '链上法庭与 DAO 治理',
    tabRelief: '资产质押与分红',
    protocolBadge: 'Alpha Protocol / α 协议',
  },
  zh: {
    tagline: 'DAO 驱动的链上国库、赔付救济、自动质押分红、项目认证与社区共建协议',
    sub: '当前版本为 Devnet Alpha 测试网原型，链上国库分流账本已在 Devnet 验证。DAO、赔付、质押、绿标认证和社区共建模块将在后续合约版本中逐步开放。',
    tabTreasury: '国库分流账本',
    tabShame: '链上法庭与 DAO 治理',
    tabRelief: '资产质押与分红',
    protocolBadge: 'Alpha Protocol / α 协议',
  },
};

interface AppContentProps {
  walletNotice: string | null;
  onClearWalletNotice: () => void;
}

function AppContent({ walletNotice, onClearWalletNotice }: AppContentProps) {
  const { connection } = useConnection();
  const { publicKey, connected: walletConnected } = useWallet();
  const [lang, setLang] = useState<Lang>('zh');
  const [activeTab, setActiveTab] = useState<Tab>('home');
  const [walletBalance, setWalletBalance] = useState<number | null>(null);
  const [rpcStatus, setRpcStatus] = useState<RpcStatus>('checking');
  const [rpcMessage, setRpcMessage] = useState<string | null>(null);

  useEffect(() => {
    if (walletConnected) {
      onClearWalletNotice();
    }
  }, [onClearWalletNotice, walletConnected]);

  useEffect(() => {
    let mounted = true;

    async function checkRpcHealth(showChecking = false) {
      if (showChecking) {
        setRpcStatus('checking');
      }

      try {
        await connection.getLatestBlockhash('confirmed');
        if (!mounted) return;

        setRpcStatus('ok');
        setRpcMessage(null);
      } catch {
        if (!mounted) return;

        setRpcStatus('error');
        setRpcMessage(RPC_UNAVAILABLE_MESSAGE);
      }
    }

    checkRpcHealth(true);
    const interval = setInterval(() => checkRpcHealth(false), 30000);

    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, [connection]);

  useEffect(() => {
    let mounted = true;

    async function fetchBalance() {
      if (walletConnected && publicKey) {
        try {
          const lamports = await connection.getBalance(publicKey);
          if (!mounted) return;

          setWalletBalance(lamports / LAMPORTS_PER_SOL);
          setRpcStatus('ok');
          setRpcMessage(null);
        } catch {
          if (!mounted) return;

          setWalletBalance(null);
          setRpcStatus('error');
          setRpcMessage(RPC_UNAVAILABLE_MESSAGE);
        }
      } else {
        setWalletBalance(null);
      }
    }

    fetchBalance();
    const interval = setInterval(fetchBalance, 10000);

    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, [walletConnected, publicKey, connection]);

  const h = HERO[lang];
  const rpcStatusMeta: Record<RpcStatus, { label: string; className: string; dotClassName: string }> = {
    checking: {
      label: 'RPC: CHECKING',
      className: 'border-yellow-400/20 bg-yellow-400/5 text-yellow-400',
      dotClassName: 'bg-yellow-400',
    },
    ok: {
      label: 'RPC: OK',
      className: 'border-green-500/20 bg-green-500/5 text-green-400',
      dotClassName: 'bg-green-400',
    },
    error: {
      label: 'RPC: ERROR',
      className: 'border-red-400/30 bg-red-400/10 text-red-400',
      dotClassName: 'bg-red-400',
    },
  };

  const tabs: {
    key: Tab;
    label: string;
    icon: React.ElementType;
    activeColor: string;
    activeBorder: string;
  }[] = [
    { key: 'home', label: '协议首页', icon: Home, activeColor: 'text-cyan-400', activeBorder: 'border-cyan-400' },
    { key: 'treasury', label: h.tabTreasury, icon: BookOpen, activeColor: 'text-green-400', activeBorder: 'border-green-400' },
    { key: 'shame', label: h.tabShame, icon: ShieldCheck, activeColor: 'text-red-400', activeBorder: 'border-red-400' },
    { key: 'relief', label: h.tabRelief, icon: LayoutDashboard, activeColor: 'text-cyan-400', activeBorder: 'border-cyan-400' },
    { key: 'greenLabel', label: 'Green Label 认证', icon: ShieldCheck, activeColor: 'text-emerald-400', activeBorder: 'border-emerald-400' },
    { key: 'tokenRevenue', label: '代币与收入闭环', icon: Coins, activeColor: 'text-yellow-400', activeBorder: 'border-yellow-400' },
  ];

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-200 font-mono select-none">
      <div className="fixed inset-0 pointer-events-none z-0 scanlines opacity-[0.02]" />
      <div className="fixed inset-0 pointer-events-none z-0 cyber-grid opacity-[0.03]" />

      <header className="sticky top-0 z-50 border-b border-zinc-900 bg-zinc-950/90 backdrop-blur-md">
        <div className="border-b border-zinc-900 px-4 py-1 flex items-center justify-between bg-zinc-950">
          <div className="flex items-center gap-4 text-zinc-600 text-[10px]">
            <span className="flex items-center gap-1.5 text-cyan-400/80 font-bold">
              <span className="w-1.5 h-1.5 rounded-full bg-cyan-400 inline-block" />
              DEVNET
            </span>
            <span className="hidden sm:inline-flex items-center gap-1 px-2 py-0.5 rounded border border-emerald-400/20 bg-emerald-400/5 text-emerald-400 font-bold text-[9px]">
              Devnet Alpha
            </span>
            <span className={`flex items-center gap-1 px-2 py-0.5 rounded border font-bold text-[9px] ${rpcStatusMeta[rpcStatus].className}`}>
              <span className={`w-1.5 h-1.5 rounded-full inline-block ${rpcStatusMeta[rpcStatus].dotClassName} ${rpcStatus === 'checking' ? 'animate-pulse' : ''}`} />
              {rpcStatus === 'error' ? RPC_UNAVAILABLE_MESSAGE : rpcStatusMeta[rpcStatus].label}
            </span>

            {walletConnected && (
              <span className="flex items-center gap-1 px-2 py-0.5 rounded border border-green-500/20 bg-green-500/5 text-green-400 font-bold text-[9px] animate-fade-in">
                <Wallet className="w-2.5 h-2.5" />
                BALANCE: {rpcStatus === 'error'
                  ? (rpcMessage ?? RPC_UNAVAILABLE_MESSAGE)
                  : walletBalance !== null
                    ? `${walletBalance.toFixed(4)} SOL`
                    : 'LOADING...'}
              </span>
            )}
            {walletNotice && (
              <span className="flex items-center gap-1 px-2 py-0.5 rounded border border-yellow-400/20 bg-yellow-400/5 text-yellow-400 font-bold text-[9px] animate-fade-in">
                {walletNotice}
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <span className="hidden sm:flex items-center gap-1 px-2 py-0.5 rounded border border-zinc-900 text-zinc-500 text-[9px] uppercase tracking-widest font-black">
              测试网原型
            </span>
            <WalletMultiButton className="!bg-green-500 !text-black font-mono hover:!bg-green-400 transition-all rounded-md px-4 py-2 text-sm" />
          </div>
        </div>

        <div className="px-4 sm:px-6 py-5 flex flex-col sm:flex-row sm:items-start gap-4 sm:gap-6 justify-between">
          <div className="flex items-start gap-4">
            <div className="relative flex-shrink-0 w-12 h-12 flex items-center justify-center">
              <Hexagon className="w-12 h-12 text-green-400/10 absolute" strokeWidth={1} />
              <span
                className="text-2xl font-black text-green-400 select-none relative z-10"
                style={{ textShadow: '0 0 20px rgba(74,222,128,0.6)' }}
              >
                α
              </span>
            </div>

            <div className="min-w-0">
              <div className="flex items-center gap-2 flex-wrap">
                <h1 className="text-lg font-black text-zinc-100 tracking-tight leading-none">{h.protocolBadge}</h1>
                <span className="text-[9px] font-black px-1.5 py-0.5 rounded bg-green-400/10 border border-green-400/20 text-green-400 uppercase tracking-widest">
                  Devnet Alpha
                </span>
              </div>
              <p className="text-green-400 font-mono text-xs font-bold mt-2 leading-snug max-w-lg">{h.tagline}</p>
              <p className="text-zinc-500 font-mono text-[10px] mt-1 leading-relaxed max-w-xl">{h.sub}</p>
            </div>
          </div>

          <button
            type="button"
            onClick={() => setLang(lang === 'en' ? 'zh' : 'en')}
            className="self-start sm:self-auto flex-shrink-0 flex items-center gap-1.5 px-3 py-1.5 rounded border border-zinc-800 bg-zinc-900/5
                       text-zinc-500 hover:bg-zinc-900 hover:border-zinc-700 hover:text-zinc-300 transition-all duration-200 text-[11px] font-bold tracking-wider group"
          >
            <Globe className="w-3 h-3 group-hover:rotate-12 transition-transform duration-300" />
            <span>EN</span>
            <span className="text-zinc-800">/</span>
            <span>中文</span>
          </button>
        </div>

        <nav className="px-4 sm:px-6 flex overflow-x-auto border-t border-zinc-900">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            const isActive = activeTab === tab.key;
            return (
              <button
                key={tab.key}
                type="button"
                onClick={() => setActiveTab(tab.key)}
                className={`flex items-center gap-2 px-5 py-3 text-xs font-black border-b-2 whitespace-nowrap transition-all duration-200 ${
                  isActive
                    ? `${tab.activeColor} ${tab.activeBorder} bg-zinc-900/10`
                    : 'text-zinc-600 border-transparent hover:text-zinc-400 hover:bg-zinc-900/5'
                }`}
              >
                <Icon className="w-3.5 h-3.5" />
                {tab.label}
              </button>
            );
          })}
        </nav>
      </header>

      <main className="max-w-6xl mx-auto px-4 sm:px-6 py-8 relative z-10">
        {activeTab === 'home' && <PublicLandingPage onNavigate={(target) => setActiveTab(target)} />}
        {activeTab === 'treasury' && (
          <TreasuryDashboard
            lang={lang}
            walletConnected={walletConnected}
            walletBalance={walletBalance}
          />
        )}
        {activeTab === 'shame' && <WallOfShame lang={lang} />}
        {activeTab === 'relief' && <VictimRelief lang={lang} />}
        {activeTab === 'greenLabel' && <GreenLabelDashboard />}
        {activeTab === 'tokenRevenue' && <TokenRevenueDashboard />}
      </main>

      <footer className="border-t border-zinc-900 mt-24 px-6 py-8 text-center space-y-4 bg-zinc-950">
        <div className="max-w-4xl mx-auto rounded-xl border border-red-400/25 bg-red-400/5 px-4 py-3 text-left">
          <p className="text-red-300 text-[10px] font-mono font-bold uppercase tracking-widest mb-1">
            Devnet Alpha 风险提示
          </p>
          <p className="text-red-100/80 text-[11px] font-mono leading-relaxed">
            当前版本为 Devnet Alpha 测试网原型。
            当前不涉及真实主网资金、真实赔付、真实质押收益或正式绿标认证。
            ALPHA 代币相关 creator fee / developer reward 接入协议国库的机制将在后续版本中逐步公开和验证。
            质押分红不代表固定收益，不承诺固定年化，不保证收益。
            后续功能将通过合约升级、测试、安全审查和社区治理逐步开放。
          </p>
        </div>
        <p className="text-zinc-700 text-[10px] font-mono tracking-wider max-w-3xl mx-auto leading-relaxed">
          © 2026 Alpha Protocol / α 协议 — Devnet Alpha 测试网原型。链上国库分流账本已在 Devnet 验证。
        </p>
        <div className="flex items-center justify-center gap-2 flex-wrap">
          {['Solana Devnet', 'Anchor Program', 'TreasuryState PDA', 'React / Vite', 'Phantom Wallet'].map((item) => (
            <span
              key={item}
              className="text-[9px] text-zinc-600 border border-zinc-900 bg-zinc-950 px-2 py-0.5 rounded font-mono font-bold tracking-tight"
            >
              {item}
            </span>
          ))}
        </div>
      </footer>
    </div>
  );
}

function isUserRejectedWalletError(error: WalletError): boolean {
  const nested = error.error instanceof Error ? error.error.message : String(error.error ?? '');
  const text = `${error.name} ${error.message} ${nested}`.toLowerCase();

  return text.includes('user rejected')
    || text.includes('reject')
    || text.includes('decline')
    || text.includes('cancel');
}

export default function App() {
  const [walletNotice, setWalletNotice] = useState<string | null>(null);
  const clearWalletNotice = useCallback(() => setWalletNotice(null), []);
  const handleWalletError = useCallback((error: WalletError) => {
    setWalletNotice(isUserRejectedWalletError(error) ? WALLET_REJECTED_MESSAGE : '钱包连接暂时不可用');
  }, []);

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect onError={handleWalletError}>
        <WalletModalProvider>
          <AppContent walletNotice={walletNotice} onClearWalletNotice={clearWalletNotice} />
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
