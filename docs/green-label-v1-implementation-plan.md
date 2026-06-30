# Alpha Protocol Green Label V1 Implementation Plan

Last updated: 2026-06-27

This document is an implementation plan only. It does not define final contract code and does not change the current deployed program, Program ID, Anchor configuration, keypairs, frontend business logic, or Devnet deployment state.

Source design reference:

- `docs/green-label-v1-design.md`

Completed foundation:

- Treasury V2 Devnet milestone
- Alpha Guardian Staking V1 Devnet milestone
- System Security Layer V1 Devnet milestone
- Green Label V1 design milestone

## 1. Goal

Green Label V1 should add a project-level risk commitment flow on top of the existing Alpha Protocol foundation.

Core goals:

- Project owner submits a Project Bond / Risk Bond.
- Minimum Project Bond is 299 USDC.
- Additional Bond is voluntary and uncapped.
- Bond funds enter an isolated Green Bond Vault, not the normal Treasury at submission time.
- After the observation period, a project can receive a refund.
- If a dispute confirms rug or malicious behavior, the bond can enter a slash path.
- Refund and slash must go through System Security Layer V1:
  - decision
  - queue
  - timelock
  - execute

Green Label V1 must remain legally and product-wise conservative:

- Not insurance.
- Not investment advice.
- Not a credit rating.
- Not a guarantee of project safety.
- Bond Tier only reflects economic commitment.
- No fixed compensation promise.
- No direct victim payout in Phase 1.

## 2. Files To Add / Modify

Future implementation is expected to append Green Label V1 surfaces to the existing Anchor program. The exact file split can follow the current repository structure at implementation time.

Expected files to add:

- `server/programs/my_first_solana_program/src/instructions/green_label_v1.rs`

Expected files to modify:

- `server/programs/my_first_solana_program/src/state.rs`
- `server/programs/my_first_solana_program/src/error.rs`
- `server/programs/my_first_solana_program/src/constants.rs`
- `server/programs/my_first_solana_program/src/instructions.rs`
- `server/programs/my_first_solana_program/src/lib.rs`

Implementation constraints:

- Only append accounts, enums, helper functions, and instructions.
- Do not break Treasury V2 account layout.
- Do not break Staking V1 account layout.
- Do not break Security Layer V1 account layout.
- Do not modify Program ID.
- Do not modify `Anchor.toml`.
- Do not modify `target/deploy`.
- Do not modify keypair files.
- Do not deploy unless explicitly requested later.

Recommended module boundary:

- `green_label_v1.rs` contains instruction account contexts and handlers.
- `state.rs` contains new account structs and enums.
- `constants.rs` contains Green Label constants and PDA seed constants.
- `error.rs` contains new Green Label error variants.
- `instructions.rs` re-exports the new Green Label instruction module.
- `lib.rs` appends instruction entrypoints without changing existing entrypoints.

## 3. Constants

Constants should be appended to the existing constants module. All USDC amounts use 6 decimals.

Recommended constants:

| Constant | Value | Purpose |
| --- | ---: | --- |
| `MIN_GREEN_LABEL_BASE_BOND_USDC` | `299_000_000` | 299 USDC minimum base bond |
| `BASE_BOND_REFUND_BPS` | `8000` | 80% base bond refund to project owner |
| `BASE_BOND_TREASURY_BPS` | `2000` | 20% base bond treasury share |
| `MAX_BPS` | `10000` | Basis point denominator |
| `DEFAULT_OBSERVATION_PERIOD_SECONDS` | 30 days | Default observation period |
| `DEFAULT_DISPUTE_WINDOW_SECONDS` | TBD policy value | Dispute opening window |
| `DEFAULT_RESPONSE_WINDOW_SECONDS` | TBD policy value | Project response window |

Recommended PDA seed constants:

| Seed | Value |
| --- | --- |
| `GREEN_LABEL_CONFIG_V1_SEED` | `green_label_config_v1` |
| `GREEN_LABEL_PROJECT_V1_SEED` | `green_label_project_v1` |
| `GREEN_LABEL_DISPUTE_V1_SEED` | `green_label_dispute_v1` |
| `GREEN_BOND_VAULT_AUTHORITY_V1_SEED` | `green_bond_vault_authority_v1` |

Validation notes:

