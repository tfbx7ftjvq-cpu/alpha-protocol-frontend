# Alpha Protocol Green Label V1 Design

Last updated: 2026-06-27

## 1. Overview

Green Label V1 is a Devnet design for adding a risk commitment layer on top of Alpha Protocol's existing Treasury V2, Alpha Guardian Staking V1, and System Security Layer V1 foundation.

The goal is simple: a project voluntarily locks a Project Bond, enters an observation period, and can later receive a refund or face a slash decision through the Security Layer. Users can open disputes with evidence hashes. DAO or Security Layer governance can create a decision and queued action, but value movement only happens after the execution queue and timelock complete.

Current completed foundation:

- Treasury V2: real Devnet USDC treasury routing with 50/20/20/10 split.
- Alpha Guardian Staking V1: ALPHA stake, USDC claim, and unstake flow.
- System Security Layer V1: decision -> queue -> timelock -> execute.
- Security Layer V1 Devnet verification:
  - happy path execution succeeded.
  - pause blocks execute.
  - cancel blocks execute.

Green Label V1 should use the following product language:

- Project Bond
- Risk Bond
- Bond Tier
- 风险承诺金
- 风险承诺等级

Green Label V1 should not use "credit rating" or "信用等级" as the core concept. The Project Bond is an economic commitment by the project, not a safety certificate.

## 2. Legal and Messaging Boundaries

Green Label V1 must be presented with clear boundaries:

- Not insurance.
- Not investment advice.
- Not a credit rating.
- Not a guarantee of project safety.
- Bond Tier only reflects economic commitment.
- No fixed compensation promise.
- No promise that Green Label projects cannot rug.

Recommended English frontend copy:

- "Green Label indicates that a project has locked a Project Bond with Alpha Protocol. It is not insurance, investment advice, a credit rating, or a guarantee of safety."
- "Bond Tier reflects the amount of economic commitment provided by the project. A higher Bond Tier does not mean the project is risk-free."
- "If a valid dispute or malicious behavior is confirmed through the Security Layer, the Project Bond may be slashed after queueing and timelock execution."
- "Alpha Protocol does not promise fixed compensation and does not guarantee that Green Label projects cannot fail or rug."

Recommended Chinese frontend copy:

- "Green Label 表示项目方已在 Alpha Protocol 锁定 Project Bond（风险承诺金）。它不是保险、不是投资建议、不是信用评级，也不保证项目安全。"
- "Bond Tier（风险承诺等级）只反映项目方提供的经济承诺金额。更高的 Bond Tier 不代表项目没有风险。"
- "如果有效 dispute 或恶意行为经 Security Layer 确认，Project Bond 可能在进入执行队列并经过 timelock 后被罚没。"
- "Alpha Protocol 不承诺固定赔付，也不保证 Green Label 项目不会失败或 rug。"

Preferred UI labels:

- English: "Project Bond", "Bond Tier", "Risk Commitment", "Dispute Window", "Observation Period".
- Chinese: "风险承诺金", "风险承诺等级", "观察期", "争议期", "Security Layer 决策中".

Avoid:

- "Guaranteed safe"
- "Insured"
- "Credit rating"
- "Risk-free"
- "Fixed compensation"
- "官方背书安全项目"
- "保本赔付"

## 3. Bond Model

Green Label V1 uses a two-part Project Bond model:

- Minimum Bond: 299 USDC.
- Additional Bond: uncapped voluntary extra bond.
- Base Bond is the configured base portion, 299 USDC in Phase 1. Any voluntary amount above the base portion must be represented as Extra Bond.
- Base Bond refund:
  - 80% refund to the project owner.
  - 20% routed to the protocol treasury.
- Extra Bond refund:
  - 100% refund if there is no valid dispute, confirmed rug, or confirmed malicious behavior.
- Slash:
  - the full bond can be slashed if rug or malicious behavior is confirmed.
  - the slashable amount includes both the 299 USDC base bond and any extra bond.

All amounts must use USDC 6 decimals. The design should store raw integer token amounts, for example `299_000_000` for 299 USDC.

Example A: 299 USDC project

- Project locks 299 USDC.
- Base bond amount: 299 USDC.
- Extra bond amount: 0 USDC.
- If no valid dispute or rug is confirmed:
  - 239.2 USDC returns to the project owner.
  - 59.8 USDC goes to the treasury-controlled vault.
- If rug or malicious behavior is confirmed:
  - up to the full 299 USDC can be slashed.

Example B: 1299 USDC project

