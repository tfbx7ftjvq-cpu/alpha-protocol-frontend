import { useState, useEffect, useCallback, useRef } from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import {
  PublicKey,
  TransactionMessage,
  VersionedTransaction,
  TransactionInstruction,
  AccountMeta,
} from '@solana/web3.js';
import { Database, Zap, RefreshCw, TrendingUp, CircleAlert as AlertCircle, CircleCheck as CheckCircle2, Loader as Loader2, ArrowDownToLine, Activity, Target, Shield } from 'lucide-react';
import { Lang } from '../translations';

interface Props {
  lang: Lang;
}

const TREASURY_PDA = new PublicKey('8UVQ1SMYZmcKLGwBcoJJFKkFPtBAw1A9sR77s8GZQeJo');
const PROGRAM_ID = new PublicKey('BPRyK5jBrcpFMiM3rasChzDwvTVmicUPjumZmiTG99jf');
const BUYBACK_THRESHOLD_USDC = 1000;
const BUYBACK_THRESHOLD_BASE = BigInt(1_000_000_000);
const POLL_INTERVAL_MS = 5000;
const USDC_DECIMALS = 1_000_000;

// Rust struct layout: 8-byte discriminator, then retail_reserve as u64 LE
const RETAIL_RESERVE_OFFSET = 8;

type TxStatus = 'idle' | 'signing' | 'sending' | 'confirming' | 'success' | 'error';

const copy = {
  en: {
    title: 'On-Chain Treasury Monitor',
    subtitle: 'Live PDA State · Devnet · Auto-Poll 5s',
    poolBalance: 'Retail Reserve Balance',
    buybackThreshold: 'Buyback Trigger Threshold',
    progress: 'Pool Fill Progress',
    progressSub: 'Distance to next programmatic buyback',
    depositTitle: 'Inject Treasury Funds',
    depositPlaceholder: 'Amount (USDC)',
    depositBtn: 'Deposit Fund',
    depositSigning: 'Awaiting Signature...',
    depositSending: 'Broadcasting...',
    depositConfirming: 'Confirming...',
    depositSuccess: 'Deposit Confirmed On-Chain',
    depositError: 'Transaction Failed',
    connectWallet: 'Connect wallet to deposit',
    polling: 'LIVE POLLING',
    lastFetch: 'Last sync',
    pdaAddress: 'Treasury PDA',
    programId: 'Program ID',
    noData: 'Fetching on-chain state...',
    fetchError: 'RPC fetch failed — retrying',
    buybackReady: 'BUYBACK THRESHOLD REACHED',
    estimatedBurn: 'Est. tokens to burn at trigger',
  },
  zh: {
    title: '链上国库实时监控',
    subtitle: '链上 PDA 状态 · Devnet · 5秒自动轮询',
    poolBalance: '散户储备金水位',
    buybackThreshold: '回购销毁触发阈值',
    progress: '资金池填充进度',
    progressSub: '距离下一次程序化回购的距离',
    depositTitle: '注入国库资金',
    depositPlaceholder: '金额（USDC）',
    depositBtn: '注入国库',
    depositSigning: '等待钱包签名...',
    depositSending: '广播交易中...',
    depositConfirming: '确认上链中...',
    depositSuccess: '存款已链上确认',
    depositError: '交易失败',
    connectWallet: '请先连接钱包以发起注入',
    polling: '实时轮询中',
    lastFetch: '上次同步',
    pdaAddress: '国库 PDA 地址',
    programId: '合约程序 ID',
    noData: '正在读取链上状态...',
    fetchError: 'RPC 读取失败 — 自动重试中',
    buybackReady: '回购触发阈值已达成',
    estimatedBurn: '触发时预计销毁代币数量',
  },
};