- `BASE_BOND_REFUND_BPS + BASE_BOND_TREASURY_BPS` must equal `MAX_BPS`.
- Phase 1 should treat the base bond as exactly 299 USDC.
- Amounts above the base bond are extra bond.
- Extra bond is uncapped, subject only to `u64` token amount limits and practical token transfer constraints.

## 4. Accounts To Add

The implementation should add three new account types:

- `GreenLabelConfigV1`
- `GreenLabelProjectV1`
- `GreenLabelDisputeV1`

Every new account should include:

- Explicit versioned PDA seeds.
- A bump field.
- Padding / reserved bytes for future extension.
- Conservative space allocation.
- No changes to existing Treasury V2, Staking V1, or Security Layer V1 account layouts.

### GreenLabelConfigV1

Purpose:

- Stores global Green Label configuration.
- Links Green Label to USDC, Treasury V2 vaults, and Security Layer V1 governance config.

Expected PDA seeds:

```text
green_label_config_v1
```

Implementation fields should include:

- `authority`
- `usdc_mint`
- `min_base_bond_usdc`
- `base_refund_bps`
- `base_treasury_bps`
- `observation_period_seconds`
- `dispute_window_seconds`
- `response_window_seconds`
- `project_count`
- `treasury_usdc_state_v2`
- `base_bond_treasury_vault`
- `relief_usdc_vault`
- `builders_usdc_vault`
- `staking_usdc_vault`
- `buyback_usdc_vault`
- `green_bond_risk_reserve_vault`
- `security_governance_config`
- `is_paused`
- `bump`
- `reserved`

Space estimate:

- Core fields are roughly 270 to 320 bytes before Anchor discriminator and padding.
- Recommended initial allocation: at least 384 bytes plus 8-byte discriminator.
- Prefer a larger reserve if the existing program style commonly pads future-facing accounts.

Implementation notes:

- `base_bond_treasury_vault` receives the base bond 20% treasury share on refund.
- `green_bond_risk_reserve_vault` or a configured relief vault receives slash funds.
- `security_governance_config` must point to System Security Layer V1 config.

### GreenLabelProjectV1

Purpose:

- Stores one Green Label project application and final terminal action links.
- Tracks bond amount, bond vault, status, observation window, disputes, and refund/slash timestamps.

Expected PDA seeds:

```text
green_label_project_v1 + project_id
```

Implementation fields should include:

- `project_id`
- `project_owner`
- `project_name_hash`
- `project_url_hash`
- `token_mint`
- `project_treasury_wallet`
- `base_bond_amount`
- `extra_bond_amount`
- `total_bond_amount`
- `bond_vault`
- `bond_vault_authority`
- `bond_tier`
- `status`
- `submitted_at`
- `observation_start_ts`
- `observation_end_ts`
- `dispute_count`
- `active_dispute`
- `terminal_action_proposal_id`
- `terminal_action_proposal_decision`
- `terminal_action_execution_queue_item`
- `terminal_action_payload_hash`
- `approved_at`
- `refunded_at`
- `slashed_at`
- `risk_score_snapshot`
- `bump`
- `reserved`

Space estimate:

- Core fields are roughly 390 to 460 bytes before Anchor discriminator and padding, depending on enum and option encoding.
- Recommended initial allocation: at least 512 bytes plus 8-byte discriminator.
- If using Anchor `Option<Pubkey>` and `Option<[u8; 32]>`, confirm serialized sizes in tests.

Implementation notes:

- `terminal_action_*` fields are required so observation refund can still bind to a Security Layer decision, execution queue item, and payload hash even when there is no active dispute.
- Slash must not use the no-dispute path.
- `GreenLabelSlash` requires a linked dispute.
- `active_dispute` enforces the Phase 1 rule of no multiple simultaneous disputes.

### GreenLabelDisputeV1

Purpose:

- Stores one dispute against a Green Label project.
- Stores evidence hash and optional Security Layer linkage for dispute-driven refund or slash.

Expected PDA seeds:

```text
green_label_dispute_v1 + project_id + dispute_id
```

Implementation fields should include:

- `project_id`
- `dispute_id`
- `project`
- `disputer`
- `reason_code`
- `evidence_hash`
- `status`
- `opened_at`
- `evidence_end_ts`
- `response_end_ts`
- `resolved_at`
- `proposal_id`
- `proposal_decision`
- `execution_queue_item`
- `bump`
- `reserved`

Space estimate:

