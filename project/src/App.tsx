import { useState } from 'react';
import { Globe, ShieldCheck, BookOpen, LayoutDashboard, Wifi, WifiOff, Hexagon } from 'lucide-react';
import { Lang } from './translations';
import TreasuryDashboard from './components/TreasuryDashboard';
import WallOfShame from './components/WallOfShame';
import AlphaStaking from './components/AlphaStaking'; // 优雅重命名：升级为全网公共质押组件

type Tab = 'treasury' | 'shame' | 'relief';

const HERO = {
  en: {
    tagline: 'Fully Decentralized, Code-Enforced Retail Haven & On-Chain Court',
    sub: 'Permissionless streaming router for all Solana participants. No whitelist. No admin private keys. 50/20/20/10 automated splits.',
    tabTreasury: 'Treasury Router',
    tabShame: 'On-Chain Court & DAO',
    tabRelief: 'Staking & Dividends',
    connected: 'CONNECTED',
    connectWallet: 'Connect Wallet',
    protocolBadge: 'α Protocol',
    permissionless: 'PERMISSIONLESS',
  },
  zh: {
    tagline: '完全去中心化、代码强制执行的散户避风港与链上法庭',
    sub: '对全体 Solana 用户无需许可开放。无白名单限制，无管理员私钥。50/20/20/10 资金流自动化实时拆分。',
    tabTreasury: '国库分流账本',
    tabShame: '链上法庭与DAO治理',
    tabRelief: '资产质押与分红',
    connected: '已连接',
    connectWallet: '连接钱包',
    protocolBadge: 'α 协议',
    permissionless: '无许可',
  },
};

