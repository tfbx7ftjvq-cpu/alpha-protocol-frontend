import { type ElementType } from 'react';
import {
  CheckCircle2,
  Clock3,
  ExternalLink,
  FileSearch,
  Gavel,
  Loader2,
  LockKeyhole,
  RefreshCw,
  ShieldCheck,
  TimerReset,
  Vote,
} from 'lucide-react';
import { useSecurityGovernanceConfig } from '../hooks/useSecurityGovernanceConfig';
import { useSecurityGovernanceItems } from '../hooks/useSecurityGovernanceItems';
import {
  GOVERNANCE_CONFIG_PDA,
  SECURITY_LAYER_PROGRAM_ID,
  formatSecurityLayerDuration,
  formatSecurityLayerTimestamp,
  getActionTypeLabel,
  getExecutionStatusLabel,
  getProposalDecisionLabel,
  getProposalTypeLabel,
  getSecurityExplorerAddressUrl,
  type ExecutionQueueItemV1,
  type GovernanceConfigV1,
  type ProposalDecisionV1,
  type SecurityGovernanceItem,
} from '../lib/securityLayer';

const DAO_SCOPE = [
  'Green Label refund / slash',
  'Relief pool payout policy',
  'Treasury parameter changes',
  'Protocol fee split changes',
  'Risk exposure / blacklist evidence policy',
  'Staking reward policy',
  'Emergency pause review',
  'Contributor / builders pool spending',
];

const DAO_ROADMAP = [
  { label: 'Completed', value: 'Security Layer V1 execution guard', tone: 'emerald' },
  { label: 'Completed', value: 'Green Label refund/slash linked to Security Layer', tone: 'emerald' },
  { label: 'Current', value: 'Read-only DAO Governance Dashboard', tone: 'cyan' },
  { label: 'Next', value: 'DAO proposal product model', tone: 'yellow' },
  { label: 'Later', value: 'ALPHA voting power / quorum / threshold / delegation', tone: 'zinc' },
  { label: 'Later', value: 'DAO-controlled treasury and governance authority', tone: 'zinc' },
] as const;

export default function DAOGovernanceDashboard() {
  const {
    config,
    error: configError,
    lastLoadedAt: configLastLoadedAt,
    refresh: refreshConfig,
    status: configStatus,
  } = useSecurityGovernanceConfig();
  const {
    error: itemsError,
    items,
    lastLoadedAt: itemsLastLoadedAt,
    refresh: refreshItems,
    status: itemsStatus,
  } = useSecurityGovernanceItems();

  function refreshAll() {
    void Promise.all([refreshConfig(), refreshItems()]);
  }

  const isLoading = configStatus === 'loading' || itemsStatus === 'loading';

  return (
    <section className="space-y-5">
      <div className="rounded-xl border border-cyan-400/20 bg-cyan-400/5 p-5">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div className="min-w-0 space-y-3">
            <div className="flex flex-wrap items-center gap-2">
              <Badge icon={Gavel} label="DAO Execution Layer" tone="cyan" />
              <Badge icon={ShieldCheck} label="Devnet" tone="emerald" />
              <Badge icon={LockKeyhole} label="Read-only" tone="zinc" />
            </div>
            <div>
              <h3 className="text-xl font-black uppercase tracking-wide text-zinc-100">
                DAO Governance Dashboard
              </h3>
              <p className="mt-2 max-w-3xl text-xs leading-relaxed text-zinc-400">
                Productized read-only view of the Security Layer V1 execution guard. This page reads Devnet governance config, proposal decisions, and execution queue items without wallet signatures or write buttons.
              </p>
            </div>
          </div>

          <button
            type="button"
            onClick={refreshAll}
            disabled={isLoading}
            className="inline-flex items-center justify-center gap-2 rounded border border-cyan-400/30 bg-cyan-400/10 px-4 py-2 text-xs font-bold text-cyan-300 transition-all hover:bg-cyan-400/15 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {isLoading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <RefreshCw className="h-3.5 w-3.5" />}
            {isLoading ? 'Loading...' : 'Refresh DAO data'}
          </button>
        </div>

        {(configError || itemsError) && (
          <div className="mt-4 rounded border border-red-400/30 bg-red-400/10 px-3 py-2 text-xs leading-relaxed text-red-200">
            {configError && <p className="break-words">Governance config read failed: {configError}</p>}
            {itemsError && <p className="break-words">Proposal / queue read failed: {itemsError}</p>}
          </div>
        )}
      </div>

      <div className="grid grid-cols-1 gap-4 xl:grid-cols-[minmax(0,1.05fr)_minmax(0,0.95fr)]">
        <CoreStatusCard
          config={config}
          lastLoadedAt={configLastLoadedAt}
          status={configStatus}
        />
        <ExecutionLayerCard />
      </div>

      <VerifiedGovernancePaths items={items} status={itemsStatus} />

      <ProposalQueueTable
        items={items}
        lastLoadedAt={itemsLastLoadedAt}
        status={itemsStatus}
      />

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <DAOScopeCard />
        <DAORoadmapCard />
      </div>
    </section>
  );
}

