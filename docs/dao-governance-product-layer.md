# DAO Governance Product Layer

Date: 2026-07-08

## 1. Current DAO Status

Alpha Protocol currently has the Security Layer V1 execution layer completed on Devnet. The full ALPHA token voting layer is still pending.

Current completed scope:

- Security Layer V1 governance config.
- Proposal decision accounts.
- Execution queue accounts.
- Timelock-gated execution.
- Cancel path.
- Pause / unpause path.
- Green Label refund / slash linked to Security Layer decisions.

Current Phase 2A scope:

- Read-only DAO Governance Dashboard.
- No voting button.
- No queue / execute / cancel / pause write button.
- No claim of a fully decentralized DAO.

## 2. Security Layer V1 Capabilities

Security Layer V1 provides the execution guard for sensitive protocol actions:

- Proposal decision recording.
- Queue execution.
- Timelock enforcement.
- Execute queued action.
- Cancel queued action.
- Pause and unpause controls.
- Action type checks.
- Payload hash checks.

This layer is the execution safety foundation. It is not yet the full community voting product.

## 3. DAO Read-only Dashboard Purpose

The DAO Governance Dashboard makes the execution layer visible to external users and reviewers.

It should show:

- Governance config authority and emergency guardian.
- Minimum execution delay.
- Proposal count.
- Pause status.
- Verified Devnet proposal / queue paths.
- Raw proposal and queue account fields for auditability.
- Devnet and read-only warnings.

It must not expose write actions until the product and governance model are ready.

## 4. DAO Governance Scope

Future DAO governance should cover:

- Green Label refund / slash.
- Relief pool payout policy.
- Treasury parameter changes.
- Protocol fee split changes.
- Risk exposure / blacklist evidence policy.
- Staking reward policy.
- Emergency pause review.
- Contributor / builders pool spending.

## 5. Why DAO Is Core To Alpha Protocol

Alpha Protocol handles treasury routing, risk labeling, dispute outcomes, refund/slash actions, emergency controls, and long-term incentive policy. These are governance-sensitive surfaces.

DAO governance is core because it provides:

- Public accountability for sensitive execution paths.
- A clear process for parameter changes.
- Timelock and review windows before irreversible actions.
- Separation between emergency protection and ordinary governance.
- A path toward community-controlled treasury and protocol policy.

## 6. Security Layer Cannot Be Bypassed

Refund, slash, treasury, and emergency actions must not bypass the Security Layer.

Reasons:

- Refund/slash decisions affect project funds and protocol credibility.
- Treasury parameter changes affect protocol revenue routing.
- Emergency controls can change user-facing availability.
- Payload hash and action type checks protect against wrong execution.
- Timelock gives reviewers time to detect mistakes or malicious actions.

Any shortcut around Security Layer governance weakens the protocol trust model.

## 7. DAO Oversight of Treasury and Token Revenue

DAO / Security Layer governance should eventually supervise the Token / Revenue / Treasury product loop.

This includes:

- Treasury V2 parameters and revenue split policy.
- Builders / contributors pool spending.
- Relief pool payout and risk response policy.
- Staking reward policy.
- Green Label refund / slash execution.
- Emergency pause review and timelock-protected recovery.

The Token / Revenue dashboard is read-only product presentation. It must not imply guaranteed yield, automatic insurance payouts, or discretionary spending without governance.

## 8. Phase 2A Roadmap

1. Read-only governance dashboard.
2. Proposal product model.
3. Off-chain / community voting MVP or ALPHA voting design.
4. On-chain voting layer.
5. DAO-controlled treasury and governance authority.

## 9. Messaging Boundary

Until the voting layer is complete:

- Do not market Alpha Protocol as a fully decentralized DAO.
- Describe the current system as a Devnet-verified execution / security layer.
- Make clear that read-only dashboards expose state but do not execute governance.
- Make clear that full ALPHA voting power, quorum, threshold, and delegation are future phases.