export default function App() {
  const [lang, setLang] = useState<Lang>('zh'); // 默认对齐国人开发直觉，设为中文初始状态
  const [activeTab, setActiveTab] = useState<Tab>('treasury');
  const [walletConnected, setWalletConnected] = useState(false);

  const h = HERO[lang];

  const tabs: {
    key: Tab;
    label: string;
    icon: React.ElementType;
    activeColor: string;
    activeBorder: string;
  }[] = [
    { key: 'treasury', label: h.tabTreasury, icon: BookOpen,        activeColor: 'text-green-400', activeBorder: 'border-green-400' },
    { key: 'shame',    label: h.tabShame,    icon: ShieldCheck,     activeColor: 'text-red-400',   activeBorder: 'border-red-400'   },
    { key: 'relief',   label: h.tabRelief,   icon: LayoutDashboard, activeColor: 'text-cyan-400',  activeBorder: 'border-cyan-400'  },
  ];

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-200 font-mono select-none">
      {/* Scanline overlay */}
      <div className="fixed inset-0 pointer-events-none z-0 scanlines opacity-[0.02]" />
      {/* Grid background */}
      <div className="fixed inset-0 pointer-events-none z-0 cyber-grid opacity-[0.03]" />

      {/* ── Header ── */}
      <header className="sticky top-0 z-50 border-b border-zinc-900 bg-zinc-950/90 backdrop-blur-md">

        {/* Status bar */}
        <div className="border-b border-zinc-900 px-4 py-1 flex items-center justify-between bg-zinc-950">
          <div className="flex items-center gap-4 text-zinc-600 text-[10px]">
            <span className="flex items-center gap-1.5 text-green-400/70 font-bold animate-pulse">
              <span className="w-1.5 h-1.5 rounded-full bg-green-400 inline-block" />
              LIVE MONITOR
            </span>
            <span>SLOT: #284,723,019</span>
            <span className="hidden sm:block">BASE FEE: 0.000025 SOL</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="hidden sm:flex items-center gap-1 px-2 py-0.5 rounded border border-zinc-900 text-zinc-600 text-[9px] uppercase tracking-widest font-black">
              {h.permissionless}
            </span>
            <button
              onClick={() => setWalletConnected(!walletConnected)}
              className={`flex items-center gap-1.5 px-3 py-1 rounded border text-[10px] font-black tracking-wider transition-all duration-200 ${
                walletConnected
                  ? 'border-green-400/30 text-green-400 bg-green-400/5 hover:bg-green-400/10'
                  : 'border-zinc-800 text-zinc-500 hover:border-zinc-700 hover:text-zinc-400'
              }`}
            >
              {walletConnected ? <Wifi className="w-3 h-3" /> : <WifiOff className="w-3 h-3" />}
              {walletConnected ? h.connected : h.connectWallet}
            </button>
          </div>
        </div>

        {/* Hero identity row */}
        <div className="px-4 sm:px-6 py-5 flex flex-col sm:flex-row sm:items-start gap-4 sm:gap-6 justify-between">
          <div className="flex items-start gap-4">
            {/* Logo mark */}
            <div className="relative flex-shrink-0 w-12 h-12 flex items-center justify-center">
              <Hexagon className="w-12 h-12 text-green-400/10 absolute" strokeWidth={1} />
              <span
                className="text-2xl font-black text-green-400 select-none relative z-10"
                style={{ textShadow: '0 0 20px rgba(74,222,128,0.6)' }}
              >
                α
              </span>
            </div>

            {/* Title block */}
            <div className="min-w-0">
              <div className="flex items-center gap-2 flex-wrap">
                <h1 className="text-lg font-black text-zinc-100 tracking-tight leading-none">
                  {h.protocolBadge}
                </h1>
                <span className="text-[9px] font-black px-1.5 py-0.5 rounded bg-green-400/10 border border-green-400/20 text-green-400 uppercase tracking-widest">
                  v2.1 Production
                </span>
              </div>
              <p className="text-green-400 font-mono text-xs font-bold mt-2 leading-snug max-w-lg">
                {h.tagline}
              </p>
              <p className="text-zinc-500 font-mono text-[10px] mt-1 leading-relaxed max-w-xl">
                {h.sub}
              </p>
            </div>
          </div>

          {/* Language switcher */}
          <button
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

        {/* Nav tabs */}
        <nav className="px-4 sm:px-6 flex overflow-x-auto border-t border-zinc-900">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            const isActive = activeTab === tab.key;
            return (
              <button
                key={tab.key}
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

      {/* ── Main content ── */}
      {/* 核心风控注入：将钱包状态统一向下游透传，保证子组件内部投票、质押能无缝读取上下文 */}
      <main className="max-w-6xl mx-auto px-4 sm:px-6 py-8 relative z-10">
        {activeTab === 'treasury' && <TreasuryDashboard lang={lang} walletConnected={walletConnected} />}
        {activeTab === 'shame'    && <WallOfShame lang={lang} walletConnected={walletConnected} />}
        {activeTab === 'relief'   && <AlphaStaking lang={lang} walletConnected={walletConnected} />}
      </main>

      {/* ── Footer ── */}
      <footer className="border-t border-zinc-900 mt-24 px-6 py-8 text-center space-y-4 bg-zinc-950">
        <p className="text-zinc-700 text-[10px] font-mono tracking-wider max-w-3xl mx-auto leading-relaxed">
          {lang === 'en'
            ? '© 2026 α Protocol — Code-Enforced Router Asset — Autonomous state transitions are programmatically committed via decentralized daemon nodes. Zero administrative trust.'
            : '© 2026 α 协议 — 代码硬履约路由资产 — 自动化状态转换由去中心化守护节点机械提交，免除一切管理人信任风险。'}
        </p>
        <div className="flex items-center justify-center gap-2 flex-wrap">
          {['Solana Web3', 'Jupiter Swap API', 'Jito Bundles', 'Node.js Core Daemon', 'SQLite Persistent Storage'].map((item) => (
            <span key={item} className="text-[9px] text-zinc-600 border border-zinc-900 bg-zinc-950 px-2 py-0.5 rounded font-mono font-bold tracking-tight">
              {item}
            </span>
          ))}
        </div>
      </footer>
    </div>
  );
}