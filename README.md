# Alpha Protocol

Alpha Protocol 是 Solana 上的链上国库、绿标认证、散户救济与未来保险/理赔协议原型。

## 项目权威目标与上线清单

`docs/alpha-protocol-canonical-product-charter-and-launch-checklist-v1.md`
是项目范围、收入模型、DAO 权限边界、反闪电治理和上线优先级的唯一权威入口。
若阶段文档、旧 README 状态或自动生成的工作摘要与该文件冲突，以该文件为准；
任何核心模型变更必须由项目所有者明确确认并同步更新该文件。

## 当前仓库结构

- `project/` 是 Vite + React + TypeScript 前端。
- `server/` 是 Anchor / Solana Program 工程。
- `public/auditSnapshot.json` 是当前前端读取的静态审计快照。

## 当前已实现功能

- Phantom 钱包连接。
- Devnet SOL 余额读取。
- 前端国库、链上法庭、质押赔付、DAO 治理页面。
- 静态 `auditSnapshot.json` 读取。
- Anchor 合约工程骨架。

## 当前尚未实现功能

The following historic prototype inventory is not the current release-status
source of truth. It is superseded by the Devnet, off-chain Staging, and Mainnet
boundaries documented below and in the Phase 2E-6F runbook.

- 真实链上 `TreasuryState`。
- `deposit` 指令。
- 50/20/20/10 链上分账。
- 绿标认证链上账户。
- 保险保单账户。
- 理赔账户。
- DAO 链上投票。
- 后端 daemon / worker。

## 本地运行前端

```bash
cd project
npm install
npm run dev
```

## Anchor 合约构建

```bash
cd server
anchor build
```

## 当前阶段声明

当前项目仍处于 MVP / prototype 阶段，不能用于真实资金托管或主网生产环境。

## Deployment boundary (Phase 2E-6F)

The repository contains verified Devnet-oriented Solana program work and a
separate off-chain Supabase Staging operations boundary. The Pages frontend can
publish static release evidence at `/release.json`, but a Pages Production
deployment is not a Mainnet deployment, custody system, authority change, or
funds movement. Mainnet remains unlaunched. See
`docs/phase-2e-6f-release-monitoring-recovery-and-launch-operations-v1.md` for
the release, monitoring, recovery, and human-only rollback procedure.

## Public Pilot boundary (Phase 2E-6G)

The Public Pilot candidate is an off-chain, human-controlled release readiness
state, not a Mainnet launch. It keeps real role grants, gate changes, funds,
and Solana execution outside automated tooling. The local Go/No-Go validator
and required evidence schemas are documented in
`docs/phase-2e-6g-public-pilot-launch-readiness-v1.md`.
