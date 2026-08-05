# Phase 2E-6B-4O：救助审核与公开进度闭环 v1

## 结论

本阶段把救助申请从“私有提交”推进到可审计的人工审核闭环，同时严格保持以下边界：

- `approved` 只是资格审核结果，不是付款授权；
- 审核 RPC 不创建国库执行意图、不创建付款回执、不发送 Solana 交易；
- 申请人身份、钱包、申请金额、私有证据和审核备注不会进入公开进度；
- 只有申请人明确同意时，批准路径才能生成一条脱敏公开进度；
- 拒绝路径不生成公开进度；
- 审核与公开记录不可覆盖修改，只能由后续独立流程追加新记录；
- Staging E2E 数据只能由精确匹配的 service-role 清理 RPC 删除。

因此，本阶段实现的是“申请—审核—公开进度—审计证据”的运营闭环，不是自动赔付系统。

## 数据边界

### 私有申请

`relief_applications` 保存：

- 申请人 Auth 用户标识；
- 事件说明；
- 请求 USDC 金额；
- 私有证据 URL；
- 已验证的 Solana 钱包地址；
- 审核状态、审核人、审核时间和私有审核备注；
- 是否同意发布脱敏公开进度。

匿名用户不能读取该表。申请人只能读取自己的申请，审核角色只能通过 RLS 读取待审核内容。

### 公开进度

`relief_public_updates` 只保存：

- 不含身份信息的公开案例引用；
- 公开标题和摘要；
- `approved` 等进度结果；
- 公开依据；
- 发布时间。

该表不包含申请人、钱包、申请金额、证据 URL、审核备注或付款回执字段。

### 私有审计事件

`operations_relief_workflow_events` 为每次终态审核追加一条事件：

- `application_approved`；
- `application_rejected`。

事件记录审核角色、唯一审计引用和结构化边界证明：

```json
{
  "payment_intent_created": false,
  "payment_receipt_created": false,
  "approval_is_payment": false
}
```

事件仅审核角色可读，普通浏览器用户和匿名用户不可读。

## 审核 RPC

`review_relief_application_v1` 是唯一允许的终态审核入口。

它要求：

1. 调用者已认证；
2. 调用者角色为 `relief_reviewer`、`operator` 或 `governance_admin`；
3. 调用者不是申请人本人；
4. 申请尚未进入终态；
5. 申请不存在付款回执或国库执行意图；
6. 审核备注和审计引用满足长度与控制字符约束；
7. 批准时的公开字段要么全部省略，要么完整提供；
8. 生成公开进度必须已有申请人同意；
9. 拒绝时所有公开字段必须为空。

RPC 在一个数据库事务中完成状态迁移、可选公开进度和审计事件，任何一步失败都会整体回滚。

## 权限收紧

本阶段撤销 authenticated 对以下操作的直接权限：

- 更新或删除 `relief_applications`；
- 插入、更新或删除 `relief_public_updates`。

这意味着即使具备审核 RLS 身份，也不能绕过审核 RPC 直接改写状态或发布内容。

## Staging E2E

4O 扩展现有一次性 Turnstile + Solana Web3 Staging E2E，验证：

- 创建两条真实钱包认证的私有申请；
- 匿名读取私有申请失败；
- 无角色审核失败；
- 申请人自审失败；
- 直接表更新失败；
- 同意公开的批准申请生成一条脱敏公开进度；
- 未同意公开的拒绝申请不生成公开进度；
- 终态重放失败；
- 匿名读取审计事件失败；
- 审核角色能读取两条精确事件；
- 事件明确证明没有付款意图、付款回执或自动付款；
- 公开进度不可改写；
- service-role-only 的只读付款状态 RPC 返回两个精确申请、零国库执行意图和零付款回执；
- 精确清理 1 条公开进度、2 条事件和 2 条私有申请。

加入 4O 后，整套 Staging E2E 预期为 61 条断言；完整成功路径清理 20 行测试数据和 4 个临时 Auth 用户。

## 文件

- `supabase/migrations/202608060001_operations_relief_moderation_closure.sql`
- `supabase/migrations/202608060002_operations_relief_staging_e2e_cleanup.sql`
- `supabase/migrations/202608060003_operations_relief_staging_e2e_payment_guard.sql`
- `project/scripts/operations-staging/e2e.ts`
- `project/src/features/operations/domain.ts`
- `project/src/features/operations/repository.ts`
- `project/src/hooks/useOperationsStaff.ts`
- `project/src/components/OperationsDashboard.tsx`
- `project/tests/operations-domain.test.ts`
- `project/tests/operations-e2e-cleanup.test.ts`
- `project/tests/operations-schema.test.ts`
- `project/tests/operations-staging-e2e-auth.test.ts`
- `supabase/tests/database/operations_schema.test.sql`
- `supabase/tests/database/operations_cleanup_privileges.test.sql`

## 部署状态

初始 4O 变更集部署后：

- GitHub 与 Cloudflare Production 已部署 `f278c1f26c7bb80976efeb1f144bd0aa1ddb3f0f`；
- Supabase Staging 已应用 `202608060001` 和 `202608060002`；
- 首次远程 4O E2E 在付款缺失断言处发现 `service_role` 没有私有国库表的直接 `SELECT` 权限；精确清理随后成功，没有遗留测试数据；
- `202608060003` 以 service-role-only、精确 fixture 绑定的只读 RPC 修复该断言，不授予整表读取权限；
- 应用 `202608060003` 后必须使用新的单次 Turnstile token 重跑完整 Staging E2E。

任何审核结果都不能被描述为已赔付、保证赔付、自动赔付或链上支付。