- Project locks 1299 USDC.
- Base bond amount: 299 USDC.
- Extra bond amount: 1000 USDC.
- If no valid dispute or rug is confirmed:
  - 239.2 USDC from the base bond returns to the project owner.
  - 59.8 USDC from the base bond goes to the treasury-controlled vault.
  - 1000 USDC extra bond returns to the project owner.
- If rug or malicious behavior is confirmed:
  - up to the full 1299 USDC can be slashed.

The extra bond exists to increase the project's downside if it acts maliciously. It must not be marketed as buying safety, buying reputation, or buying a guaranteed approval outcome.

## 4. Bond Tier

Bond Tier is a simple display and policy tier derived from total Project Bond amount:

| Tier | Minimum Total Bond | Meaning |
| --- | ---: | --- |
| Base | 299 USDC | Minimum participation threshold |
| Bronze | 500 USDC+ | Higher voluntary risk commitment |
| Silver | 1000 USDC+ | Higher voluntary risk commitment |
| Gold | 3000 USDC+ | Higher voluntary risk commitment |
| Platinum | 10000 USDC+ | Higher voluntary risk commitment |
| Custom | Higher amount | Custom project-level commitment |

Bond Tier is not a safety level and is not a credit rating. It only represents the economic commitment locked by the project.

Design rules:

- Bond Tier should be derived from `total_bond_amount`.
- Bond Tier should be displayed alongside disclaimers.
- Bond Tier should be only one input to Alpha Risk Score.
- A project must not be allowed to imply that Platinum means safe, audited, insured, or endorsed.
- A Custom tier can be shown for very high commitments, but the UI should still show the actual USDC amount.

## 5. Risk Score Model

Green Label V1 can reserve fields for an Alpha Risk Score, but Phase 1 should not implement complex automatic scoring. Phase 1 should store a snapshot field and allow future expansion after the account model is stable.

Suggested future score components:

- Bond Tier / Bond Amount
- On-chain permission safety
- LP / liquidity risk
- Project age
- Transparency
- Dispute history
- Audit / third-party review

Bond must only be part of the score. The protocol must avoid a "pay to buy green label" model.

Recommended Phase 1 representation:

- `risk_score_snapshot`: optional or defaulted score field.
- `risk_score_version`: optional future field if account space allows.
- `risk_score_updated_at`: optional future field if account space allows.

Suggested future weighting principle:

| Component | Directional Use |
| --- | --- |
| Bond amount | Measures economic commitment, not safety |
| Permission safety | Checks mint, freeze, upgrade, and admin controls |
| Liquidity risk | Checks LP depth, concentration, and withdrawal risk |
| Project age | Rewards longer observable history |
| Transparency | Rewards clear disclosures and public information |
| Dispute history | Penalizes unresolved or confirmed disputes |
| Audit review | Adds context from third-party review |

Phase 1 should not use automated rug detection, oracle feeds, or off-chain crawlers to compute the score.

## 6. Account Design

The following field lists are Rust-like / Anchor-like design sketches. They are not formal contract code.

### GreenLabelConfigV1

```text
GreenLabelConfigV1 {
  authority: Pubkey,
  usdc_mint: Pubkey,
  min_base_bond_usdc: u64,
  base_refund_bps: u16,
  base_treasury_bps: u16,
  observation_period_seconds: i64,
  dispute_window_seconds: i64,
  response_window_seconds: i64,
  project_count: u64,
  treasury_usdc_state_v2: Pubkey,
  base_bond_treasury_vault: Pubkey,
  relief_usdc_vault: Pubkey,
  builders_usdc_vault: Pubkey,
  staking_usdc_vault: Pubkey,
  buyback_usdc_vault: Pubkey,
  green_bond_risk_reserve_vault: Pubkey,
  security_governance_config: Pubkey,
  is_paused: bool,
  bump: u8,
}
```

Notes:

- `min_base_bond_usdc` should default to 299 USDC in 6-decimal units.
- `base_refund_bps` should default to 8000.
- `base_treasury_bps` should default to 2000.
- `base_refund_bps + base_treasury_bps` must equal 10000.
- `base_bond_treasury_vault` receives the base bond 20% treasury share during refund execution.
- `security_governance_config` links Green Label V1 to System Security Layer V1.
- Relevant treasury vaults should point to Treasury V2-controlled accounts where possible.

### GreenLabelProjectV1