function CoreStatusCard({
  config,
  lastLoadedAt,
  status,
}: {
  config: GovernanceConfigV1 | null;
  lastLoadedAt: Date | null;
  status: 'idle' | 'loading' | 'ready' | 'error';
}) {
  const rows = config
    ? [
        ['Program ID', SECURITY_LAYER_PROGRAM_ID.toBase58()],
        ['Governance config PDA', GOVERNANCE_CONFIG_PDA.toBase58()],
        ['authority', config.authority],
        ['emergency_guardian', config.emergencyGuardian],
        ['min_execution_delay_seconds', `${config.minExecutionDelaySeconds.toString()} (${formatSecurityLayerDuration(config.minExecutionDelaySeconds)})`],
        ['proposal_count', config.proposalCount.toString()],
        ['is_paused', config.isPaused ? 'true' : 'false'],
        ['bump', config.bump.toString()],
      ]
    : [
        ['Program ID', SECURITY_LAYER_PROGRAM_ID.toBase58()],
        ['Governance config PDA', GOVERNANCE_CONFIG_PDA.toBase58()],
      ];

  return (
    <div className="rounded-xl border border-emerald-400/20 bg-emerald-400/5 p-5">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <SectionHeader
          icon={ShieldCheck}
          eyebrow="DAO Core Status"
          title="Security Layer governance config"
          description="Read-only decode of GovernanceConfigV1 on Solana Devnet."
          tone="text-emerald-300"
        />
        <StatusPill status={status} />
      </div>

      <div className="mt-4 grid grid-cols-1 gap-2">
        {rows.map(([label, value]) => (
          <KeyValueRow key={label} label={label} value={value} />
        ))}
      </div>

      {lastLoadedAt && (
        <p className="mt-3 text-[10px] font-bold text-zinc-600">
          Last loaded: {lastLoadedAt.toLocaleTimeString('zh-CN')}
        </p>
      )}
    </div>
  );
}

