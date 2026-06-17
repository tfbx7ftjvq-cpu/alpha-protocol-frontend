import { useCallback, useEffect, useMemo, useState, type ElementType } from 'react';
import { type Wallet as AnchorWallet } from '@coral-xyz/anchor';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import {
  AlertCircle,
  ArrowDown,
  CheckCircle2,
  Coins,
  Gauge,
  Landmark,
  Loader2,
  RefreshCw,
  ShieldAlert,
  Target,
  Wallet,
  WalletCards,
} from 'lucide-react';
import { type Lang } from '../translations';
import {
  PROGRAM_ID,
  createAlphaProgram,
  depositToTreasuryState,
  getTreasuryStatePda,
  initializeTreasuryState,
} from '../lib/alphaProgram';

interface Props {
  lang: Lang;
  walletConnected: boolean;
  walletBalance: number | null;
}

const TEST_DEPOSIT_AMOUNT = 10000;
const TREASURY_STATE_PDA = getTreasuryStatePda();

const SPLIT_RULES = [
  {
    label: '50% 受害者赔付 / 救济池',
    usage: '50% 补充赔付救济池',
    field: 'reliefPool',
    color: 'text-emerald-400 border-emerald-400/25 bg-emerald-400/5',
  },
  {
    label: '20% 回购 / 销毁池',
    usage: '20% 用于回购 / 销毁',
    field: 'buybackPool',
    color: 'text-red-400 border-red-400/25 bg-red-400/5',
  },
  {
    label: '20% DAO 贡献者 / 生态建设池',
    usage: '20% 用于 DAO 贡献者 / 生态建设池',
    field: 'payrollPool',
    color: 'text-blue-400 border-blue-400/25 bg-blue-400/5',
  },
  {
    label: '10% 质押分红池',
    usage: '10% 用于质押分红池',
    field: 'stakingPool',
    color: 'text-yellow-400 border-yellow-400/25 bg-yellow-400/5',
  },
];

const REVENUE_PATH = [
  'ALPHA 交易产生 creator fee / developer reward',
  '进入公开协议国库钱包',
  '纳入 Alpha Protocol 国库模型',
  '支持 50/20/20/10 分配机制',
];