- Core fields are roughly 190 to 240 bytes before Anchor discriminator and padding.
- Recommended initial allocation: at least 288 bytes plus 8-byte discriminator.

Implementation notes:

- Large evidence content must remain off-chain.
- Only a stable evidence hash is stored on-chain.
- `proposal_decision` and `execution_queue_item` must be checked against System Security Layer V1 before final value movement.

## 5. Enums To Add

Enums should be append-only and version-aware.

### GreenLabelStatus

Expected states:

- `PendingBondDeposit`
- `PendingObservation`
- `ObservationPassed`
- `Disputed`
- `PendingSecurityDecision`
- `QueuedRefund`
- `QueuedSlash`
- `Refunded`
- `Slashed`
- `Cancelled`

Transition boundaries:

- `PendingBondDeposit -> PendingObservation` only after Green Bond Vault creation and successful Project Bond / Risk Bond transfer.
- `PendingBondDeposit -> Cancelled` is allowed before bond deposit completes.
- `PendingBondDeposit` must not transition directly to `ActiveGreenLabel`, `Disputed`, `QueuedRefund`, or `QueuedSlash`.
- `PendingObservation -> ObservationPassed` after observation conditions are met.
- `PendingObservation -> Disputed` when a valid dispute opens.
- `ObservationPassed -> QueuedRefund` only after Security Layer decision and queue link.
- `Disputed -> PendingSecurityDecision` when dispute is ready for decision.
- `PendingSecurityDecision -> QueuedRefund` for a refund decision.
- `PendingSecurityDecision -> QueuedSlash` for a slash decision.
- `QueuedRefund -> Refunded` only after queue, timelock, action type, and payload hash checks.
- `QueuedSlash -> Slashed` only after queue, timelock, action type, payload hash, and linked dispute checks.
- Terminal states `Refunded` and `Slashed` must not transition again.
- Terminal state `Cancelled` must not transition again.

### BondTier

Expected variants:

- `Base`
- `Bronze`
- `Silver`
- `Gold`
- `Platinum`
- `Custom`

Transition boundaries:

- Bond Tier should be derived from `total_bond_amount`.
- Bond Tier should not be manually upgraded without a bond amount change.
- Phase 1 can derive tier at submission time and store the snapshot.

### RugReasonCode

Expected variants:

- `LiquidityRemoved`
- `DeveloperDump`
- `WebsiteOrCommunityAbandoned`
- `MintOrFreezeAuthorityAbuse`
- `TreasuryMisuse`
- `FalseDisclosure`
- `MaliciousContractUpgrade`
- `Other`

Transition boundaries:

- Reason code is set at dispute creation.
- Reason code should not change after dispute creation in Phase 1.
- If more evidence is needed, update off-chain evidence content and refer to a new deterministic evidence hash only through a future explicit instruction.

### DisputeStatus

Expected states:

- `Open`
- `EvidencePeriod`
- `AwaitingProjectResponse`
- `ReadyForDecision`
- `QueuedRefund`
- `QueuedSlash`
- `ResolvedRefund`
- `ResolvedSlash`
- `Rejected`
- `Cancelled`

Transition boundaries:

- `Open -> EvidencePeriod` if implementation separates initial open from evidence period.
- `EvidencePeriod -> AwaitingProjectResponse` when evidence window closes.
- `AwaitingProjectResponse -> ReadyForDecision` when response window closes or governance marks ready.
- `ReadyForDecision -> QueuedRefund` after linked Security Layer refund decision.
- `ReadyForDecision -> QueuedSlash` after linked Security Layer slash decision.
- `QueuedRefund -> ResolvedRefund` only after refund execution.
- `QueuedSlash -> ResolvedSlash` only after slash execution.
- `Rejected` and `Cancelled` are terminal for that dispute.

## 6. Instructions To Implement

Implement instructions in the order below. Each instruction should have focused validation and clear state transition boundaries.

### 1. initialize_green_label_config

Accounts:

- `green_label_config`
- `authority`
- `usdc_mint`
- `treasury_usdc_state_v2`
- `base_bond_treasury_vault`
- `relief_usdc_vault`
- `builders_usdc_vault`
- `staking_usdc_vault`
- `buyback_usdc_vault`
- `green_bond_risk_reserve_vault`
- `security_governance_config`
- `system_program`

Args:

- `min_base_bond_usdc`
- `base_refund_bps`
- `base_treasury_bps`
- `observation_period_seconds`
- `dispute_window_seconds`
- `response_window_seconds`

