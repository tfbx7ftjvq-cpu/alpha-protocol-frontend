# Governance Lock and Voting Power Design

## Purpose

Phase 2E-4C-1 adds the first real governance power primitive for Alpha Protocol:

- a governance lock configuration account
- an ALPHA governance vault
- user governance positions
- ALPHA lock / unlock instructions
- deterministic voting power calculation

This phase does not implement DAO voting, proposal vote casting, snapshot execution, proposal finalization, Security Layer connection, frontend UI, or Mainnet deployment.

## Naming Note

The Security Layer already uses `GovernanceConfigV1` for execution queue and timelock configuration. To avoid breaking that verified account layout, the governance lock configuration account is named `GovernanceLockConfigV1`.

The public instruction is still named `initialize_governance_config_v1` because it initializes the Governance V1 lock configuration.

## Governance Lock Design

`GovernanceLockConfigV1` stores global lock parameters:

- `authority`
- `alpha_mint`
- `governance_vault`
- `min_lock_amount`
- `min_lock_duration_seconds`
- `max_lock_duration_seconds`
- `max_time_multiplier_bps`
- `created_at`
- `bump`

Default parameters:

- minimum lock duration: 30 days
- maximum lock duration: 365 days
- maximum time multiplier: 20,000 bps, or 2x
- minimum lock amount: 1 raw ALPHA unit

`GovernancePositionV1` records one wallet's governance lock:

- `owner`
- `alpha_mint`
- `vault`
- `locked_amount`
- `lock_start_time`
- `lock_end_time`
- `holding_multiplier_bps`
- `voting_power`
- `status`
- `last_updated_at`
- `bump`

## PDA Seeds

Governance lock config:

```text
governance_lock_config_v1
```

Governance vault:

```text
governance_vault_v1 + governance_config.key()
```

Governance position:

```text
governance_position_v1 + owner.key()
```

The governance vault token account is owned by the governance lock config PDA. Unlock transfers therefore require the governance config PDA signer seeds and cannot be initiated by an arbitrary wallet.

## Why Governance Lock Is Separate From Staking

Governance lock and staking rewards are intentionally separate systems.

Staking V1 is a rewards accounting mechanism funded by protocol revenue rules. Governance lock is a voting-power primitive for future DAO decisions. Keeping them separate avoids coupling reward eligibility to governance control and makes future risk reviews cleaner.

Future phases may decide whether staking positions can be mirrored or migrated into governance positions, but this phase only supports explicit ALPHA governance locking.

## Voting Power Formula

Voting power uses integer-only arithmetic:

```text
sqrt(locked_amount) * holding_multiplier_bps / 10_000
```

No floating point math is used.

Time multiplier tiers:

| Lock Duration | Multiplier |
| --- | --- |
| 30 days | 10,000 bps |
| 90 days | 11,000 bps |
| 180 days | 15,000 bps |
| 365 days | 20,000 bps |

Examples:

- `locked_amount = 10,000`, 30 days -> `sqrt(10,000) * 1.0 = 100`
- `locked_amount = 10,000`, 365 days -> `sqrt(10,000) * 2.0 = 200`

The square-root term dampens whale dominance. The time multiplier rewards longer lock commitment up to the 2x cap.

## Lifecycle

1. Initialize governance config and governance vault.
2. Initialize a user's governance position.
3. User locks ALPHA into the governance vault.
4. Position records locked amount, lock start, lock end, multiplier, and voting power.
5. After `lock_end_time`, user may unlock ALPHA back to their own token account.
6. Position is marked `Closed` after full unlock.

This phase supports full unlock only. Partial unlocks, relocking after close, vote snapshots, and proposal voting are future phases.

## Security Restrictions

- ALPHA mint must match `GovernanceLockConfigV1.alpha_mint`.
- User source token account owner must match the signer.
- Governance vault must match the configured vault and mint.
- Governance vault owner must be the governance config PDA.
- Lock amount must be at least `min_lock_amount`.
- Lock duration must be between 30 and 365 days.
- Unlock is only allowed after `lock_end_time`.
- Unlock transfers can only be signed by the governance config PDA.
- Governance position updates are restricted to the owner-derived PDA.
- All amount and timestamp arithmetic uses checked operations.

## Future Voting Layer Connection

Future phases should connect this lock foundation to:

1. `GovernanceSnapshotV1` voting power freeze.
2. `VoteRecordV1` cast-vote records.
3. quorum and threshold rules.
4. proposal finalization.
5. conversion of passed governance proposals into Security Layer `ProposalDecisionV1`.
6. Security Layer queue / timelock / execute flow.

## Not Implemented In This Phase

- DAO vote casting
- proposal voting
- snapshot execution
- vote finalization
- Security Layer connection
- reputation multiplier
- frontend
- Mainnet deployment