export default function TreasuryDashboard({ lang, walletConnected, walletBalance }: Props) {
  const locale = lang === 'zh' ? 'zh-CN' : 'en-US';
  const { connection } = useConnection();
  const wallet = useWallet();
  const { connected, publicKey, signAllTransactions, signTransaction } = wallet;

  const [chainReadStatus, setChainReadStatus] = useState<ChainReadStatus>('idle');
  const [chainTreasuryState, setChainTreasuryState] = useState<ChainTreasuryState | null>(null);
  const [chainReadError, setChainReadError] = useState<string | null>(null);
  const [chainLastSync, setChainLastSync] = useState<Date | null>(null);
  const [initializeStatus, setInitializeStatus] = useState<ActionStatus>('idle');
  const [initializeMessage, setInitializeMessage] = useState<string | null>(null);
  const [depositStatus, setDepositStatus] = useState<ActionStatus>('idle');
  const [depositMessage, setDepositMessage] = useState<string | null>(null);
  const [refreshStatus, setRefreshStatus] = useState<ActionStatus>('idle');
  const [refreshMessage, setRefreshMessage] = useState<string | null>(null);

  const walletAddress = walletConnected && connected && publicKey ? publicKey.toBase58() : '钱包未连接';
  const walletAddressShort = walletConnected && connected && publicKey ? shortAddress(publicKey.toBase58()) : '钱包未连接';
  const canSign = Boolean(walletConnected && connected && publicKey && signTransaction && signAllTransactions);
  const isReading = chainReadStatus === 'loading';
  const treasuryReady = chainReadStatus === 'ready' && chainTreasuryState !== null;

  const chainStatusMeta = useMemo<Record<ChainReadStatus, { label: string; className: string }>>(() => ({
    idle: {
      label: '钱包未连接',
      className: 'text-zinc-400 border-zinc-700 bg-zinc-800/40',
    },
    loading: {
      label: '读取中',
      className: 'text-yellow-400 border-yellow-400/30 bg-yellow-400/10',
    },
    ready: {
      label: '已同步',
      className: 'text-emerald-400 border-emerald-400/30 bg-emerald-400/10',
    },
    missing: {
      label: '未初始化',
      className: 'text-orange-400 border-orange-400/30 bg-orange-400/10',
    },
    error: {
      label: '读取失败',
      className: 'text-red-400 border-red-400/40 bg-red-400/10',
    },
  }), []);

  const loadTreasuryState = useCallback(async (mode: LoadMode = 'auto') => {
    if (!walletConnected || !connected || !publicKey) {
      setChainReadStatus('idle');
      setChainTreasuryState(null);
      setChainReadError(null);
      setChainLastSync(null);
      setInitializeStatus('idle');
      setInitializeMessage(null);
      setDepositStatus('idle');
      setDepositMessage(null);

      if (mode === 'refresh') {
        setRefreshStatus('error');
        setRefreshMessage('请先连接钱包');
      } else {
        setRefreshStatus('idle');
        setRefreshMessage(null);
      }
      return;
    }

    if (!signTransaction || !signAllTransactions) {
      setChainReadStatus('error');
      setChainTreasuryState(null);
      setChainReadError('当前钱包不支持交易签名');

      if (mode === 'refresh') {
        setRefreshStatus('error');
        setRefreshMessage('当前钱包不支持交易签名');
      }
      return;
    }

    setChainReadStatus('loading');
    setChainReadError(null);

    if (mode === 'refresh') {
      setRefreshStatus('loading');
      setRefreshMessage(null);
    }

    try {
      const program = createAlphaProgram(connection, {
        publicKey,
        signAllTransactions,
        signTransaction,
      } as unknown as AnchorWallet);
      const accountClient = (program.account as unknown as {
        treasuryState: {
          fetchNullable(address: typeof TREASURY_STATE_PDA): Promise<AnchorTreasuryState | null>;
        };
      }).treasuryState;
      const treasuryState = await accountClient.fetchNullable(TREASURY_STATE_PDA);

      if (!treasuryState) {
        setChainTreasuryState(null);
        setChainReadStatus('missing');
        setChainLastSync(new Date());

        if (mode === 'refresh') {
          setRefreshStatus('success');
          setRefreshMessage('链上状态已刷新：TreasuryState PDA 尚未初始化');
        }
        return;
      }

      setChainTreasuryState({
        totalInflow: formatU64(treasuryState.totalInflow),
        reliefPool: formatU64(treasuryState.reliefPool),
        buybackPool: formatU64(treasuryState.buybackPool),
        payrollPool: formatU64(treasuryState.payrollPool),
        stakingPool: formatU64(treasuryState.stakingPool),
      });
      setChainReadStatus('ready');
      setChainLastSync(new Date());

      if (mode === 'refresh') {
        setRefreshStatus('success');
        setRefreshMessage('链上状态已刷新');
      }
    } catch (err) {
      const message = getReadableErrorMessage(err);
      const isMissing = message.toLowerCase().includes('account does not exist');

      setChainTreasuryState(null);
      setChainReadStatus(isMissing ? 'missing' : 'error');
      setChainReadError(isMissing ? null : message);
      setChainLastSync(new Date());

      if (mode === 'refresh') {
        setRefreshStatus(isMissing ? 'success' : 'error');
        setRefreshMessage(isMissing ? '链上状态已刷新：TreasuryState PDA 尚未初始化' : message);
      }
    }
  }, [connected, connection, publicKey, signAllTransactions, signTransaction, walletConnected]);

  useEffect(() => {
    void loadTreasuryState('auto');
  }, [loadTreasuryState]);

  async function handleInitializeTreasury() {
    if (initializeStatus === 'loading') return;

    if (!canSign || !publicKey) {
      setInitializeStatus('error');
      setInitializeMessage('请先连接钱包');
      return;
    }

    setInitializeStatus('loading');
    setInitializeMessage(null);
    setChainReadError(null);

    try {
      const signature = await initializeTreasuryState(
        connection,
        {
          publicKey,
          signAllTransactions,
          signTransaction,
        } as unknown as AnchorWallet,
        publicKey,
      );

      setInitializeStatus('success');
      setInitializeMessage(`initialize_protocol 成功：${signature}`);
      await loadTreasuryState('after-action');
    } catch (err) {
      setInitializeStatus('error');
      setInitializeMessage(getReadableErrorMessage(err));
    }
  }

  async function handleTestDeposit() {
    if (depositStatus === 'loading') return;

    if (!canSign || !publicKey) {
      setDepositStatus('error');
      setDepositMessage('请先连接钱包');
      return;
    }

    if (!treasuryReady) {
      setDepositStatus('error');
      setDepositMessage('TreasuryState PDA 尚未初始化');
      return;
    }

    setDepositStatus('loading');
    setDepositMessage(null);
    setChainReadError(null);

    try {
      const signature = await depositToTreasuryState(
        connection,
        {
          publicKey,
          signAllTransactions,
          signTransaction,
        } as unknown as AnchorWallet,
        TEST_DEPOSIT_AMOUNT,
      );

      setDepositStatus('success');
      setDepositMessage(`deposit(${TEST_DEPOSIT_AMOUNT}) 成功：${signature}`);
      await loadTreasuryState('after-action');
    } catch (err) {
      setDepositStatus('error');
      setDepositMessage(getReadableErrorMessage(err));
    }
  }

  async function handleRefreshTreasury() {
    if (refreshStatus === 'loading' || isReading) return;
    await loadTreasuryState('refresh');
  }

  const metricCards = [
    { label: 'totalInflow', caption: '累计流入', value: chainTreasuryState?.totalInflow ?? '--', color: 'text-emerald-400' },
    { label: 'reliefPool', caption: '受害者赔付 / 救济池', value: chainTreasuryState?.reliefPool ?? '--', color: 'text-green-400' },
    { label: 'buybackPool', caption: '回购 / 销毁池', value: chainTreasuryState?.buybackPool ?? '--', color: 'text-red-400' },
    { label: 'payrollPool', caption: 'DAO 贡献者 / 生态建设池', value: chainTreasuryState?.payrollPool ?? '--', color: 'text-blue-400' },
    { label: 'stakingPool', caption: '质押分红池', value: chainTreasuryState?.stakingPool ?? '--', color: 'text-yellow-400' },
  ];

  return (
    <div className="space-y-8">
      <section className="space-y-5">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div className="space-y-3">
            <div className="flex flex-wrap items-center gap-2">
              <Badge icon={Gauge} label="Devnet Alpha" tone="emerald" />
              <Badge icon={Gauge} label="Devnet" tone="cyan" />
              <Badge icon={Landmark} label="链上国库分流账本已在 Devnet 验证" tone="zinc" />
            </div>
            <div>
              <h2 className="text-xl font-black text-zinc-100 font-mono tracking-wide uppercase">
                国库分流账本
              </h2>
              <p className="mt-2 max-w-3xl text-xs font-mono leading-relaxed text-zinc-500">
                当前版本为 Devnet Alpha 测试网原型。Treasury 国库板块是真实 Devnet 链上功能，已用于验证 50/20/20/10 分流账本。
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2 rounded border border-zinc-800 bg-zinc-950/70 px-3 py-2 text-xs text-zinc-400">
            <Wallet className="h-4 w-4 text-emerald-400" />
            <span className="font-mono">{walletAddressShort}</span>
          </div>
        </div>

        <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-4">
          <InfoTile icon={Gauge} label="版本 / 网络" value="Devnet Alpha / Devnet" tone="text-emerald-400" />
          <InfoTile
            icon={WalletCards}
            label="当前连接钱包地址"
            value={walletAddress}
            tone={walletConnected && connected ? 'text-cyan-400' : 'text-zinc-500'}
          />
          <InfoTile icon={Target} label="Program ID" value={PROGRAM_ID.toBase58()} tone="text-cyan-400" />
          <InfoTile icon={Landmark} label="TreasuryState PDA" value={TREASURY_STATE_PDA.toBase58()} tone="text-cyan-400" />
        </div>

        <div className="flex flex-wrap items-center gap-2 text-[11px] text-zinc-500">
          <span className="rounded border border-zinc-800 bg-zinc-950/60 px-2 py-1">
            钱包 SOL：{walletConnected && walletBalance !== null ? `${walletBalance.toFixed(4)} SOL` : '钱包未连接'}
          </span>
          <span className={`rounded border px-2 py-1 ${chainStatusMeta[chainReadStatus].className}`}>
            链上状态：{chainStatusMeta[chainReadStatus].label}
          </span>
        </div>
      </section>

      <section className="space-y-5 rounded-xl border border-cyan-400/20 bg-cyan-400/5 p-5">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
          <SectionHeader
            icon={Landmark}
            eyebrow="Devnet Anchor Treasury State"
            title="Treasury 链上国库"
            description="读取真实 TreasuryState PDA，并展示链上 totalInflow 与四个资金池原始 u64 余额。"
          />
          <div className={`inline-flex w-fit items-center gap-2 rounded border px-3 py-1.5 text-xs font-bold ${chainStatusMeta[chainReadStatus].className}`}>
            {isReading && <Loader2 className="h-3.5 w-3.5 animate-spin" />}
            {chainStatusMeta[chainReadStatus].label}
          </div>
        </div>

        <div className="grid grid-cols-1 gap-3 md:grid-cols-5">
          {metricCards.map((metric) => (
            <div key={metric.label} className="rounded border border-zinc-800 bg-zinc-950/70 p-4">
              <p className="mb-1 text-[10px] font-bold uppercase tracking-widest text-zinc-600">{metric.label}</p>
              <p className="mb-2 text-[10px] text-zinc-500">{metric.caption}</p>
              <p className={`break-all font-mono text-xl font-black tabular-nums ${metric.color}`}>
                {metric.value}
              </p>
              <p className="mt-1 text-[10px] text-zinc-600">raw u64</p>
            </div>
          ))}
        </div>

        <div className="grid grid-cols-1 gap-3 md:grid-cols-4">
          {SPLIT_RULES.map((rule) => (
            <div key={rule.label} className={`rounded border p-4 ${rule.color}`}>
              <p className="text-2xl font-black">{rule.label.split(' ')[0]}</p>
              <p className="mt-1 text-xs font-bold text-zinc-200">{rule.label}</p>
              <p className="mt-2 text-[10px] text-zinc-500">{rule.field}</p>
            </div>
          ))}
        </div>

        <div className="rounded border border-zinc-800 bg-zinc-950/60 p-4">
          <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
            <div>
              <p className="text-sm font-black text-zinc-100">链上操作</p>
              <p className="mt-1 text-xs text-zinc-500">
                保留 initialize_protocol、deposit(10000) 与链上状态刷新，用于 Devnet Alpha 验证。
              </p>
            </div>
            <div className="flex flex-col gap-2 sm:flex-row">
              <button
                type="button"
                onClick={handleRefreshTreasury}
                disabled={refreshStatus === 'loading' || isReading}
                className="inline-flex items-center justify-center gap-2 rounded border border-cyan-400/30 bg-cyan-400/10 px-4 py-2 text-xs font-bold text-cyan-300 transition-all hover:bg-cyan-400/15 disabled:cursor-not-allowed disabled:opacity-50"
              >
                {refreshStatus === 'loading' || isReading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <RefreshCw className="h-3.5 w-3.5" />}
                {refreshStatus === 'loading' || isReading ? '刷新中...' : '刷新链上状态'}
              </button>

              {chainReadStatus === 'missing' && (
                <button
                  type="button"
                  onClick={handleInitializeTreasury}
                  disabled={initializeStatus === 'loading'}
                  className="inline-flex items-center justify-center gap-2 rounded border border-orange-400/35 bg-orange-400/10 px-4 py-2 text-xs font-bold text-orange-300 transition-all hover:bg-orange-400/15 disabled:cursor-not-allowed disabled:opacity-50"
                >
                  {initializeStatus === 'loading' ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Landmark className="h-3.5 w-3.5" />}
                  {initializeStatus === 'loading' ? '初始化中...' : '初始化链上国库'}
                </button>
              )}

              {treasuryReady && (
                <button
                  type="button"
                  onClick={handleTestDeposit}
                  disabled={depositStatus === 'loading'}
                  className="inline-flex items-center justify-center gap-2 rounded border border-emerald-400/35 bg-emerald-400/10 px-4 py-2 text-xs font-bold text-emerald-300 transition-all hover:bg-emerald-400/15 disabled:cursor-not-allowed disabled:opacity-50"
                >
                  {depositStatus === 'loading' ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Coins className="h-3.5 w-3.5" />}
                  {depositStatus === 'loading' ? '测试入账中...' : '测试入账 10000'}
                </button>
              )}
            </div>
          </div>

          <div className="mt-4 space-y-2">
            {chainReadStatus === 'idle' && <StatusNotice tone="neutral" message="钱包未连接。连接钱包后可读取 TreasuryState PDA。" />}
            {chainReadStatus === 'missing' && <StatusNotice tone="warning" message="TreasuryState PDA 不存在。请使用初始化按钮创建链上国库。" />}
            {chainReadError && <StatusNotice tone="error" message={chainReadError} />}
            {initializeMessage && <StatusNotice tone={initializeStatus === 'success' ? 'success' : 'error'} message={initializeMessage} />}
            {depositMessage && <StatusNotice tone={depositStatus === 'success' ? 'success' : 'error'} message={depositMessage} />}
            {refreshMessage && <StatusNotice tone={refreshStatus === 'success' ? 'success' : 'error'} message={refreshMessage} />}
          </div>

          {chainLastSync && (
            <p className="mt-4 text-right text-[10px] text-zinc-600">
              Last sync: {chainLastSync.toLocaleTimeString(locale)}
            </p>
          )}
        </div>
      </section>

      <section className="space-y-5 rounded-xl border border-zinc-800 bg-zinc-950/40 p-5">
        <SectionHeader
          icon={Coins}
          eyebrow="Roadmap Revenue Module"
          title="Token Revenue / 协议收入来源"
          description="ALPHA 代币未来通过发币平台或交易平台产生的 creator fee / developer reward，将作为 Alpha Protocol 的协议收入来源之一。"
        />

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
          <div className="rounded border border-zinc-800 bg-zinc-950/60 p-4">
            <h3 className="mb-4 text-sm font-black text-zinc-100">资金路径</h3>
            <div className="space-y-2">
              {REVENUE_PATH.map((step, index) => (
                <div key={step}>
                  <div className="rounded border border-zinc-800 bg-zinc-950 px-3 py-2 text-xs font-bold text-zinc-300">
                    {step}
                  </div>
                  {index < REVENUE_PATH.length - 1 && (
                    <ArrowDown className="mx-auto my-1 h-4 w-4 text-zinc-700" />
                  )}
                </div>
              ))}
            </div>
          </div>

          <div className="rounded border border-zinc-800 bg-zinc-950/60 p-4">
            <h3 className="mb-4 text-sm font-black text-zinc-100">分配用途</h3>
            <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
              {SPLIT_RULES.map((rule) => (
                <div key={rule.usage} className={`rounded border p-3 ${rule.color}`}>
                  <p className="text-lg font-black">{rule.usage.split(' ')[0]}</p>
                  <p className="text-xs font-bold text-zinc-200">{rule.usage}</p>
                </div>
              ))}
            </div>
          </div>
        </div>

        <div className="rounded border border-cyan-400/20 bg-cyan-400/5 px-4 py-3 text-xs leading-relaxed text-cyan-100">
          当前 Devnet Alpha 已验证链上国库账本分流逻辑。主网 creator fee / developer reward 接入自动分流将在后续版本开放。
        </div>
        <div className="rounded border border-zinc-800 bg-zinc-950/60 px-4 py-3 text-xs leading-relaxed text-zinc-400">
          协议收入将优先服务于受害者保护、生态建设、长期持有者激励和社区治理。本页面不提供收益承诺，也不暗示无风险获利。
        </div>
      </section>
    </div>
  );
}