function formatUsdc(amount: number): string {
  return amount.toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

function shortKey(pk: PublicKey): string {
  const s = pk.toBase58();
  return `${s.slice(0, 6)}...${s.slice(-6)}`;
}

function ProgressBar({ pct, threshold }: { pct: number; threshold: boolean }) {
  const clampedPct = Math.min(100, Math.max(0, pct));
  return (
    <div className="relative h-5 bg-zinc-900 rounded-full overflow-hidden border border-zinc-800">
      {/* Threshold marker at 100% (fill line) */}
      <div
        className="absolute inset-y-0 w-0.5 bg-cyan-400/50 z-10"
        style={{ left: '100%', transform: 'translateX(-1px)' }}
      />
      {/* Fill */}
      <div
        className={`h-full rounded-full relative overflow-hidden transition-all duration-700 ease-out ${
          threshold
            ? 'bg-gradient-to-r from-red-700 via-red-500 to-orange-400'
            : clampedPct > 66
            ? 'bg-gradient-to-r from-emerald-900 via-emerald-500 to-emerald-400'
            : clampedPct > 33
            ? 'bg-gradient-to-r from-cyan-900 via-cyan-600 to-cyan-400'
            : 'bg-gradient-to-r from-zinc-700 via-zinc-500 to-zinc-400'
        }`}
        style={{ width: `${clampedPct}%` }}
      >
        {/* Shimmer */}
        <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/15 to-transparent animate-shimmer" />
      </div>
      {/* Breathing glow overlay */}
      {clampedPct > 0 && (
        <div
          className={`absolute inset-y-0 left-0 rounded-full pointer-events-none transition-all duration-700 ${
            threshold ? 'shadow-[0_0_16px_4px_rgba(239,68,68,0.5)]' : 'shadow-[0_0_12px_3px_rgba(52,211,153,0.4)]'
          }`}
          style={{ width: `${clampedPct}%` }}
        />
      )}
    </div>
  );
}

function PulsingDot({ color }: { color: 'green' | 'red' | 'yellow' }) {
  const map = {
    green: 'bg-emerald-400 shadow-[0_0_8px_2px_rgba(52,211,153,0.6)]',
    red: 'bg-red-400 shadow-[0_0_8px_2px_rgba(239,68,68,0.6)]',
    yellow: 'bg-yellow-400 shadow-[0_0_8px_2px_rgba(250,204,21,0.6)]',
  };
  return (
    <span className={`inline-block w-2 h-2 rounded-full animate-pulse ${map[color]}`} />
  );
}

export default function OnChainTreasury({ lang }: Props) {
  const c = copy[lang];
  const zh = lang === 'zh';
  const { connection } = useConnection();
  const { publicKey, connected, sendTransaction } = useWallet();

  // ─── On-chain state ───────────────────────────────────────────────────────
  const [retailReserveUsdc, setRetailReserveUsdc] = useState<number | null>(null);
  const [fetchError, setFetchError] = useState(false);
  const [lastSync, setLastSync] = useState<Date | null>(null);
  const [isPolling, setIsPolling] = useState(false);

  // ─── Deposit state ────────────────────────────────────────────────────────
  const [depositAmount, setDepositAmount] = useState('');
  const [txStatus, setTxStatus] = useState<TxStatus>('idle');
  const [txSig, setTxSig] = useState<string | null>(null);
  const [txError, setTxError] = useState<string | null>(null);

  // ─── Derived ─────────────────────────────────────────────────────────────
  const balance = retailReserveUsdc ?? 0;
  const progressPct = (balance / BUYBACK_THRESHOLD_USDC) * 100;
  const buybackReached = balance >= BUYBACK_THRESHOLD_USDC;

  // ─── PDA polling ─────────────────────────────────────────────────────────
  const fetchTreasury = useCallback(async () => {
    try {
      setIsPolling(true);
      const info = await connection.getAccountInfo(TREASURY_PDA);
      if (!info || !info.data) {
        setRetailReserveUsdc(0);
        setFetchError(false);
        setLastSync(new Date());
        return;
      }
      const buf = Buffer.from(info.data);
      // Guard: need at least 16 bytes (8 discriminator + 8 u64)
      if (buf.length < RETAIL_RESERVE_OFFSET + 8) {
        setRetailReserveUsdc(0);
        setFetchError(false);
        setLastSync(new Date());
        return;
      }
      const rawBig = buf.readBigUInt64LE(RETAIL_RESERVE_OFFSET);
      // Convert BigInt to Number safely before any React rendering
      const usdc = Number(rawBig) / USDC_DECIMALS;
      setRetailReserveUsdc(usdc);
      setFetchError(false);
      setLastSync(new Date());
    } catch {
      setFetchError(true);
    } finally {
      setIsPolling(false);
    }
  }, [connection]);

  useEffect(() => {
    fetchTreasury();
    const id = setInterval(fetchTreasury, POLL_INTERVAL_MS);
    return () => clearInterval(id);
  }, [fetchTreasury]);

  // ─── Deposit transaction ──────────────────────────────────────────────────
  const depositRef = useRef<string>('');

  async function handleDeposit() {
    if (!connected || !publicKey) return;
    const amountFloat = parseFloat(depositAmount);
    if (!Number.isFinite(amountFloat) || amountFloat <= 0) return;

    depositRef.current = depositAmount;
    setTxStatus('signing');
    setTxSig(null);
    setTxError(null);

    try {
      const amountBase = BigInt(Math.round(amountFloat * USDC_DECIMALS));
      // Encode instruction: 8-byte discriminator for "deposit" (first 8 bytes of sha256("global:deposit"))
      // Using a simple placeholder discriminator; real usage would match the program's IDL discriminator
      const discriminator = Buffer.from([0xf2, 0x23, 0xc6, 0x89, 0x52, 0xe1, 0xf2, 0xb6]);
      const amountBuf = Buffer.alloc(8);
      amountBuf.writeBigUInt64LE(amountBase);
      const data = Buffer.concat([discriminator, amountBuf]);

      const keys: AccountMeta[] = [
        { pubkey: TREASURY_PDA, isSigner: false, isWritable: true },
        { pubkey: publicKey, isSigner: true, isWritable: true },
      ];

      const ix = new TransactionInstruction({ programId: PROGRAM_ID, keys, data });

      const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
      const msg = new TransactionMessage({
        payerKey: publicKey,
        recentBlockhash: blockhash,
        instructions: [ix],
      }).compileToV0Message();

      const tx = new VersionedTransaction(msg);

      setTxStatus('sending');
      const sig = await sendTransaction(tx, connection);
      setTxSig(sig);

      setTxStatus('confirming');
      await connection.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight }, 'confirmed');

      setTxStatus('success');
      setDepositAmount('');
      // Refresh balance after successful deposit
      setTimeout(fetchTreasury, 1500);
    } catch (err) {
      setTxError(err instanceof Error ? err.message : String(err));
      setTxStatus('error');
    }
  }

  // ─── Button label ─────────────────────────────────────────────────────────
  function depositBtnLabel() {
    switch (txStatus) {
      case 'signing':     return c.depositSigning;
      case 'sending':     return c.depositSending;
      case 'confirming':  return c.depositConfirming;
      case 'success':     return c.depositSuccess;
      case 'error':       return c.depositError;
      default:            return c.depositBtn;
    }
  }

  const isBusy = txStatus === 'signing' || txStatus === 'sending' || txStatus === 'confirming';

  return (
    <div className="space-y-5">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center gap-3 justify-between">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg border border-emerald-400/25 bg-emerald-400/10">
            <Database className="w-5 h-5 text-emerald-400" />
          </div>
          <div>
            <h3 className="text-sm font-black text-zinc-100 font-mono tracking-wide uppercase">
              {c.title}
            </h3>
            <p className="text-zinc-600 font-mono text-[10px] uppercase tracking-widest flex items-center gap-1.5 mt-0.5">
              <PulsingDot color={fetchError ? 'red' : 'green'} />
              {c.subtitle}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2 flex-wrap">
          <span className="text-[9px] font-mono font-bold px-2 py-1 rounded border border-emerald-400/25 bg-emerald-400/5 text-emerald-400 uppercase tracking-widest flex items-center gap-1">
            <Activity className="w-2.5 h-2.5 animate-pulse" />
            {c.polling}
          </span>
          <button
            type="button"
            onClick={fetchTreasury}
            disabled={isPolling}
            className="flex items-center gap-1 text-[10px] font-mono text-zinc-500 hover:text-zinc-300 px-2 py-1 rounded border border-zinc-800 hover:border-zinc-700 transition-all"
          >
            <RefreshCw className={`w-3 h-3 ${isPolling ? 'animate-spin' : ''}`} />
            {zh ? '手动刷新' : 'Refresh'}
          </button>
        </div>
      </div>

      {/* PDA metadata strip */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <div className="border border-zinc-800 bg-zinc-950/60 rounded-lg px-3 py-2 font-mono text-[10px]">
          <p className="text-zinc-600 uppercase tracking-widest mb-1">{c.pdaAddress}</p>
          <p className="text-cyan-400 font-bold">{shortKey(TREASURY_PDA)}</p>
        </div>
        <div className="border border-zinc-800 bg-zinc-950/60 rounded-lg px-3 py-2 font-mono text-[10px]">
          <p className="text-zinc-600 uppercase tracking-widest mb-1">{c.programId}</p>
          <p className="text-cyan-400 font-bold">{shortKey(PROGRAM_ID)}</p>
        </div>
      </div>

      {/* Balance + threshold cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        {/* Pool Balance */}
        <div className="relative border border-emerald-400/20 bg-emerald-400/5 rounded-xl p-5 overflow-hidden">
          <div className="absolute inset-x-0 top-0 h-0.5 bg-gradient-to-r from-emerald-600 via-emerald-400 to-cyan-400 opacity-80" />
          <div className="flex items-start justify-between mb-3">
            <div className="flex items-center gap-2">
              <TrendingUp className="w-4 h-4 text-emerald-400" />
              <span className="text-emerald-400 font-mono text-[10px] font-bold uppercase tracking-widest">
                {c.poolBalance}
              </span>
            </div>
            {lastSync && (
              <span className="text-zinc-600 font-mono text-[9px]">
                {lastSync.toLocaleTimeString()}
              </span>
            )}
          </div>
          {fetchError ? (
            <div className="flex items-center gap-2 text-red-400 font-mono text-xs">
              <AlertCircle className="w-4 h-4" />
              {c.fetchError}
            </div>
          ) : retailReserveUsdc === null ? (
            <div className="flex items-center gap-2 text-zinc-500 font-mono text-xs">
              <Loader2 className="w-4 h-4 animate-spin" />
              {c.noData}
            </div>
          ) : (
            <div>
              <p
                className="text-4xl font-black font-mono tabular-nums text-emerald-400"
                style={{ textShadow: '0 0 24px rgba(52,211,153,0.5)' }}
              >
                {formatUsdc(balance)}
              </p>
              <p className="text-zinc-500 font-mono text-[10px] mt-1">USDC · retail_reserve</p>
            </div>
          )}
        </div>

        {/* Buyback Threshold */}
        <div className="relative border border-cyan-400/20 bg-cyan-400/5 rounded-xl p-5 overflow-hidden">
          <div className="absolute inset-x-0 top-0 h-0.5 bg-gradient-to-r from-cyan-600 via-cyan-400 to-blue-400 opacity-80" />
          <div className="flex items-start justify-between mb-3">
            <div className="flex items-center gap-2">
              <Target className="w-4 h-4 text-cyan-400" />
              <span className="text-cyan-400 font-mono text-[10px] font-bold uppercase tracking-widest">
                {c.buybackThreshold}
              </span>
            </div>
            {buybackReached && (
              <span className="text-[9px] font-mono text-red-400 border border-red-400/40 px-1.5 py-0.5 rounded animate-pulse">
                {c.buybackReady}
              </span>
            )}
          </div>
          <p
            className="text-4xl font-black font-mono tabular-nums text-cyan-400"
            style={{ textShadow: '0 0 24px rgba(34,211,238,0.5)' }}
          >
            {formatUsdc(BUYBACK_THRESHOLD_USDC)}
          </p>
          <p className="text-zinc-500 font-mono text-[10px] mt-1">
            USDC · {BUYBACK_THRESHOLD_BASE.toString()} base units
          </p>
        </div>
      </div>

      {/* Progress bar */}
      <div className="border border-zinc-800 bg-zinc-950/60 rounded-xl p-5 space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Zap className={`w-4 h-4 ${buybackReached ? 'text-red-400' : 'text-emerald-400'}`} />
            <span className="text-zinc-300 font-mono text-xs font-bold uppercase tracking-widest">
              {c.progress}
            </span>
          </div>
          <span
            className={`text-xl font-black font-mono tabular-nums ${
              buybackReached ? 'text-red-400' : progressPct > 66 ? 'text-emerald-400' : 'text-cyan-400'
            }`}
          >
            {progressPct.toFixed(2)}%
          </span>
        </div>
        <ProgressBar pct={progressPct} threshold={buybackReached} />
        <div className="flex items-center justify-between font-mono text-[10px] text-zinc-600">
          <span>0 USDC</span>
          <span className="text-zinc-500">{c.progressSub}</span>
          <span>{formatUsdc(BUYBACK_THRESHOLD_USDC)} USDC</span>
        </div>
      </div>

      {/* Deposit card */}
      <div className="relative border border-zinc-700/50 bg-zinc-950/80 rounded-xl overflow-hidden">
        <div className="absolute inset-x-0 top-0 h-0.5 bg-gradient-to-r from-zinc-700 via-emerald-500/50 to-zinc-700" />
        <div className="px-5 py-4 border-b border-zinc-800/70 flex items-center gap-2">
          <ArrowDownToLine className="w-4 h-4 text-emerald-400" />
          <span className="text-zinc-200 font-mono text-xs font-bold uppercase tracking-widest">
            {c.depositTitle}
          </span>
          <span className="ml-auto text-[9px] font-mono text-zinc-600 border border-zinc-800 px-1.5 py-0.5 rounded">
            VersionedTransaction v0
          </span>
        </div>
        <div className="p-5 space-y-4">
          <div className="flex gap-3">
            <div className="flex-1 relative">
              <input
                type="number"
                min={0}
                step="any"
                value={depositAmount}
                onChange={(e) => setDepositAmount(e.target.value)}
                placeholder={c.depositPlaceholder}
                disabled={!connected || isBusy}
                className="w-full bg-zinc-950 border border-zinc-700/60 rounded-lg px-4 py-3 text-sm font-mono text-zinc-200 placeholder-zinc-700 focus:outline-none focus:border-emerald-400/50 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
              />
              <span className="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-600 font-mono text-xs pointer-events-none">
                USDC
              </span>
            </div>
            <button
              type="button"
              onClick={handleDeposit}
              disabled={!connected || isBusy || !depositAmount || parseFloat(depositAmount) <= 0}
              className={`flex-shrink-0 flex items-center gap-2 px-5 py-3 rounded-lg border text-xs font-mono font-black uppercase tracking-wider transition-all duration-200 ${
                !connected
                  ? 'border-zinc-700/50 bg-zinc-900/50 text-zinc-500 cursor-not-allowed opacity-50'
                  : txStatus === 'success'
                  ? 'border-emerald-400/60 bg-emerald-400/20 text-emerald-300'
                  : txStatus === 'error'
                  ? 'border-red-400/60 bg-red-400/10 text-red-400'
                  : 'border-emerald-500/50 bg-emerald-500/15 text-emerald-400 hover:bg-emerald-500/25 hover:shadow-lg hover:shadow-emerald-500/20'
              } disabled:opacity-40 disabled:cursor-not-allowed`}
            >
              {isBusy ? (
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
              ) : txStatus === 'success' ? (
                <CheckCircle2 className="w-3.5 h-3.5" />
              ) : txStatus === 'error' ? (
                <AlertCircle className="w-3.5 h-3.5" />
              ) : (
                <ArrowDownToLine className="w-3.5 h-3.5" />
              )}
              <span className="hidden sm:inline">{depositBtnLabel()}</span>
            </button>
          </div>

          {!connected && (
            <p className="text-zinc-600 font-mono text-xs flex items-center gap-1.5">
              <Shield className="w-3 h-3" />
              {c.connectWallet}
            </p>
          )}

          {txStatus === 'success' && txSig && (
            <div className="border border-emerald-400/30 bg-emerald-400/5 rounded-lg px-3 py-2.5 font-mono text-[11px] space-y-1">
              <p className="text-emerald-400 font-bold">{c.depositSuccess}</p>
              <p className="text-zinc-500 break-all">
                {zh ? '签名：' : 'Sig: '}
                <span className="text-zinc-400">{txSig}</span>
              </p>
            </div>
          )}

          {txStatus === 'error' && txError && (
            <div className="border border-red-400/30 bg-red-400/5 rounded-lg px-3 py-2.5 font-mono text-[11px]">
              <p className="text-red-400 font-bold">{c.depositError}</p>
              <p className="text-zinc-500 mt-0.5 break-all">{txError}</p>
            </div>
          )}

          {/* Technical detail strip */}
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 pt-1 border-t border-zinc-800/50">
            {[
              {
                label: zh ? '结构偏移量' : 'Data Offset',
                value: `${RETAIL_RESERVE_OFFSET} bytes`,
                color: 'text-cyan-400',
              },
              {
                label: zh ? '字节序' : 'Endianness',
                value: 'LE u64',
                color: 'text-yellow-400',
              },
              {
                label: zh ? 'USDC 精度' : 'USDC Decimals',
                value: `${USDC_DECIMALS.toLocaleString()}`,
                color: 'text-emerald-400',
              },
            ].map((m) => (
              <div key={m.label} className="text-center">
                <p className="text-zinc-600 font-mono text-[9px] uppercase tracking-widest">{m.label}</p>
                <p className={`font-mono text-xs font-bold ${m.color} tabular-nums`}>{m.value}</p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
