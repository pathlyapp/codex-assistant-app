# M6 定向修复契约

本文定义 `WP-602` 第一工作包。目标是让诊断页根据当前错误和真实系统状态只显示一个
安全、可解释的修复动作，而不是固定命令列表。

关联需求：`DIAG-001`、`DIAG-007`、`DIAG-010`、`LIFE-006`、`REL-001`。

## 1. 用户行为

诊断页先读取 `SystemStatusV1`，再调用 Rust `get_repair_plan`。页面显示以下三种状态：

| 状态 | 用户看到的结果 |
| --- | --- |
| `action_available` | 一个针对当前问题的主修复按钮 |
| `manual_required` | 不提供有风险的自动修改，引导导出诊断或返回配置页 |
| `not_needed` | 明确说明当前无需修复 |

页面不再常驻“恢复配置”等固定修复命令。复制摘要、导出诊断包仍是独立支持动作。

## 2. 核心契约

`RepairPlanV1`：

```json
{
  "schemaVersion": 1,
  "state": "action_available|manual_required|not_needed",
  "title": "string",
  "detail": "string",
  "errorCode": "ROUTER_CONNECTION_REFUSED",
  "action": {
    "id": "revalidate_router",
    "label": "重新验证 Router",
    "description": "string",
    "requiresConfirmation": false
  }
}
```

`RepairResultV1`：

```json
{
  "schemaVersion": 1,
  "actionId": "revalidate_router",
  "success": true,
  "changed": true,
  "summary": "string",
  "before": {
    "overall": "blocked",
    "appState": "installed",
    "routerState": "unreachable",
    "configState": "verified",
    "appearanceState": "official"
  },
  "after": {
    "overall": "ready",
    "appState": "installed",
    "routerState": "responses_verified",
    "configState": "verified",
    "appearanceState": "official"
  }
}
```

前端只能使用 `action.id` 和结构化字段选择交互，禁止解析中文错误字符串。

## 3. 第一批动作

| action ID | 触发条件 | 修改边界 | 确认 |
| --- | --- | --- | --- |
| `recheck_official_app` | 官方应用状态为 `needs_repair` | 只重新读取包身份、签名、版本、注册和程序文件证据 | 否 |
| `revalidate_router` | 已有配置，Router 不可达、仅 `/models` 有效或错误码属于 Router | 使用已保存认证检查 `/models` 和 `/responses`；成功后只更新验证证据 | 否 |
| `restore_configuration` | 配置错误、当前配置未验证且存在完整事务快照 | 使用现有事务恢复能力，执行前再次建立可回退快照 | 是 |
| `clear_appearance_session` | 外观错误且当前不是官方外观 | 原子写入 `official` 外观状态；不修改 ChatGPT 安装文件或 Codex 配置 | 是 |

`ROLLBACK_FAILED` 永远不提供自动修改。诊断证据必须保留，用户先导出诊断包。

## 4. 幂等与竞态

- `run_repair` 执行前重新生成方案。
- 请求动作与最新方案不一致时返回 `REPAIR_PLAN_STALE`，不执行修改。
- 当前已无可执行动作时返回 `REPAIR_NOT_AVAILABLE`。
- Router 成功后状态变为 `responses_verified`，旧按钮不能再次执行。
- 官方应用复检是只读操作。
- 外观清理在 `official` 状态下不再提供。
- 配置恢复后必须重新读取 `SystemStatusV1`；若仍未验证，不把操作显示为系统就绪。
- 每个结果同时返回 `before`、`after` 和 `changed`，便于 E2E 与现场演示核对。

## 5. 安全边界

- 修复请求不接收路径、Key、命令或脚本。
- `errorCode` 只接受 64 字节以内的大写字母、数字和下划线。
- Router 认证只从现有安全存储读取，不返回前端。
- 主题清理不停止或启动 ChatGPT。
- 官方应用异常只重新检测；当前版本不静默重注册、不覆盖安装、不卸载。
- 助手运行文件和快捷方式修复需要安装器级签名、权限和 E2E，未在本工作包伪装为已完成。

## 6. 稳定错误

| code | 含义 |
| --- | --- |
| `REPAIR_NOT_AVAILABLE` | 当前状态已无该自动修复动作 |
| `REPAIR_PLAN_STALE` | 执行前状态变化，原方案失效 |
| `REPAIR_PLAN_FAILED` | 无法读取足够状态生成方案 |
| `REPAIR_EXECUTION_FAILED` | 执行未完成且未被更具体错误码分类 |

Router、配置、应用和外观的具体失败继续保留原稳定错误码。

## 7. 验证

本工作包至少需要：

1. Rust 方案映射、错误码校验和高风险阻断单测。
2. macOS 宿主格式、Clippy、全量测试。
3. Windows ARM64/x64 目标测试和 NSIS 构建。
4. Windows UI E2E 验证 `/responses` 失败后只显示 `revalidate_router`。
5. Windows UI E2E 验证旧固定诊断恢复按钮不存在。
6. Router 恢复后执行修复，前后状态从 `models_verified` 变为
   `responses_verified`，且 Codex 配置文件 SHA256 不变。

## 8. 后续门禁

- Windows 官方包损坏注册的安全重注册或官方重装。
- 助手文件完整性清单和签名修复。
- Windows/macOS 快捷方式或 Launch Services 修复。
- macOS 正式签名样本上的定向修复 E2E。
- 配置恢复的权限、磁盘满和快照损坏故障矩阵。

## 9. 当前证据

2026-07-28：

- macOS 宿主格式、严格 Clippy、前端语法、版本一致性和 Rust 全量测试通过：
  62 通过、0 失败、1 个本地 Ollama live test 忽略。
- Windows ARM64/x64 目标测试结果相同，两个 NSIS 均构建成功。
- Windows ARM64 受控 `/responses` 404 验证：
  - 配置 SHA256 不变。
  - 旧 Responses 证据撤销。
  - 状态变为 `models_verified`。
  - 诊断页只显示 `revalidate_router`。
- Router 恢复后从页面执行修复：
  - 收据为 `models_verified -> responses_verified`。
  - `changed=true`，`config.toml` SHA256 不变。
  - 再次生成方案为 `not_needed`，历史错误码不会重复产生动作。
- ARM64 原生和 x64 兼容层均通过静默安装不自启、首次响应、单实例和完整 UI E2E；
  VM 最终恢复 ARM64 原生候选。
- 候选安装包：
  - ARM64：5,609,117 字节，SHA256
    `16bc39ee86f2122dc7f1acf58bd39e674cd73cf9a1cbcbd1b552044c8bad6bc4`。
  - x64：5,984,362 字节，SHA256
    `dee005ed4a1f9d042d1248826a2d7db60ef98ca05e9f730c6d69fb52527e9912`。

真实 x64 硬件和签名 macOS 候选仍是独立门禁，兼容层证据不替代真实 x64。