function Badge({ icon: Icon, label, tone }: { icon: ElementType; label: string; tone: 'emerald' | 'cyan' | 'zinc' }) {
  const className = {
    emerald: 'border-emerald-400/25 bg-emerald-400/10 text-emerald-400',
    cyan: 'border-cyan-400/25 bg-cyan-400/10 text-cyan-400',
    zinc: 'border-zinc-700 bg-zinc-900/60 text-zinc-400',
  }[tone];

  return (
    <span className={`inline-flex items-center gap-1.5 rounded border px-2 py-1 text-[10px] font-bold uppercase tracking-widest ${className}`}>
      <Icon className="h-3 w-3" />
      {label}
    </span>
  );
}

function SectionHeader({
  icon: Icon,
  eyebrow,
  title,
  description,
}: {
  icon: ElementType;
  eyebrow: string;
  title: string;
  description: string;
}) {
  return (
    <div className="max-w-4xl">
      <div className="mb-2 flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-emerald-400">
        <Icon className="h-3.5 w-3.5" />
        {eyebrow}
      </div>
      <h3 className="text-lg font-black text-zinc-100">{title}</h3>
      <p className="mt-2 text-xs leading-relaxed text-zinc-400">{description}</p>
    </div>
  );
}

function InfoTile({
  icon: Icon,
  label,
  value,
  tone,
}: {
  icon: ElementType;
  label: string;
  value: string;
  tone: string;
}) {
  return (
    <div className="rounded border border-zinc-800 bg-zinc-950/70 p-4">
      <div className="mb-2 flex items-center gap-2">
        <Icon className={`h-4 w-4 ${tone}`} />
        <p className="text-[10px] font-bold uppercase tracking-widest text-zinc-600">{label}</p>
      </div>
      <p className={`break-all font-mono text-xs font-bold leading-relaxed ${tone}`}>{value}</p>
    </div>
  );
}