```text
GreenLabelProjectV1 {
  project_id: u64,
  project_owner: Pubkey,
  project_name_hash: [u8; 32],
  project_url_hash: [u8; 32],
  token_mint: Pubkey,
  project_treasury_wallet: Pubkey,
  base_bond_amount: u64,
  extra_bond_amount: u64,
  total_bond_amount: u64,
  bond_vault: Pubkey,
  bond_vault_authority: Pubkey,
  bond_tier: BondTier,
  status: GreenLabelStatus,
  submitted_at: i64,
  observation_start_ts: i64,
  observation_end_ts: i64,
  dispute_count: u64,
  active_dispute: Option<Pubkey>,
  terminal_action_proposal_id: Option<u64>,
  terminal_action_proposal_decision: Option<Pubkey>,
  terminal_action_execution_queue_item: Option<Pubkey>,
  terminal_action_payload_hash: Option<[u8; 32]>,
  approved_at: Option<i64>,
  refunded_at: Option<i64>,
  slashed_at: Option<i64>,
  risk_score_snapshot: u16,
  bump: u8,
}
```

Notes:

- `project_name_hash` and `project_url_hash` should be stable hashes of normalized off-chain strings.
- `bond_vault` must be an isolated Green Bond Vault for the project.
- `bond_vault_authority` should be PDA-controlled.
- `active_dispute` should prevent multiple simultaneous disputes in Phase 1.
- Terminal action fields store the Security Layer decision and queue link for the final refund or slash. This also supports a no-active-dispute refund after observation while still requiring Security Layer decision, execution queue, payload hash, and timelock checks.
- `status` drives whether refund, slash, dispute, or approval transitions are allowed.

### GreenLabelDisputeV1

```text
GreenLabelDisputeV1 {
  project_id: u64,
  dispute_id: u64,
  project: Pubkey,
  disputer: Pubkey,
  reason_code: RugReasonCode,
  evidence_hash: [u8; 32],
  status: DisputeStatus,
  opened_at: i64,
  evidence_end_ts: i64,
  response_end_ts: i64,
  resolved_at: Option<i64>,
  proposal_id: Option<u64>,
  proposal_decision: Option<Pubkey>,
  execution_queue_item: Option<Pubkey>,
  bump: u8,
}
```

Notes:

- Large evidence content must stay off-chain; only the hash is stored on-chain.
- `proposal_id`, `proposal_decision`, and `execution_queue_item` are populated when the dispute links to System Security Layer V1.
- `status` should prevent executing refund or slash before the Security Layer path is complete.

## 7. Enums

The following enum lists are design sketches, not formal code.

### GreenLabelStatus

```text
GreenLabelStatus {
  PendingObservation,
  ObservationPassed,
  Disputed,
  PendingSecurityDecision,
  QueuedRefund,
  QueuedSlash,
  Refunded,
  Slashed,
  Cancelled,
}
```

Suggested meaning:

- `PendingObservation`: project submitted bond and is inside observation period.
- `ObservationPassed`: observation period passed without active dispute.
- `Disputed`: at least one active dispute exists.
- `PendingSecurityDecision`: dispute is ready and waiting for Security Layer decision.
- `QueuedRefund`: refund action is queued in the Security Layer.
- `QueuedSlash`: slash action is queued in the Security Layer.
- `Refunded`: final refund executed.
- `Slashed`: final slash executed.
- `Cancelled`: application cancelled before final approval where allowed.

### BondTier

```text
BondTier {
  Base,
  Bronze,
  Silver,
  Gold,
  Platinum,
  Custom,
}
```

### RugReasonCode

```text
RugReasonCode {
  LiquidityRemoved,
  DeveloperDump,
  WebsiteOrCommunityAbandoned,
  MintOrFreezeAuthorityAbuse,
  TreasuryMisuse,
  FalseDisclosure,
  MaliciousContractUpgrade,
  Other,
}
```

### DisputeStatus

```text
DisputeStatus {
  Open,
  EvidencePeriod,
  AwaitingProjectResponse,
  ReadyForDecision,
  QueuedRefund,
  QueuedSlash,
  ResolvedRefund,
  ResolvedSlash,
  Rejected,
  Cancelled,
}
```

## 8. Instruction Design

The following Phase 1 instructions are design-level specifications only. They are not formal implementation code.

### initialize_green_label_config

Purpose:

- Create the global Green Label V1 configuration.
- Link Green Label V1 to Treasury V2 vaults and System Security Layer V1 governance config.

Accounts:

- `green_label_config`
- `authority`
- `usdc_mint`
- `treasury_usdc_state_v2`
- relevant Treasury V2 vaults
- `security_governance_config`
- `system_program`

Args:

- `min_base_bond_usdc`
- `base_refund_bps`
- `base_treasury_bps`
- `observation_period_seconds`
- `dispute_window_seconds`
- `response_window_seconds`

Validation rules:

- Authority must sign.
- `min_base_bond_usdc` must be at least 299 USDC in 6-decimal units.
- `base_refund_bps + base_treasury_bps` must equal 10000.
- Default split should be 8000 / 2000.
- Treasury V2 and Security Layer accounts must match expected configured addresses.

State transitions:

- Creates config with `is_paused = false`.
- Initializes `project_count = 0`.

Funds movement:

- None.

### submit_green_label_application

Purpose:

- Register a project and lock its Project Bond in an isolated Green Bond Vault.

Accounts:

- `green_label_config`
- `green_label_project`
- `project_owner`
- `project_owner_usdc_ata`
- `bond_vault`
- `bond_vault_authority`
- `usdc_mint`
- `token_mint`
- `token_program`
- `system_program`

Args:

- `project_name_hash`
- `project_url_hash`
- `token_mint`
- `project_treasury_wallet`
- `base_bond_amount`
- `extra_bond_amount`

Validation rules:

- Config must not be paused.
- Project owner must sign.
- `base_bond_amount` must equal the configured base bond amount in Phase 1.
- `extra_bond_amount` may be zero and is uncapped.
- `total_bond_amount = base_bond_amount + extra_bond_amount`.
- `total_bond_amount` must be at least `min_base_bond_usdc`.
- Bond Tier must be derived from total bond amount.
- Bond must be transferred into the project's isolated Green Bond Vault.
- New project must start without active dispute.

State transitions:

- Creates `GreenLabelProjectV1`.
- Sets status to `PendingObservation`.
- Sets `submitted_at`, `observation_start_ts`, and `observation_end_ts`.
- Increments `project_count`.

Funds movement:

- `project_owner_usdc_ata -> bond_vault`.

### open_green_label_dispute

Purpose:

- Allow a user to open a dispute against a project during the allowed dispute window.
- Store evidence hash and reason code.

Accounts:

- `green_label_config`
- `green_label_project`
- `green_label_dispute`
- `disputer`
- `system_program`

Args:

- `project_id`
- `reason_code`
- `evidence_hash`

Validation rules:

- Config must not be paused.
- Disputer must sign.
- Project must be in a disputable state.
- Phase 1 allows only one active dispute per project.
- `evidence_hash` must not be all zeroes.
- Current timestamp must be within the allowed observation or dispute window.

State transitions:

- Project status moves to `Disputed`.
- Project `active_dispute` is set.
- Project `dispute_count` increments.
- Dispute status starts as `EvidencePeriod` or `Open`.
- Sets `opened_at`, `evidence_end_ts`, and `response_end_ts`.

Funds movement:

- None in Phase 1.

### mark_dispute_ready_for_decision

Purpose:

- Move a dispute into a state where DAO or Security Layer governance can make a decision.

Accounts:

- `green_label_config`
- `green_label_project`
- `green_label_dispute`
- `authority_or_dispute_admin`

Args:

- `project_id`
- `dispute_id`

Validation rules:

- Config must not be paused.
- Caller must be authorized by governance policy.
- Dispute must be active.
- Evidence and response windows must be complete, or governance must explicitly mark the dispute ready under emergency policy.

State transitions:

- Dispute status moves to `ReadyForDecision`.
- Project status moves to `PendingSecurityDecision`.

Funds movement:

- None.

### link_security_decision

Purpose:

- Link a Green Label dispute to a verified System Security Layer decision and queued action.

Accounts:

- `green_label_config`
- `green_label_project`
- `green_label_dispute` when linking a dispute-driven action
- `security_governance_config`
- `proposal_decision`
- `execution_queue_item`
- `authority_or_security_adapter`

Args:

- `project_id`
- `dispute_id`
- `proposal_id`
- `expected_action_type`
- `expected_payload_hash`

Validation rules:

- Config must not be paused.
- For dispute-driven actions, dispute must be `ReadyForDecision`.
- For a no-active-dispute refund path, project must be `ObservationPassed` and `active_dispute` must be empty.
- `GreenLabelSlash` always requires a linked dispute account.
- `proposal_decision` must belong to the configured Security Layer program.
- `execution_queue_item` must belong to the same `proposal_id`.
- Security decision must be approved for the requested action.
- Action type must be `GreenLabelRefund` or `GreenLabelSlash`.
- `payload_hash` must match the canonical Green Label payload.
- Queued action must not be cancelled or already executed.

