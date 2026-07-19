# Governance Action Framework V1 Design

## 1. Why GovernanceActionTypeV1 Exists

Alpha Protocol now has:

- a Governance Layer that can create and finalize DAO proposals;
- a Security Layer that can queue and timelock execution;
- business modules such as Treasury, Green Label, Contributor Governance, Victim Relief, and Scam Registry.

Before this framework, Security `ActionType` had to describe both DAO meaning and Security execution. That works for early Devnet paths, but it does not scale cleanly as more modules join.

`GovernanceActionTypeV1` is the DAO-layer action language. It describes what the DAO intends to approve.

Security `ActionType` remains the execution-layer action language. It describes what the Security Layer can queue and execute today.

## 2. DAO Action vs Security Action

```text
GovernanceActionTypeV1
  DAO semantic intent
  examples: TreasuryApproveBuilderPayout, ScamRegistryAppealDecision

Security ActionType
  current executable Security Layer action
  examples: GreenLabelRefund, ContributorApproveMilestone
```

Every governance action maps to a Security action.

Some mappings are currently executable by existing modules, such as Green Label refund / slash and Contributor governance actions. Other mappings are placeholders for future modules, such as Victim Relief and Scam Registry. Placeholder mappings do not mean the business execution path is implemented.

Phase 2E-6B-2 activates the first Victim Relief business decision paths for `VictimReliefApproveCompensation` and `VictimReliefRejectClaim`. These paths still do not transfer USDC; approve creates a frozen `ReliefPayoutRequestV1`, and reject creates an immutable decision record.

Phase 2E-6B-3 appends `VictimReliefUpholdAppeal` and `VictimReliefOverturnAppeal`. Opening an appeal is not a governance action; it is a claimant business action. Appeal decisions are DAO actions that target `VictimReliefAppealV1`.

Phase 2E-6B-4B-4B appends `VictimReliefCancelPayout`. This is a terminal DAO + Security action for unpaid approved `ReliefPayoutRequestV1` accounts. It does not transfer USDC and does not reuse reject/uphold actions.

## 3. Mapping Model

The mapping is intentionally centralized:

```text
GovernanceActionTypeV1
-> map_governance_action_to_module()
-> ProtocolModuleIdV1

GovernanceActionTypeV1
-> map_governance_action_to_security_action()
-> Security ActionType
```

The mapping uses exhaustive `match` branches and has no default fallback.

The goal is to prevent callers from bypassing DAO semantics by directly choosing a low-level Security `ActionType`.

For initial Victim Relief approve/reject, the canonical governance target is the `VictimReliefCaseV1` account. The program recomputes the Victim Relief decision parameters hash from the frozen evidence snapshot, policy, case, Treasury config, relief vault, action, and proposal id. Callers do not choose the approved amount or recipient at execution time.

For appeal uphold/overturn, the canonical governance target is `VictimReliefAppealV1`. The program recomputes the appeal decision parameters hash from the case, appeal, original snapshot, original reject receipt, policy, Treasury config, relief vault, action, and proposal id.

For payout cancellation, the canonical governance target is `ReliefPayoutRequestV1`. The program recomputes the cancellation parameters hash from the payout request, original authorization source, frozen amount and recipient, cancellation proposal/decision/queue/action, expected statuses, and original authorization hash. Authority or guardian wallets cannot single-sign cancellation, and there is no generic cancel action.

## 4. ProtocolModuleIdV1

`ProtocolModuleIdV1` identifies the module targeted by a governance action:

- Treasury
- Green Label
- Victim Relief
- Scam Registry
- Contributor
- Protocol

Phase 2E-FINAL Stage 3 uses it with a module registry:

```text
ProtocolModuleRegistryV1
  security_governance_config
  module_id
  module_code
  program_id
  enabled
  schema_version
```

The V1 registry is an allow-list for modules inside the current Alpha Protocol program. It does not yet support external program registration or registry mutation.

## 5. Payload Hash Security Model

`GovernancePayloadV1` defines a canonical payload envelope:

```text
schema_version
action_type
module_id
target_program
target_account
parameters_hash
evidence_hash
created_at
```

