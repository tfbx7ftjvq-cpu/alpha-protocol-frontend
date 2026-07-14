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

## 4. ProtocolModuleIdV1

`ProtocolModuleIdV1` identifies the module targeted by a governance action:

- Treasury
- Green Label
- Victim Relief
- Scam Registry
- Contributor
- Protocol

Future phases can use it with a module registry:

```text
ProtocolModuleRegistryV1
  module_id
  program_id
  authority
  enabled
```

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

This phase does not change adapter execution logic. It only adds the shared action framework that future adapter phases should consume.

## 8. Non-Goals

This phase does not implement:

- Treasury transfer
- Builder payout
- Green Label execution changes
- Victim Relief execution
- Scam Registry execution
- frontend
- deployment
- chain transactions