Validation:

- Authority signs.
- Minimum base bond is at least 299 USDC.
- Refund and treasury BPS sum to `MAX_BPS`.
- USDC mint matches expected Devnet or configured mint.
- Treasury and Security Layer accounts match known configured addresses.

State transition:

- Creates `GreenLabelConfigV1`.
- Sets `is_paused = false`.
- Sets `project_count = 0`.

Token transfer:

- No token transfer.

Failure cases:

- Invalid BPS config.
- Invalid minimum bond.
- Missing signer.
- Invalid treasury or Security Layer account.

### 2. submit_green_label_application

Phase 1D should split submit into two implementation steps:

- Phase 1D-1: create project account only, with `status = PendingBondDeposit`.
- Phase 1D-2: create Green Bond Vault and transfer USDC, then move `status -> PendingObservation`.

Phase 1D-1 must not start the observation period. `observation_start_ts` and `observation_end_ts` should be set only after Phase 1D-2 successfully locks the Project Bond / Risk Bond.

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

Validation:

- Config is not paused.
- Project owner signs.
- `base_bond_amount` equals the configured base amount in Phase 1.
- `extra_bond_amount` is allowed to be zero or higher.
- `total_bond_amount = base_bond_amount + extra_bond_amount`.
- `total_bond_amount` is at least 299 USDC.
- Bond Tier is derived from total bond amount.
- Project PDA and bond vault authority PDA match expected seeds.

State transition:

- Creates `GreenLabelProjectV1`.
- Phase 1D-1 sets status to `PendingBondDeposit`.
- Phase 1D-1 does not set observation timestamps.
- Phase 1D-2 sets status to `PendingObservation`.
- Phase 1D-2 sets `observation_start_ts` and `observation_end_ts`.
- Increments config `project_count`.

Token transfer:

- Phase 1D-1: no token transfer.
- Phase 1D-2: `project_owner_usdc_ata -> bond_vault`.

Failure cases:

- Bond below minimum.
- Invalid owner ATA.
- Invalid USDC mint.
- Invalid PDA.
- Insufficient project owner USDC.
- Arithmetic overflow.
- Attempting refund, slash, dispute, or observation activation while still `PendingBondDeposit`.

### 3. open_green_label_dispute

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

Validation:

- Config is not paused.
- Disputer signs.
- Project is in a disputable state.
- Project has no active dispute.
- Evidence hash is not zero.
- Dispute is opened within the allowed observation or dispute window.

State transition:

- Project status moves to `Disputed`.
- Project `active_dispute` is set.
- Project `dispute_count` increments.
- Dispute status moves to `EvidencePeriod` or `Open`.

Token transfer:

- No token transfer in Phase 1.

Failure cases:

- Active dispute already exists.
- Evidence hash is zero.
- Invalid project status.
- Dispute window closed.
- Missing signer.

### 4. mark_dispute_ready_for_decision

Accounts:

- `green_label_config`
- `green_label_project`
- `green_label_dispute`
- `authority_or_dispute_admin`

Args:

- `project_id`
- `dispute_id`

Validation:

- Config is not paused.
- Caller is authorized by governance policy.
- Dispute belongs to project.
- Dispute is active.
- Evidence and response windows are complete, unless future governance policy allows an explicit emergency ready path.

State transition:

- Project status moves to `PendingSecurityDecision`.
- Dispute status moves to `ReadyForDecision`.

Token transfer:

- No token transfer.

Failure cases:

- Unauthorized caller.
- Invalid dispute status.
- Dispute does not belong to project.
- Evidence or response period not complete.

### 5. link_security_decision

Accounts:

- `green_label_config`
- `green_label_project`
- optional `green_label_dispute` for dispute-driven actions
- `security_governance_config`
- `proposal_decision`
- `execution_queue_item`
- `authority_or_security_adapter`

Args:

- `project_id`
- optional `dispute_id`
- `proposal_id`
- `expected_action_type`
- `expected_payload_hash`

Validation:

- Config is not paused.
- Security governance config matches Green Label config.
- Proposal decision belongs to the configured Security Layer.
- Execution queue item belongs to the same proposal id.
- Security decision is approved for the expected action.
- Action type is `GreenLabelRefund` or `GreenLabelSlash`.
- Payload hash matches the canonical Green Label payload.
- Queue item is not cancelled and not already executed.
- For no-active-dispute refund, project is `ObservationPassed` and has no active dispute.
- For slash, a linked dispute is required and dispute is `ReadyForDecision`.