function ExecutionLayerCard() {
  const items = [
    'Completed layer: DAO execution / Security Layer V1.',
    'Responsible for proposal decision, queue, timelock, execute, cancel, and pause.',
    'Full ALPHA token voting layer is not open yet.',
    'This UI is read-only and does not provide vote, queue, execute, cancel, or pause buttons.',
  ];

  return (
    <div className="rounded-xl border border-zinc-800 bg-zinc-950/70 p-5">
      <SectionHeader
        icon={LockKeyhole}
        eyebrow="Governance Execution Layer"
        title="Execution guard, not full voting yet"
        description="The current Devnet milestone verifies controlled execution paths. Voting product and ALPHA voting power are later phases."
        tone="text-cyan-300"
      />
      <div className="mt-4 space-y-2">
        {items.map((item) => (
          <div key={item} className="flex items-start gap-2 rounded border border-zinc-800 bg-zinc-950/80 px-3 py-2 text-xs leading-relaxed text-zinc-300">
            <CheckCircle2 className="mt-0.5 h-3.5 w-3.5 flex-shrink-0 text-cyan-300" />
            <span>{item}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function VerifiedGovernancePaths({
  items,
  status,
}: {
  items: SecurityGovernanceItem[];
  status: 'idle' | 'loading' | 'ready' | 'error';
}) {
  const cards = items.length > 0 ? items : [];

  return (
    <div className="rounded-xl border border-blue-400/20 bg-blue-400/5 p-5">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <SectionHeader
          icon={FileSearch}
          eyebrow="Verified Devnet Governance Paths"
          title="Verified proposal / queue paths"
          description="Historical Devnet proposal IDs are read from PDAs when available. Missing historical accounts are shown as unavailable rather than crashing the page."
          tone="text-blue-300"
        />
        <StatusPill status={status} />
      </div>

      {status === 'loading' && cards.length === 0 ? (
        <div className="mt-4 flex items-center gap-2 rounded border border-zinc-800 bg-zinc-950/70 px-3 py-3 text-xs font-bold text-zinc-400">
          <Loader2 className="h-4 w-4 animate-spin text-blue-300" />
          Reading proposal and queue PDAs...
        </div>
      ) : (
        <div className="mt-4 grid grid-cols-1 gap-3 lg:grid-cols-2">
          {cards.map((item) => (
            <div key={item.target.proposalId} className="rounded border border-zinc-800 bg-zinc-950/75 p-4">
              <div className="flex items-start justify-between gap-3">
                <div>
                  <p className="text-[10px] font-black uppercase tracking-widest text-zinc-600">
                    proposal_id {item.target.proposalId}
                  </p>
                  <p className="mt-1 text-sm font-black text-zinc-100">{item.target.expectedPathLabel}</p>
                  <p className="mt-1 text-xs text-zinc-500">{item.target.description}</p>
                </div>
                <PathStatusBadge item={item} />
              </div>
              <div className="mt-3 grid grid-cols-1 gap-2 sm:grid-cols-2">
                <MiniStatus label="Proposal" value={item.proposalDecision ? item.proposalDecision.decision : item.proposalError ?? 'unavailable'} />
                <MiniStatus label="Queue" value={item.executionQueueItem ? item.executionQueueItem.status : item.queueError ?? 'unavailable'} />
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function ProposalQueueTable({
  items,
  lastLoadedAt,
  status,
}: {
  items: SecurityGovernanceItem[];
  lastLoadedAt: Date | null;
  status: 'idle' | 'loading' | 'ready' | 'error';
}) {
  return (
    <div className="rounded-xl border border-zinc-800 bg-zinc-950/70 p-5">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <SectionHeader
          icon={Vote}
          eyebrow="Proposal / Queue Read-only Table"
          title="DAO execution records"
          description="Raw decoded ProposalDecisionV1 and ExecutionQueueItemV1 accounts for proposal IDs 1, 3, 4, and 5."
          tone="text-zinc-300"
        />
        <div className="flex flex-col items-start gap-2 sm:items-end">
          <StatusPill status={status} />
          {lastLoadedAt && (
            <p className="text-[10px] font-bold text-zinc-600">
              Last loaded: {lastLoadedAt.toLocaleTimeString('zh-CN')}
            </p>
          )}
        </div>
      </div>

      <div className="mt-4 overflow-x-auto">
        <table className="min-w-[1100px] w-full text-left font-mono text-xs">
          <thead>
            <tr className="border-b border-zinc-800 text-[10px] uppercase tracking-widest text-zinc-600">
              <th className="px-3 py-2">proposal_id</th>
              <th className="px-3 py-2">proposal type</th>
              <th className="px-3 py-2">decision</th>
              <th className="px-3 py-2">execution status</th>
              <th className="px-3 py-2">action type</th>
              <th className="px-3 py-2">payload hash</th>
              <th className="px-3 py-2">queue item</th>
              <th className="px-3 py-2">execute_after</th>
              <th className="px-3 py-2">executed / cancelled</th>
              <th className="px-3 py-2">expected path</th>
              <th className="px-3 py-2">explorer</th>
            </tr>
          </thead>
          <tbody>
            {items.map((item) => (
              <GovernanceTableRow key={item.target.proposalId} item={item} />
            ))}
          </tbody>
        </table>
      </div>

      <div className="mt-4 grid grid-cols-1 gap-3">
        {items.map((item) => (
          <RawGovernanceDetails key={`raw-${item.target.proposalId}`} item={item} />
        ))}
      </div>
    </div>
  );
}

function GovernanceTableRow({ item }: { item: SecurityGovernanceItem }) {
  const proposal = item.proposalDecision;
  const queue = item.executionQueueItem;

  return (
    <tr className="border-b border-zinc-800/70 align-top text-zinc-300">
      <td className="px-3 py-3 font-black text-zinc-100">{item.target.proposalId}</td>
      <td className="px-3 py-3">{proposal ? getProposalTypeLabel(proposal.proposalType) : item.proposalError ?? 'unavailable'}</td>
      <td className="px-3 py-3">{proposal ? getProposalDecisionLabel(proposal.decision) : 'unavailable'}</td>
      <td className="px-3 py-3">{queue ? getExecutionStatusLabel(queue.status) : item.queueError ?? 'unavailable'}</td>
      <td className="px-3 py-3">{queue ? getActionTypeLabel(queue.actionType) : 'unavailable'}</td>
      <td className="max-w-[220px] px-3 py-3 break-all text-[10px] text-zinc-500">{queue?.payloadHash ?? 'unavailable'}</td>
      <td className="max-w-[180px] px-3 py-3 break-all text-[10px] text-cyan-300">{shortAddress(item.executionQueueItemPda)}</td>
      <td className="px-3 py-3">{queue ? formatTimestampCell(queue.executeAfter) : 'unavailable'}</td>
      <td className="px-3 py-3">{queue ? formatTerminalTime(queue) : 'unavailable'}</td>
      <td className="px-3 py-3">{item.target.expectedPathLabel}</td>
      <td className="px-3 py-3">
        <a
          href={getSecurityExplorerAddressUrl(item.executionQueueItemPda)}
          target="_blank"
          rel="noreferrer"
          className="inline-flex items-center gap-1.5 text-cyan-300 hover:text-cyan-200"
        >
          queue
          <ExternalLink className="h-3 w-3" />
        </a>
      </td>
    </tr>
  );
}

function RawGovernanceDetails({ item }: { item: SecurityGovernanceItem }) {
  return (
    <details className="rounded border border-zinc-800 bg-zinc-950/75 p-3">
      <summary className="cursor-pointer text-xs font-black text-zinc-300 hover:text-cyan-300">
        Raw accounts for proposal_id {item.target.proposalId}
      </summary>
      <div className="mt-3 grid grid-cols-1 gap-3 xl:grid-cols-2">
        <RawAccountPanel
          title="ProposalDecisionV1"
          address={item.proposalDecisionPda}
          rows={item.proposalDecision ? proposalRows(item.proposalDecision) : [['status', item.proposalError ?? 'unavailable']]}
        />
        <RawAccountPanel
          title="ExecutionQueueItemV1"
          address={item.executionQueueItemPda}
          rows={item.executionQueueItem ? queueRows(item.executionQueueItem) : [['status', item.queueError ?? 'unavailable']]}
        />
      </div>
    </details>
  );
}

function DAOScopeCard() {
  return (
    <div className="rounded-xl border border-emerald-400/20 bg-emerald-400/5 p-5">
      <SectionHeader
        icon={Gavel}
        eyebrow="DAO Scope"
        title="Future governance scope"
        description="Areas that should move through DAO proposal, queue, timelock, and execution controls."
        tone="text-emerald-300"
      />
      <div className="mt-4 grid grid-cols-1 gap-2 sm:grid-cols-2">
        {DAO_SCOPE.map((item) => (
          <div key={item} className="rounded border border-zinc-800 bg-zinc-950/75 px-3 py-2 text-xs font-bold text-zinc-300">
            {item}
          </div>
        ))}
      </div>
    </div>
  );
}

function DAORoadmapCard() {
  return (
    <div className="rounded-xl border border-yellow-400/20 bg-yellow-400/5 p-5">
      <SectionHeader
        icon={TimerReset}
        eyebrow="DAO Roadmap"
        title="From execution guard to voting layer"
        description="The read-only dashboard makes the execution layer visible before the full voting product is opened."
        tone="text-yellow-300"
      />
      <div className="mt-4 space-y-2">
        {DAO_ROADMAP.map((item) => (
          <div key={`${item.label}-${item.value}`} className="grid grid-cols-1 gap-1 rounded border border-zinc-800 bg-zinc-950/75 px-3 py-2 text-xs sm:grid-cols-[140px_minmax(0,1fr)]">
            <span className={`font-black uppercase tracking-widest ${roadmapToneClass(item.tone)}`}>{item.label}</span>
            <span className="font-bold text-zinc-300">{item.value}</span>
          </div>
        ))}
      </div>
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
  tone,
}: {
  icon: ElementType;
  eyebrow: string;
  title: string;
  description: string;
  tone: string;
}) {
  return (
    <div>
      <div className={`mb-2 flex items-center gap-2 text-[10px] font-black uppercase tracking-widest ${tone}`}>
        <Icon className="h-3.5 w-3.5" />
        {eyebrow}
      </div>
      <h3 className="text-lg font-black text-zinc-100">{title}</h3>
      <p className="mt-2 text-xs leading-relaxed text-zinc-400">{description}</p>
    </div>
  );
}

function StatusPill({ status }: { status: 'idle' | 'loading' | 'ready' | 'error' }) {
  const meta = {
    idle: { label: 'Idle', className: 'border-zinc-700 bg-zinc-900/60 text-zinc-400' },
    loading: { label: 'Loading', className: 'border-yellow-400/30 bg-yellow-400/10 text-yellow-300' },
    ready: { label: 'Ready', className: 'border-emerald-400/30 bg-emerald-400/10 text-emerald-300' },
    error: { label: 'Error', className: 'border-red-400/30 bg-red-400/10 text-red-300' },
  }[status];

  return (
    <span className={`inline-flex items-center gap-1.5 rounded border px-2 py-1 text-[10px] font-black uppercase tracking-widest ${meta.className}`}>
      {status === 'loading' ? <Loader2 className="h-3 w-3 animate-spin" /> : <Clock3 className="h-3 w-3" />}
      {meta.label}
    </span>
  );
}

function PathStatusBadge({ item }: { item: SecurityGovernanceItem }) {
  const hasBoth = Boolean(item.proposalDecision && item.executionQueueItem);
  const className = hasBoth
    ? 'border-emerald-400/30 bg-emerald-400/10 text-emerald-300'
    : 'border-yellow-400/35 bg-yellow-400/10 text-yellow-200';

  return (
    <span className={`flex-shrink-0 rounded border px-2 py-1 text-[10px] font-black uppercase tracking-widest ${className}`}>
      {hasBoth ? 'On-chain' : 'Unavailable'}
    </span>
  );
}

function MiniStatus({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded border border-zinc-800 bg-zinc-950/80 px-3 py-2">
      <p className="text-[10px] font-black uppercase tracking-widest text-zinc-600">{label}</p>
      <p className="mt-1 break-all font-mono text-xs font-bold text-zinc-200">{value}</p>
    </div>
  );
}

function KeyValueRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid grid-cols-1 gap-1 rounded border border-zinc-800 bg-zinc-950/75 px-3 py-2 text-xs sm:grid-cols-[minmax(0,0.75fr)_minmax(0,1.25fr)]">
      <span className="break-all font-mono font-bold text-zinc-500">{label}</span>
      <span className="min-w-0 break-all font-mono font-black text-zinc-100 sm:text-right">{value}</span>
    </div>
  );
}

function RawAccountPanel({
  address,
  rows,
  title,
}: {
  address: string;
  rows: [string, string][];
  title: string;
}) {
  return (
    <div className="rounded border border-zinc-800 bg-zinc-950/80 p-3">
      <div className="mb-3 flex items-center justify-between gap-3">
        <p className="text-[10px] font-black uppercase tracking-widest text-zinc-500">{title}</p>
        <a
          href={getSecurityExplorerAddressUrl(address)}
          target="_blank"
          rel="noreferrer"
          className="inline-flex items-center gap-1.5 text-[10px] font-bold text-cyan-300 hover:text-cyan-200"
        >
          {shortAddress(address)}
          <ExternalLink className="h-3 w-3" />
        </a>
      </div>
      <div className="space-y-2">
        {rows.map(([label, value]) => (
          <KeyValueRow key={label} label={label} value={value} />
        ))}
      </div>
    </div>
  );
}

function proposalRows(proposal: ProposalDecisionV1): [string, string][] {
  return [
    ['account', proposal.account],
    ['proposal_id', proposal.proposalId.toString()],
    ['proposal_type', getProposalTypeLabel(proposal.proposalType)],
    ['proposer', proposal.proposer],
    ['decision', getProposalDecisionLabel(proposal.decision)],
    ['yes_weight', proposal.yesWeight.toString()],
    ['no_weight', proposal.noWeight.toString()],
    ['start_ts', formatTimestampCell(proposal.startTs)],
    ['end_ts', formatTimestampCell(proposal.endTs)],
    ['finalized_ts', formatTimestampCell(proposal.finalizedTs)],
    ['bump', proposal.bump.toString()],
  ];
}

function queueRows(queue: ExecutionQueueItemV1): [string, string][] {
  return [
    ['account', queue.account],
    ['proposal_id', queue.proposalId.toString()],
    ['proposer', queue.proposer],
    ['action_type', getActionTypeLabel(queue.actionType)],
    ['target_program', queue.targetProgram],
    ['target_account', queue.targetAccount],
    ['decision', getProposalDecisionLabel(queue.decision)],
    ['created_at', formatTimestampCell(queue.createdAt)],
    ['execute_after', formatTimestampCell(queue.executeAfter)],
    ['executed_at', formatTimestampCell(queue.executedAt)],
    ['status', getExecutionStatusLabel(queue.status)],
    ['payload_hash', queue.payloadHash],
    ['bump', queue.bump.toString()],
  ];
}

function formatTimestampCell(timestamp: bigint): string {
  return `${timestamp.toString()} / ${formatSecurityLayerTimestamp(timestamp)}`;
}

function formatTerminalTime(queue: ExecutionQueueItemV1): string {
  if (queue.status === 'Cancelled') {
    return queue.executedAt === 0n ? 'cancelled / timestamp unavailable' : formatTimestampCell(queue.executedAt);
  }

  if (queue.status === 'Executed') {
    return formatTimestampCell(queue.executedAt);
  }

  return 'not executed';
}

function roadmapToneClass(tone: typeof DAO_ROADMAP[number]['tone']): string {
  const className = {
    emerald: 'text-emerald-300',
    cyan: 'text-cyan-300',
    yellow: 'text-yellow-300',
    zinc: 'text-zinc-500',
  }[tone];

  return className;
}

function shortAddress(address: string): string {
  if (address.length <= 14) return address;
  return `${address.slice(0, 5)}...${address.slice(-5)}`;
}
