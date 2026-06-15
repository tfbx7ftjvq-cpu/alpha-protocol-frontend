# Alpha Protocol

Alpha Protocol 是 Solana 上的链上国库、绿标认证、散户救济与未来保险/理赔协议原型。

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