State transition:

- Stores Security Layer terminal action fields on the project.
- Stores Security Layer fields on the dispute when a dispute is supplied.
- Moves project to `QueuedRefund` or `QueuedSlash`.
- Moves dispute to `QueuedRefund` or `QueuedSlash` when applicable.

Token transfer:

- No token transfer.

Failure cases:

- Missing linked dispute for slash.
- Payload hash mismatch.
- Action type mismatch.
- Queue cancelled.
- Queue already executed.
- Decision not approved.
- Security Layer account mismatch.

### 6. execute_green_label_refund

Accounts:

- `green_label_config`
- `green_label_project`
- optional `green_label_dispute` when refund resolves a dispute
- `security_governance_config`
- `proposal_decision`
- `execution_queue_item`
- `bond_vault`
- `bond_vault_authority`
- `project_owner_usdc_ata`
- `base_bond_treasury_vault`
- `token_program`

Args:

- `project_id`
- optional `dispute_id`
- `payload_hash`

Validation:

- Config is not paused.
- Project status is `QueuedRefund`.
- If dispute is supplied, dispute status is `QueuedRefund`.
- If dispute is not supplied, project has no active dispute and observation path is valid.
- Project terminal action fields match supplied Security Layer accounts.
- Security Layer action type is `GreenLabelRefund`.
- Payload hash matches terminal action payload hash and queue payload hash.
- Queue status is executable.
- Queue is not cancelled.
- Queue is not already executed.
- Timelock is satisfied.
- No paused exception is allowed.
- Bond vault balance covers refund and treasury amounts.

State transition:

- Project status moves to `Refunded`.
- Sets `refunded_at`.
- Dispute status moves to `ResolvedRefund` when supplied.
- Clears active dispute when present.

Token transfer:

- Base bond 80%: `bond_vault -> project_owner_usdc_ata`.
- Base bond 20%: `bond_vault -> base_bond_treasury_vault`.
- Extra bond 100%: `bond_vault -> project_owner_usdc_ata`.

Failure cases:

- Missing Security Layer queue link.
- Payload hash mismatch.
- Action type mismatch.
- Timelock not satisfied.
- Security Layer paused or otherwise blocking execution.
- Already refunded.
- Bond vault balance too low.

### 7. execute_green_label_slash

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

Validation:

- Config is not paused.
- Project status is `QueuedSlash`.
- Dispute status is `QueuedSlash`.
- Slash has a linked dispute.
- Project and dispute terminal action fields match supplied Security Layer accounts.
- Security Layer action type is `GreenLabelSlash`.
- Payload hash matches terminal action payload hash and queue payload hash.
- Queue status is executable.
- Queue is not cancelled.
- Queue is not already executed.
- Timelock is satisfied.
- No paused exception is allowed.
- Slash amount does not exceed bond vault balance.

State transition:

- Project status moves to `Slashed`.
- Dispute status moves to `ResolvedSlash`.
- Sets `slashed_at` and dispute `resolved_at`.
- Clears active dispute.

Token transfer:

- `bond_vault -> risk_reserve_or_relief_vault`.

Failure cases:

- Slash without linked dispute.
- Missing Security Layer queue link.
- Payload hash mismatch.
- Action type mismatch.
- Timelock not satisfied.
- Security Layer paused or otherwise blocking execution.
- Already slashed.
- Slash amount exceeds bond vault balance.

## 7. Security Layer Integration

Green Label V1 must not directly refund or slash based only on local Green Label state. The implementation must gate final value movement through System Security Layer V1.

Required rules:

- Green Label cannot directly refund or slash.
- `execute_green_label_refund` must check the Security Layer queue item.
- `execute_green_label_slash` must check the Security Layer queue item.
- Both execution instructions must check `action_type`.
- Both execution instructions must check `payload_hash`.
- Both execution instructions must check queue status and executed state.
- Both execution instructions must check that timelock is satisfied.
- No paused exception is allowed.
- `GreenLabelSlash` must have a linked dispute.
- Observation refund, even without active dispute, must bind through project-level terminal action fields:
  - Security decision
  - execution queue item
  - payload hash

Security Layer checks should verify:

- `security_governance_config` matches the configured Security Layer account.
- `proposal_decision` PDA matches expected proposal id.
- `execution_queue_item` PDA matches expected proposal id.
- Proposal decision is approved.
- Queue item is for `GreenLabelRefund` or `GreenLabelSlash`.
- Queue item payload hash matches the Green Label canonical payload.
- Queue item is not cancelled.
- Queue item is not already executed.
- Timelock has elapsed.
- Security Layer state does not block execution.

The Green Label program should treat the Security Layer decision and queue item as the source of final authorization. Local project or dispute status is necessary but not sufficient.

## 8. Funds Flow

Submit:

```text
project USDC ATA -> Green Bond Vault
```

Refund:

```text
Green Bond Vault -> project owner USDC ATA
Green Bond Vault -> base_bond_treasury_vault
```

Slash:

```text
Green Bond Vault -> risk reserve / relief vault / treasury-controlled vault
```

Funds flow rules:

- Bond funds must enter the isolated project Green Bond Vault first.
- Submission must not route bond funds into the ordinary Treasury.
- Base 299 USDC refund uses 80/20 split.
- Extra bond refund is 100% to the project owner when no valid dispute or rug is confirmed.
- Confirmed rug or malicious behavior can slash the full bond, including base and extra bond.
- Phase 1 does not directly do user payout or victim compensation.

## 9. Payload Hash Design

`payload_hash` must be stable, deterministic, and reproducible across frontend, scripts, tests, and contract expectations.

General requirements:

- Use fixed binary encoding or deterministic serialization.
- Avoid mutable text directly in the hash.
- Normalize all optional fields.
- Include a domain separator.
- Include action type.
- Include project id.
- Include relevant accounts and amounts.
- Use raw integer token amounts in USDC 6 decimals.

Refund payload should include:

- action type: `GreenLabelRefund`
- domain separator: `green_label_v1:refund`
- project id
- project account
- project owner
- bond vault
- project owner USDC ATA
- base bond refund amount
- base bond treasury amount
- extra bond refund amount
- base bond treasury vault

Slash payload should include:

- action type: `GreenLabelSlash`
- domain separator: `green_label_v1:slash`
- project id
- dispute id
- dispute account
- project account
- bond vault
- slash amount
- destination vault

Implementation notes:

- Frontend, scripts, and tests must use the exact same field order.
- Contract-side validation should use the same canonical layout.
- Hashing should avoid locale-sensitive or whitespace-sensitive text.
- Project name and URL should already be represented by fixed hashes, not raw strings.
- Any future payload version change should use a new domain separator or explicit version field.

## 10. Testing Plan

Rust unit tests should cover pure helper logic and state transition validation before Devnet scripts are introduced.

Required tests:

- Bond tier calculation.
- Base / extra bond split.
- 299U refund calculation.
- 1299U refund calculation.
- Slash full bond calculation.
- Invalid bond below 299U.
- Invalid BPS config.
- Status transition validation.
- Slash without linked dispute should fail.
- Refund without Security Layer linked queue should fail.
- Payload hash mismatch should fail.
- Action type mismatch should fail.
- Timelock not satisfied should fail.
- Paused Security Layer should still block execution at Security Layer level.
- Cannot double refund.
- Cannot double slash.

Additional recommended tests:

- Base 20% treasury share rounds correctly in 6-decimal USDC.
- Extra bond is fully refundable.
- Full bond slash includes base and extra bond.
- Active dispute prevents a no-dispute observation refund.
- Multiple simultaneous disputes are rejected in Phase 1.
- Cancelled queue blocks refund.
- Cancelled queue blocks slash.
- Already executed queue blocks repeat execution.
- Invalid destination vault is rejected.
- Wrong project id in payload hash is rejected.

Testing order:

1. Pure helper tests for BPS, amount, and tier logic.
2. Account initialization tests.
3. State transition tests.
4. Security Layer integration validation tests.
5. Token transfer accounting tests.

## 11. Devnet Script Plan

This section is a script plan only. Do not add scripts until implementation is complete and explicitly requested.

Future scripts:

- `initialize-green-label-config-v1.ts`
- `submit-green-label-application-v1.ts`
- `open-green-label-dispute-v1.ts`
- `mark-dispute-ready-for-decision-v1.ts`
- `link-green-label-security-decision-v1.ts`
- `execute-green-label-refund-v1.ts`
- `execute-green-label-slash-v1.ts`
- `get-green-label-project-v1.ts`