State transitions:

- Stores `proposal_id`, `proposal_decision`, `execution_queue_item`, and `payload_hash` on the project terminal action fields.
- For dispute-driven actions, also stores `proposal_id`, `proposal_decision`, and `execution_queue_item` on the dispute.
- If action is `GreenLabelRefund`, project moves to `QueuedRefund`; a linked dispute also moves to `QueuedRefund`.
- If action is `GreenLabelSlash`, dispute moves to `QueuedSlash` and project moves to `QueuedSlash`.

Funds movement:

- None.

### execute_green_label_refund

Purpose:

- Execute the final project refund after Security Layer queue and timelock requirements have been satisfied.

Accounts:

- `green_label_config`
- `green_label_project`
- `green_label_dispute` when refund resolves a dispute
- `security_governance_config`
- `proposal_decision`
- `execution_queue_item`
- `bond_vault`
- `bond_vault_authority`
- `project_owner_usdc_ata`
- `treasury_usdc_vault`
- `token_program`

Args:

- `project_id`
- `dispute_id` when refund resolves a dispute
- `payload_hash`

Validation rules:

- Config must not be paused.
- Project must be `QueuedRefund`.
- If a dispute account is supplied, dispute must be `QueuedRefund`.
- If no dispute account is supplied, project must have no active dispute and must have passed observation.
- Security Layer action type must be `GreenLabelRefund`.
- Security Layer timelock must be satisfied.
- Security Layer must not be paused or otherwise blocking execution.
- Execution queue item must be in executable status.
- `payload_hash` must match the canonical refund payload.
- Bond vault balance must cover the calculated refund and treasury amounts.

State transitions:

- Project status moves to `Refunded`.
- If a dispute account is supplied, dispute status moves to `ResolvedRefund` and `resolved_at` is set.
- Sets `refunded_at`.
- Clears `active_dispute` when present.

Funds movement:

- Base bond 80%: `bond_vault -> project_owner_usdc_ata`.
- Base bond 20%: `bond_vault -> treasury_usdc_vault`.
- Extra bond 100%: `bond_vault -> project_owner_usdc_ata`.

### execute_green_label_slash

Purpose:

- Execute a slash after Security Layer queue and timelock requirements have been satisfied.

Accounts:

- `green_label_config`
- `green_label_project`
- `green_label_dispute`
- `security_governance_config`
- `proposal_decision`
- `execution_queue_item`
- `bond_vault`
- `bond_vault_authority`
- `risk_reserve_or_relief_vault`
- `token_program`

Args:

- `project_id`
- `dispute_id`
- `payload_hash`
- `slash_amount`

Validation rules:

- Config must not be paused.
- Project must be `QueuedSlash`.
- Dispute must be `QueuedSlash`.
- Security Layer action type must be `GreenLabelSlash`.
- Security Layer timelock must be satisfied.
- Security Layer must not be paused or otherwise blocking execution.
- Execution queue item must be in executable status.
- `payload_hash` must match the canonical slash payload.
- `slash_amount` must not exceed bond vault balance.
- Full bond can be slashed if rug or malicious behavior is confirmed.

State transitions:

- Project status moves to `Slashed`.
- Dispute status moves to `ResolvedSlash`.
- Sets `slashed_at` and `resolved_at`.
- Clears `active_dispute`.

Funds movement:

- `bond_vault -> risk_reserve_or_relief_vault`.
- Phase 1 does not directly compensate users.

## 9. Security Layer Integration

Green Label V1 must not directly refund or slash a bond based only on local Green Label state. Refund and slash execution must be gated by System Security Layer V1.

Required checks before final value movement:

- The configured `security_governance_config` matches Green Label config.
- The `proposal_decision` PDA is valid for the expected `proposal_id`.
- The `proposal_decision` is approved for the expected Green Label action.
- The `execution_queue_item` PDA is valid for the same `proposal_id`.
- The queued action type matches the Green Label path:
  - `GreenLabelRefund`
  - `GreenLabelSlash`
- The queued `payload_hash` matches the canonical Green Label payload.
- The queued item is past timelock.
- The queued item is not cancelled.
- The queued item is not already executed.
- The Security Layer is not in a state that blocks execution.

Canonical payload hash requirements:

- Stable field order.
- Stable integer encoding.
- Include project id.
- Include dispute id when applicable.
- Include action type.
- Include bond vault.
- Include destination vault or recipient.
- Include amount rules or final amount.
- Include program version or domain separator, such as `green_label_v1`.

Example payload domains:

- `green_label_v1:refund`
- `green_label_v1:slash`

The DAO or Security Layer can produce decisions and queued actions, but it must not bypass timelock to directly transfer funds.

## 10. Funds Flow

Submit:

```text
project USDC ATA -> project Green Bond Vault
```

Refund:

```text
Green Bond Vault -> project owner USDC ATA
Green Bond Vault -> treasury vault for base 20%
```

Slash:

```text
Green Bond Vault -> risk reserve / relief vault / treasury-controlled vault
```

Funds flow principles:

- Bond funds must first enter an isolated Green Bond Vault.
- Bond funds must not enter the ordinary Treasury at submission time.
- Base bond treasury share is only moved during refund execution.
- Extra bond is fully refundable when no valid dispute, rug, or malicious behavior is confirmed.
- Full bond is slashable when rug or malicious behavior is confirmed.
- Phase 1 does not directly perform user compensation.
- Any future user compensation flow should be a later phase built on a separate decision and payout process.

## 11. Phase 1 Scope

Phase 1 includes:

- Application.
- Bond lock.
- Dispute creation.
- Evidence hash storage.
- Decision link.
- Queue/timelock integration.
- Refund/slash execution.

Phase 1 excludes:

- KYC/KYB.
- Automatic rug detection.
- Automatic insurance payout.
- Multiple simultaneous disputes.
- Complex risk scoring.
- Oracle integration.
- Cross-chain support / cross-chain projects.
- Automatic LP lock verification.

Phase 1 success means the protocol can prove a minimal Green Label control loop on Devnet:

```text
application -> bond lock -> observation/dispute -> decision -> queue -> timelock -> refund/slash
```

## 12. Devnet Test Plan

Minimum closed-loop test:

1. Initialize Green Label config.
2. Submit a 299 USDC Project Bond.
3. Verify project status is `PendingObservation`.
4. After observation, link a Security Layer `GreenLabelRefund` decision, queue the refund, and execute refund after timelock.
5. Submit a 1299 USDC Project Bond with 299 base bond and 1000 extra bond.
6. Open a dispute with a valid reason code and evidence hash.
7. Link a Security Layer decision to the dispute.
8. Queue a `GreenLabelSlash` action.
9. Execute slash after timelock.
10. Verify bond vault balance and treasury/risk vault balance.

Recommended assertions:

- Config stores the correct 299 USDC minimum.
- 299 USDC bond derives `Base` tier.
- 1299 USDC bond derives `Silver` tier.
- Bond vault receives full Project Bond on submit.
- Green Label cannot refund or slash without linked Security Layer decision.
- Green Label cannot execute before timelock.
- Pause blocks execution according to Security Layer rules.
- Cancelled queued action blocks execution.
- Refund path sends base 80% and extra 100% to project owner, and base 20% to treasury.
- Slash path sends slash amount to risk reserve, relief vault, or treasury-controlled vault.

## 13. Implementation Notes

Implementation guardrails:

- Do not change Program ID.
- Only append account, instruction, and enum surfaces.
- Do not break Treasury V2 account layout.
- Do not break Staking V1 account layout.
- Do not break Security Layer V1 account layout.
- All amounts use USDC 6 decimals.
- `payload_hash` must be stable and reproducible.
- Large evidence content should be off-chain, only hash stored on-chain.
- Green Bond Vaults should be isolated per project.
- PDA seed design should include explicit version strings, such as `green_label_project_v1`.
- Account sizing should reserve enough space for future status and scoring fields where practical.
- Any future frontend must keep the legal and messaging boundaries visible near Bond Tier and Project Bond displays.

Suggested PDA seed plan:

```text
green_label_config_v1
green_label_project_v1 + project_id
green_label_dispute_v1 + project_id + dispute_id
green_bond_vault_authority_v1 + project_id
```

Suggested next implementation sequence:

1. Add enum definitions.
2. Add account structs with versioned fields.
3. Add config initialization.
4. Add project submission and bond vault transfer.
5. Add dispute creation and evidence hash storage.
6. Add Security Layer decision linking.
7. Add refund and slash execution gates.
8. Add Devnet scripts that print `Transaction signature: <signature>` for successful transactions.
9. Add focused Devnet tests for refund, slash, pause-blocked, and cancel-blocked paths.
