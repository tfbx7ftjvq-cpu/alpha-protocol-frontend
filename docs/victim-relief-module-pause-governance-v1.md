# Victim Relief Module Pause Governance V1

Phase 2E-6B-4B-4C-B1 adds the Victim Relief module-level pause lifecycle.

This stage does not change `VictimReliefConfigV1` layout, does not transfer USDC, does not change Treasury accounting, does not migrate global Security unpause authority, and does not add frontend or Devnet execution.

## Purpose

Victim Relief has two pause layers:

- Security global pause: `GovernanceConfigV1.is_paused`.
- Victim Relief module pause: `VictimReliefConfigV1.paused`.

The module pause is domain-specific. It blocks Victim Relief risk-increasing actions without pausing all Security Layer execution.

## Governance Actions

Two DAO actions are appended to `GovernanceActionTypeV1`:

- `VictimReliefPause`
- `VictimReliefUnpause`

They are intentionally separate. There is no generic `set_pause(bool)` action because pause is risk-reducing and unpause is risk-increasing.

Both actions map to:

- `ProtocolModuleIdV1::VictimRelief`
- `GovernanceProposalTypeV1::VictimRelief`

The canonical governance target is always `VictimReliefConfigV1`.

## Security Types

Two Security proposal/action types are appended:

- `ProposalType::VictimReliefPause`
- `ProposalType::VictimReliefUnpause`
- `ActionType::VictimReliefPause`
- `ActionType::VictimReliefUnpause`

Pause only matches pause. Unpause only matches unpause. Cross-combinations fail.

## Pause Parameters

`VictimReliefPauseParametersV1` is hashed under:

```text
alpha_victim_relief_pause_parameters_v1
```

The hash binds:

- Victim Relief config
- Security governance config
- action type
- expected paused state
- next paused state
- governance proposal
- governance proposal action sidecar
- proposal id

Executor, guardian, and timestamp are not included in the canonical hash.

Pause fixes:

```text
expected_paused = false
next_paused = true
```

Unpause fixes:

```text
expected_paused = true
next_paused = false
```

## DAO Execution Receipt

`VictimReliefPauseExecutionRecordV1` is an immutable one-per-queue DAO receipt.

PDA:

```text
victim_relief_pause_rcpt_v1 + execution_queue_item
```

The receipt records the config, Security governance config, action, before/after paused states, governance proposal, proposal decision, execution queue, action sidecar, pause parameters hash, canonical governance payload hash, executor, timestamp, schema version, bump, and reserved bytes.

Multiple pause/unpause cycles are supported because each DAO action has its own queue and receipt PDA.

Guardian emergency pause does not create this DAO receipt. It is audited by transaction history and the `VictimReliefGuardianPausedV1` event.

## Guardian Emergency Pause

`guardian_pause_victim_relief_v1` allows only `GovernanceConfigV1.emergency_guardian` to set `VictimReliefConfigV1.paused = true`.

The guardian:

- can pause the Victim Relief module
- cannot unpause it
- cannot change authority, Treasury, policy, vault, or payout state
- cannot transfer USDC
- can pause the module even when Security global pause is already active

Repeating guardian pause when the module is already paused fails.

There is no guardian unpause instruction.

## DAO Pause

`execute_pause_victim_relief_v1` is permissionless after the DAO and Security chain has already completed:

```text
GovernanceProposalV1::Passed
-> GovernanceProposalActionV1::VictimReliefPause
-> UniversalGovernanceDecisionAdapterV1
-> ProposalDecisionV1::Approved
-> ExecutionQueueItemV1::Executed
-> VictimReliefConfigV1.paused = true
-> VictimReliefPauseExecutionRecordV1
```

Because pause is risk-reducing, the wrapper can run after a valid executed queue even if `GovernanceConfigV1.is_paused == true`.

## DAO Unpause

`execute_unpause_victim_relief_v1` uses the same DAO + Security chain with `VictimReliefUnpause`.

Unpause is risk-increasing and requires:

- `VictimReliefConfigV1.paused == true`
- `GovernanceConfigV1.is_paused == false`
- valid Victim Relief module registry
- valid adapter, Security decision, and executed queue

Authority and guardian signers cannot replace the DAO chain.

## Pause Behavior Matrix

When Victim Relief module pause is active, the module blocks:

- new case submission
- evidence updates
- evidence freeze
- DAO approve / reject case execution
- opening appeals
- DAO uphold / overturn appeal execution
- original approve payout
- appeal overturn payout

When Victim Relief module pause is active, the module still allows strict payout cancellation after its cancellation queue has already executed. Cancellation is risk-reducing, transfers no USDC, and only closes an unpaid approved request.

When Security global pause is active:

- `execute_queued_action` is blocked
- payout wrappers are blocked
- DAO module unpause is blocked
- guardian module pause is still allowed
- already-executed DAO module pause is allowed
- already-executed payout cancellation remains allowed

Claimant cancellation and permissionless evidence expiry keep their existing code semantics; this stage does not rewrite them.

## Authority Boundary

`VictimReliefConfigV1.authority` cannot directly pause or unpause through this stage.

Security authority cannot bypass the DAO chain for module unpause.

The emergency guardian is pause-only.

Vault authority PDA does not participate in pause lifecycle.

Phase 2E-6B-4B-4C-B2 adds `ProtocolAuthorityControlV1` and the DAO-controlled `ProtocolUnpauseSecurity` recovery path for Security global unpause. Program upgrade authority, Treasury / Green Label / Staking authorities, and Devnet strict E2E still remain outside this module-pause stage.

## Compatibility

This stage does not modify layouts for:

- `VictimReliefConfigV1`
- `GovernanceConfigV1`
- `ReliefPayoutRequestV1`
- payout receipt
- cancellation receipt
- case / appeal accounts
- Treasury accounts
- module registry
- adapter
- proposal decision
- execution queue

Existing enums are append-only. No generated IDL is committed by this stage.

## Non-Goals

This stage does not implement:

- full protocol authority migration beyond Security global unpause
- program upgrade authority migration
- Treasury / Green Label / Staking pause
- generic pause setter
- authority direct pause / unpause
- guardian unpause
- token transfer
- payout or cancellation rewrite
- appeal expiry
- recipient migration
- reservation / FIFO
- frontend
- Devnet or Mainnet deployment

Local tests are not Devnet or Mainnet verification. Mainnet and token launch remain NO-GO.