Script requirements when implemented:

- Every successful transaction prints `Transaction signature: <signature>`.
- Scripts must not hard-code a different Program ID.
- Scripts must derive PDAs deterministically.
- Scripts must print relevant PDA addresses before sending.
- Scripts must print payload hash inputs and final payload hash.
- Scripts must support dry-run style account derivation where practical.
- Scripts must avoid deployment or key sync behavior.

## 12. Devnet Milestone Plan

Path A: 299U refund path

- Initialize config.
- Submit 299U bond.
- Create and link Security decision.
- Queue refund.
- Wait timelock.
- Execute refund.
- Verify 80/20 split.

Path B: 1299U refund path

- Submit 1299U bond.
- Confirm base 299 + extra 1000.
- Create and link Security refund decision.
- Queue refund.
- Wait timelock.
- Execute refund.
- Verify base 80/20 and extra 100% refund.

Path C: dispute slash path

- Submit bond.
- Open dispute.
- Mark dispute ready for decision.
- Link slash decision.
- Queue slash.
- Wait timelock.
- Execute slash.
- Verify bond vault drained to risk / relief vault.

Path D: failure paths

- Refund without queue fails.
- Slash without dispute fails.
- Payload mismatch fails.
- Action type mismatch fails.
- Cancelled queue fails.
- Timelock not satisfied fails.
- Already executed queue fails.
- Double refund fails.
- Double slash fails.

Milestone completion criteria:

- All expected successful transactions produce signatures.
- All expected failure paths fail with intended errors.
- Bond vault balances match expected accounting.
- Treasury / risk reserve balances match expected accounting.
- No deployment occurs unless explicitly requested.

## 13. Non-Goals

Phase 1 does not include:

- KYC/KYB.
- Automatic rug detection.
- Automatic insurance payout.
- Direct victim payout.
- Multiple simultaneous disputes.
- Complex risk score calculation.
- Oracle integration.
- Cross-chain support.
- Automatic LP verification.
- Frontend full production UI.

Phase 1 should stay focused on proving the minimal on-chain control loop:

```text
application -> bond lock -> observation/dispute -> Security Layer decision -> queue -> timelock -> refund/slash
```

## 14. Implementation Order

Recommended sequence:

1. Constants / enums / pure helper functions.
2. Account structs.
3. Initialize config.
4. Submit Phase 1D-1: create project account only, `status = PendingBondDeposit`.
5. Submit Phase 1D-2: create Green Bond Vault, transfer USDC, `status -> PendingObservation`.
6. Dispute creation.
7. Decision link.
8. Refund execution.
9. Slash execution.
10. Tests.
11. Build.
12. Devnet scripts.
13. Devnet verification.

Suggested checkpoint criteria:

- After step 1: pure helper tests pass.
- After step 2: account space tests are reviewed.
- After step 4: project account creation tests pass and observation timestamps remain unset.
- After step 5: 299U and 1299U bond deposit tests pass and observation timestamps are set.
- After step 7: payload hash and Security Layer linkage tests pass.
- After step 9: refund and slash accounting tests pass.
- After step 11: IDL is generated by build, but only synced to frontend after the implementation milestone explicitly reaches that stage.
- After step 13: Devnet status document can be updated with signatures and verified addresses.

## 15. Safety Checklist

Before implementation:

- Program ID unchanged.
- `Anchor.toml` unchanged.
- No keypair committed.
- No `target/deploy` committed.
- No account layout break for existing accounts.
- Green Label account additions are append-only.
- Treasury V2 tests still pass.
- Staking V1 tests still pass.
- Security Layer V1 tests still pass.

Before build:

- `cargo test` passed.
- Anchor account space estimates reviewed.
- Payload hash fixtures reviewed.
- Error cases covered.

Before IDL sync:

- `anchor build` passed.
- IDL changed only because implementation changed.
- Frontend IDL synced only after successful build / deploy stage.

Before Devnet:

- No deployment unless explicitly requested.
- No chain scripts unless explicitly requested.
- No `anchor keys sync`.
- Program ID still unchanged.
- Devnet scripts print `Transaction signature: <signature>`.
- Security Layer queue / timelock / cancel / pause protections are verified.

Before marking Green Label V1 complete:

- 299U refund path verified.
- 1299U refund path verified.
- Dispute slash path verified.
- Failure paths verified.
- Documentation updated with signatures and account addresses.