`hash_governance_payload_v1()` hashes:

```text
alpha_governance_payload_v1 || serialized GovernancePayloadV1
```

The domain separator prevents governance payload hashes from being confused with other protocol hashes.

The helper is deterministic and uses fixed field order through Anchor serialization.

## 6. Future Module Expansion

The framework currently covers:

- Treasury update revenue split
- Treasury spending approval
- Builder payout approval
- Green Label certification approval / rejection / revocation
- Green Label refund / slash
- Victim Relief compensation approval / rejection / policy update
- Victim Relief appeal uphold / overturn
- Scam Registry publish / remove / appeal
- Contributor add / remove / update role / milestone / payout approval
- Protocol parameter update / upgrade / emergency action

Not every action is executable today. Future-facing actions are represented at both the DAO action layer and the Security action layer first, then connected to module accounts and instructions when their execution paths are implemented.

## 7. Adapter Compatibility

Current adapter flow:

```text
GovernanceProposalV1::Passed
-> UniversalGovernanceDecisionAdapterV1
-> ProposalDecisionV1
```

Future adapter flow:

```text
GovernanceActionTypeV1
-> UniversalGovernanceDecisionAdapterV1
-> Security ActionType
-> ExecutionQueueItemV1
```

Stage 2 now changes the adapter input model: the adapter consumes `GovernanceProposalActionV1`, not caller-controlled action data and not unverified proposal mirror fields.

Stage 3 adds `ProtocolModuleRegistryV1` to the same path:

```text
GovernanceActionTypeV1
-> ProtocolModuleIdV1
-> ProtocolModuleRegistryV1
-> UniversalGovernanceDecisionAdapterV1
-> Security ActionType
```

Strict proposal initialization, snapshot creation, and adapter creation all validate the same registry helper before trusting a module target.

Stage 5B adds Green Label module consumers for strict refund and strict forfeit:

- `GreenLabelRefundBond` routes escrow funds only back to the original payer.
- `GreenLabelSlashBond` routes recorded forfeited escrow funds through the USDC Treasury router as `RevenueType::GreenLabelForfeitedBond`.
- legacy public Green Label slash / forfeit entry points are disabled.

Stage 6B-1 adds Victim Relief foundation accounts:

- `VictimReliefConfigV1`
- immutable `VictimReliefPolicyV1`
- `VictimReliefClaimantStateV1`
- `VictimReliefCaseV1`

The Victim Relief governance actions remain future execution actions. Stage 6B-1 does not approve claims, reject claims, transfer relief USDC, create payout requests, or connect cases to DAO decisions.

Stage 6B-2 implements `VictimReliefApproveCompensation` and `VictimReliefRejectClaim`. Stage 6B-3 implements `VictimReliefUpholdAppeal` and `VictimReliefOverturnAppeal`. None of these Victim Relief governance paths transfers USDC; Stage 6B-4 must add payout execution.

## 8. Typed Proposal Action Binding

Phase 2E-FINAL Stage 2 adds `GovernanceProposalActionV1` as the immutable trusted source for new governance proposals.

`GovernanceProposalV1.action_type` is now a compatibility mirror field. The strict path uses explicit stable action codes from `governance_action_stable_code_v1()` and rejects unknown codes through `governance_action_from_stable_code_v1()`.

New governance proposals should be created with `initialize_governance_proposal_with_action_v1`, which derives the proposal category, action code, target mirrors, and canonical payload hash from a typed `GovernanceActionRequestV1`.

`create_governance_snapshot_v1` and `create_governance_decision_adapter_v1` both require the sidecar and the module registry. Legacy proposals without `GovernanceProposalActionV1` cannot enter the new voting or DAO-controlled execution path.

The target program is currently fixed to the Alpha Protocol Program ID through `ProtocolModuleRegistryV1`. Module Registry mutation and external program registration are intentionally deferred.

## 9. Non-Goals

The original action framework phase did not implement:

- Treasury transfer
- Builder payout
- Victim Relief claim approval / rejection / payout execution
- Scam Registry execution
- frontend
- deployment
- chain transactions
