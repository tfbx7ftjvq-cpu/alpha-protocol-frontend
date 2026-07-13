# DAO Contributor Governance Design

## Purpose

Phase 2E-3A adds the on-chain foundation for contributor governance. It gives Alpha Protocol a structured way to record who a contributor is, what role they serve, what milestones they claim, and what future builder payout request should be reviewed by DAO governance.

This phase is not the full DAO voting layer. It does not implement ALPHA voting, quorum, voting power, vote records, proposal finalization, or payout transfers.

## Why Contributor Registry Is Needed

Builders 20% revenue is protocol revenue routed into the Builders / Contributors Treasury bucket. Before the protocol can safely pay contributors from that bucket, the DAO needs auditable records for:

- contributor wallet
- contributor role
- contributor status
- activity timestamps
- completed milestone count
- approved payout count
- reputation score

`ContributorRegistryV1` is the base identity and status account for this workflow.

## Builders 20% Future Governance

The current USDC Revenue Router splits protocol revenue into:

- 50% Relief Pool
- 20% Buyback / Burn
- 20% Builders / Contributors
- 10% Staking Rewards

The 20% builders bucket is accounting-complete, but this phase does not transfer funds from that vault. Future governance phases should decide:

- who is eligible to receive builders payouts
- which milestones are approved
- whether a payout request is approved
- when an approved payout may be queued and executed

## Adding Contributors

`ContributorRegistryV1` is initialized with:

- wallet
- role
- status = `Active`
- joined timestamp
- last active timestamp
- zeroed milestone and payout counters
- zeroed reputation score

Current initialization is contributor-wallet controlled. Future DAO phases can add governed add-contributor flows using `ContributorProposalTypeV1::AddContributor`.

## Removing Contributors

`ContributorStatusV1` supports:

- `Active`
- `Suspended`
- `Removed`

The intended governance direction is:

- `Active -> Suspended`
- `Active -> Removed`
- `Suspended -> Active`
- `Suspended -> Removed`

`Removed` is terminal. Future DAO voting can use `ContributorProposalTypeV1::RemoveContributor` to govern removals.

## Approving Contributor Work

`ContributorMilestoneV1` records:

- contributor registry account
- title
- description
- evidence hash
- requested amount
- milestone status
- creation timestamp

Milestones start as `Pending`. Future governance can approve or reject them with `ContributorProposalTypeV1::ApproveMilestone`.

## Builder Payout Requests

`BuilderPayoutRequestV1` records:

- contributor registry account
- milestone account
- requested payout amount
- destination wallet
- payout status
- creation timestamp

Payout requests start as `Pending`. This phase does not transfer USDC and does not touch Treasury vaults. Future governance can approve payout requests with `ContributorProposalTypeV1::ApproveBuilderPayout`.

## Future Connection To ALPHA Voting Layer

Future Phase 2E-4 / 2E-5 should add:

- proposal creation
- ALPHA voting power
- quorum
- thresholds
- vote records
- voting period
- finalization

After finalization, the voting layer can create a Security Layer decision for contributor registry changes, milestone approvals, or builder payout approvals.

## Future Connection To Security Layer Execution

The Security Layer should remain the execution safety layer for sensitive DAO actions. Future contributor governance should connect to:

- `ProposalDecisionV1`
- `ExecutionQueueItemV1`
- timelock
- payload hash checks
- explicit action types

For builders payouts, the expected future path is:

1. ALPHA voting approves a builder payout.
2. Security Layer queues the payout execution.
3. Timelock expires.
4. A dedicated builders payout instruction transfers from the Builders Treasury vault.

That transfer instruction is intentionally not implemented in Phase 2E-3A.

## Current Phase Scope

Implemented in Phase 2E-3A:

- `ContributorRegistryV1`
- `ContributorMilestoneV1`
- `BuilderPayoutRequestV1`
- contributor role/status enums
- milestone/payout status enums
- contributor proposal type enum for future voting compatibility
- initialization instructions
- PDA validation
- signer ownership checks
- string length limits
- status transition helper rules

Not implemented in Phase 2E-3A:

- ALPHA holder voting
- quorum
- voting power
- vote records
- proposal finalization
- Treasury payout transfer
- builders vault withdrawal
- Security Layer payout execution
