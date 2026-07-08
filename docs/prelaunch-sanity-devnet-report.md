# Alpha Protocol Devnet Prelaunch Sanity Report

Date: 2026-07-08

This report records the current Alpha Protocol Devnet read-only prelaunch sanity baseline. It represents Devnet health only and does not mean Mainnet is approved for launch.

## 1. Report Metadata

- Report name: Alpha Protocol Devnet Prelaunch Sanity Report
- Cluster: devnet
- RPC URL: `https://api.devnet.solana.com`
- Expected mode: `devnet-test`
- Report source: latest read-only `devnet:prelaunch:sanity` run
- Result: PASS with WARN only
- FAIL: none
- MANUAL_REVIEW: none

## 2. Program / IDL Check

- Program ID: `HrLBQxUD3XHkB3KABjHXTiBHuAe6jVP2UPqiwmpmH8EY`
- IDL address matches Program ID.
- Program account exists.
- Program account is executable.
- Program owner is BPFLoaderUpgradeable.
- ProgramData address parsed successfully.

Green Label key instructions exist in the IDL:

- `initialize_green_label_config`
- `submit_green_label_application`
- `initialize_green_bond_vault`
- `lock_green_label_bond`
- `open_green_label_dispute`
- `mark_dispute_ready_for_decision`
- `link_green_label_security_decision`
- `execute_green_label_refund`
- `execute_green_label_slash`
- `update_green_label_windows`
- `update_green_label_min_base_bond`

## 3. Green Label Config Check

- Green Label config PDA: `7hNAeoqZxqvp38giY9gZwfR5ai3ttYTrse63QNYrRBWS`
- Owner matches Program ID.
- Anchor discriminator matches.
- Decoded successfully.
- Authority: `CqSs2yq6Jo3gYwXBq7fGRqohcxXS7HFJNYypykZTEGa8`
- USDC mint: `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`
- `min_base_bond_usdc`: 1.000000 USDC
- `base_refund_bps`: 8000
- `base_treasury_bps`: 2000
- `observation_period_seconds`: 30
- `dispute_window_seconds`: 30
- `response_window_seconds`: 30
- `project_count`: 3
- `treasury_usdc_state_v2`: `5e7eyC5ViwH9GBn73cY6so7J6KpRCX6XsbxozHabk2fE`
- `base_bond_treasury_vault`: `5XXaoWVSxVzyupzSs5NGXx6c8JMPD26QE7oZNmnUBAt8`
- `relief_or_risk_vault`: `GQSK91eQ5zwzGfYchunVqrPtxe3WLokxY88JbzTVcuRM`
- `vault_authority_v2`: `FovfcDDZzc8ff2Z2uxNZ1fTjpuVoLkRTPUPTLvXL8TEK`
- `security_governance_config`: `5np4fcpSP8eHVLD6dsgLHf7H11VLaGcYgdadxidt9ro3`
- `is_paused`: false

Current Devnet parameters `1 USDC / 30s / 30s / 30s` are test parameters only. Mainnet must restore `299 USDC / 30 days / 7 days / 3 days`.

## 4. Security Governance Check

- Security governance config PDA: `5np4fcpSP8eHVLD6dsgLHf7H11VLaGcYgdadxidt9ro3`
- Account exists.
- Owner matches Program ID.
- Discriminator matches.
- Decoded successfully.
- Authority: `CqSs2yq6Jo3gYwXBq7fGRqohcxXS7HFJNYypykZTEGa8`
- `min_execution_delay_seconds`: 60
- `proposal_count`: 5
- `emergency_guardian`: `CqSs2yq6Jo3gYwXBq7fGRqohcxXS7HFJNYypykZTEGa8`
- `is_paused`: false
- `bump`: 255

Devnet timelock delay is nonzero, so the Devnet sanity check passed. Devnet authority and emergency guardian are test-environment assumptions. Before Mainnet, these controls must be migrated to multisig / governance / Security Layer timelock or otherwise manually confirmed.

## 5. Treasury V2 Check

- Treasury V2 decoded successfully.
- Treasury USDC state: `5e7eyC5ViwH9GBn73cY6so7J6KpRCX6XsbxozHabk2fE`
- `base_bond_treasury_vault` / builders vault: `5XXaoWVSxVzyupzSs5NGXx6c8JMPD26QE7oZNmnUBAt8`
- `relief_or_risk_vault` / relief vault: `GQSK91eQ5zwzGfYchunVqrPtxe3WLokxY88JbzTVcuRM`
- Treasury vault mints match USDC mint.
- Devnet Treasury balances are test balances.
- `base_bond_treasury_vault` balance: 8.2 USDC
- `relief_or_risk_vault` balance: 21 USDC

Before Mainnet, all four Treasury vaults, mints, owners, and authorities must be confirmed. Devnet Treasury authority and mint must not be treated as Mainnet configuration.

## 6. Staking V1 Check

- Staking V1 decoded successfully.
- Devnet staking pool: `91PjLExu9FCLY6KQuvuisEhTEciQyWXJGW9fMKUEHW35`
- Staking vault checks passed.
- Rewards vault checks passed.
- Devnet staking balances are test balances and may be zero.
- Devnet Staking authority and mints are test-environment assumptions.

Mainnet mode must explicitly pass `STAKING_POOL`. Before Mainnet, ALPHA mint, USDC rewards mint, staking vault, rewards vault, and authority must be confirmed.

## 7. Summary

PASS:

- Cluster and RPC endpoint do not conflict.
- IDL exists and address matches Program ID.
- Program account exists and is executable.
- GreenLabelConfigV1 decoded successfully.
- Devnet test parameters are allowed.
- Security GovernanceConfigV1 decoded successfully.
- Security timelock delay is nonzero.
- Treasury V2 decoded successfully.
- Treasury vault mints match USDC mint.
- Staking V1 decoded successfully.
- Staking vault checks passed.

WARN:

- Devnet test params are not Mainnet production params.
- Devnet Treasury vault balances are test balances.
- Devnet Treasury V2 authority and mint are test-environment assumptions.
- Devnet Security governance authority and emergency guardian may be test wallets.
- Devnet Green Label authority may be a test wallet.
- Devnet Staking vault balances are test balances and may be zero.
- Devnet Staking V1 authority and mints are test-environment assumptions.

FAIL:

- none

MANUAL_REVIEW:

- none

## 8. Mainnet Implications

This report cannot be used as Mainnet go-live approval. Before Mainnet, `mainnet:prelaunch:sanity` must be run and must not report any unresolved FAIL items.

Before Mainnet, Green Label parameters must be restored to:

- `min_base_bond_usdc = 299 USDC`
- `observation_period_seconds = 2592000`
- `dispute_window_seconds = 604800`
- `response_window_seconds = 259200`

Before Mainnet, the following must be confirmed:

- Program ID / IDL
- GreenLabelConfig authority
- Security governance authority
- Emergency guardian permissions
- Treasury vaults
- Staking pool
- ALPHA mint
- USDC mint
- Vault authorities

If Mainnet sanity reports any FAIL, Mainnet launch is not allowed.