function StatusNotice({ tone, message }: { tone: NoticeTone; message: string }) {
  const toneClass: Record<NoticeTone, string> = {
    neutral: 'text-zinc-400 border-zinc-800 bg-zinc-950',
    success: 'text-emerald-400 border-emerald-400/25 bg-emerald-400/5',
    warning: 'text-orange-400 border-orange-400/25 bg-orange-400/5',
    error: 'text-red-400 border-red-400/30 bg-red-400/10',
  };
  const Icon = tone === 'success'
    ? CheckCircle2
    : tone === 'warning'
      ? AlertCircle
      : tone === 'error'
        ? ShieldAlert
        : Gauge;

  return (
    <div className={`flex items-start gap-2 rounded border px-3 py-2 text-xs leading-relaxed ${toneClass[tone]}`}>
      <Icon className="mt-0.5 h-3.5 w-3.5 flex-shrink-0" />
      <span className="break-words">{message}</span>
    </div>
  );
}

type ChainReadStatus = 'idle' | 'loading' | 'ready' | 'missing' | 'error';
type ActionStatus = 'idle' | 'loading' | 'success' | 'error';
type LoadMode = 'auto' | 'refresh' | 'after-action';
type NoticeTone = 'neutral' | 'success' | 'warning' | 'error';

interface ChainTreasuryState {
  totalInflow: string;
  reliefPool: string;
  buybackPool: string;
  payrollPool: string;
  stakingPool: string;
}

interface AnchorTreasuryState {
  totalInflow: unknown;
  reliefPool: unknown;
  buybackPool: unknown;
  payrollPool: unknown;
  stakingPool: unknown;
}

function formatU64(value: unknown): string {
  const raw = value && typeof value === 'object' && 'toString' in value
    ? value.toString()
    : String(value ?? 0);

  try {
    return BigInt(raw).toString();
  } catch {
    return raw;
  }
}

function shortAddress(address: string): string {
  if (address.length <= 12) return address;
  return `${address.slice(0, 4)}...${address.slice(-4)}`;
}

function getReadableErrorMessage(err: unknown): string {
  if (err instanceof Error) {
    return err.message;
  }

  return String(err);
}
