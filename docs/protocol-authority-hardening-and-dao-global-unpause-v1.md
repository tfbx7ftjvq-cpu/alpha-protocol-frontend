# Protocol Authority Hardening and DAO Global Unpause V1

Phase 2E-6B-4B-4C-B2 adds protocol authority hardening for the Security global pause lifecycle.

This stage does not migrate the program upgrade authority, Treasury authority, Green Label authority, Staking authority, vault authority, or any frontend flow. It does not deploy or send transactions.

## Authority Control Sidecar

`ProtocolAuthorityControlV1` is a sidecar for `GovernanceConfigV1`.

PDA:

```text
protocol_authority_control_v1 + GovernanceConfigV1
```

It records:

- governance config
- authority mode
- bootstrap authority
- emergency guardian
- DAO activation proposal / decision / queue
- activation timestamp
- schema version

`GovernanceConfigV1` layout is unchanged.

## Authority Modes

`ProtocolAuthorityModeV1` uses stable codes:

- `Bootstrap = 1`
- `DaoControlled = 2`

The only valid transition is:

```text
Bootstrap -> DaoControlled
```

Rollback is not implemented.

## Bootstrap Mode

Bootstrap mode preserves the existing authority-controlled Security paths for Devnet and setup:

- create Security proposal decision
- queue Security execution
- authority global unpause

These paths now require the authority-control sidecar.

## DAO-Controlled Mode

After `execute_activate_protocol_dao_control_v1`, legacy authority paths fail closed:

- authority cannot create arbitrary Security decisions
- authority cannot queue arbitrary Security execution
- authority cannot globally unpause Security

The existing `GovernanceConfigV1.authority` value is not rewritten. The sidecar mode controls whether that authority can still use legacy Security paths.

## Activation Flow

DAO activation uses:

```text
GovernanceProposalV1 Passed
-> GovernanceProposalActionV1
-> Protocol module registry
-> UniversalGovernanceDecisionAdapterV1
-> ProposalDecisionV1 Approved
-> ExecutionQueueItemV1 Executed
-> execute_activate_protocol_dao_control_v1
-> ProtocolAuthorityControlV1.mode = DaoControlled
```

The activation wrapper is permissionless for execution, but it requires the current bootstrap authority to sign during activation. This prevents a malicious or stale chain from silently switching authority mode without the existing authority being present.

The canonical target is `ProtocolAuthorityControlV1`.

## Activation Parameters Hash

Domain:

```text
alpha_protocol_activate_dao_control_v1
```

The hash binds:

- authority control sidecar
- Security governance config
- expected mode `Bootstrap`
- next mode `DaoControlled`
- current authority
- emergency guardian
- governance proposal
- governance proposal action sidecar
- proposal decision
- execution queue item
- action `ProtocolActivateDaoControl`

The executor is not part of the canonical hash.

## DAO Global Unpause

Global Security unpause uses a dedicated recovery path:

```text
ProtocolUnpauseSecurity DAO proposal
-> adapter-created Security decision
-> queued Security action
-> execute_protocol_unpause_security_v1
-> GovernanceConfigV1.is_paused = false
-> ProtocolSecurityUnpauseExecutionRecordV1
```

This avoids a recovery deadlock: normal `execute_queued_action` remains blocked while `GovernanceConfigV1.is_paused == true`, but the dedicated recovery wrapper can execute only the `ProtocolUnpauseSecurity` action after timelock.

The canonical target is `GovernanceConfigV1`.

## Unpause Parameters Hash

Domain:

```text
alpha_protocol_unpause_security_v1
```

The hash binds:

- authority control sidecar
- Security governance config
- expected mode `DaoControlled`
- expected paused state `true`
- next paused state `false`
- action `ProtocolUnpauseSecurity`
- governance proposal
- governance proposal action sidecar
- proposal decision
- execution queue item
- proposal id

Executor and timestamp are not part of the canonical hash.

## Guardian Boundary

The emergency guardian remains risk-reducing only:

- can pause Security through the existing global pause path
- can cancel queued Security actions
- can pause Victim Relief module through its pause-only emergency path

The guardian cannot:

- globally unpause Security
- unpause Victim Relief
- activate DAO control
- execute payouts
- transfer Treasury funds
- change protocol authority mode

## Non-Goals

This stage does not implement:

- program upgrade authority migration
- Treasury / Green Label / Staking authority migration
- a universal DAO signer
- arbitrary execution
- vault authority changes
- frontend integration
- Devnet or Mainnet deployment

## Mainnet Status

This hardens the Security global unpause path, but Mainnet remains NO-GO until Devnet strict E2E, final authority review, upgrade authority policy, operational runbooks, audit, and Mainnet sanity are complete.
