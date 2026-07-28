# M6 卸载与数据边界契约

本文定义 `WP-603` 第一工作包，关联 `LIFE-001`、`LIFE-002`、`LIFE-003`、
`REL-001`、`REL-002` 和 `REL-044`。

目标不是增加一个更强的“一键清理”，而是保证用户始终知道正在操作哪一个对象。

## 1. 四个独立对象

| 对象 | 默认行为 | 可选动作 | 明确不做 |
| --- | --- | --- | --- |
| Codex 助手程序 | 由系统卸载；保留配置和数据 | 启动 NSIS 卸载器或在 Finder 定位应用 | 不卸载 ChatGPT |
| 助手管理的 Codex 配置 | 保留 | 事务化移除助手管理块 | 不删除其他 TOML 字段和 profile |
| 助手运行数据 | 保留 | 删除状态、备份、主题和保存的 Key | 受管配置仍依赖数据时禁止删除 |
| ChatGPT 官方应用 | 保留 | 打开系统应用管理，由系统再次确认 | 助手不再直接调用 `Remove-AppxPackage` |

旧“一键还原”同时停止并卸载 ChatGPT、清理配置和删除数据，违反
`LIFE-002/LIFE-003`，已经从前端和核心命令中移除。

## 2. 用户交互

诊断页“应用与数据”区域固定按以下顺序显示：

1. `Codex 助手`：卸载助手程序；说明默认保留其余对象。
2. `Codex 配置`：仅在检测到助手受管配置时启用“恢复原配置”。
3. `助手数据`：仅在数据存在且受管配置已移除时启用“删除数据”。
4. `ChatGPT 官方应用`：打开操作系统应用管理，不在助手内直接卸载。

三个修改性动作都有独立确认。删除数据不能绕过“先恢复原配置”的依赖约束。

## 3. 核心契约

`LifecycleStatusV1`：

```json
{
  "schemaVersion": 1,
  "assistantUninstallMode": "nsis|finder|unavailable|unsupported",
  "assistantUninstallAvailable": true,
  "managedConfigPresent": true,
  "assistantDataPresent": true,
  "officialAppInstalled": true,
  "officialAppTrusted": true,
  "dataRemovalBlocked": true,
  "defaultPreservesConfig": true,
  "defaultPreservesData": true,
  "defaultPreservesOfficialApp": true
}
```

`LifecycleActionRequest`：

```json
{
  "actionId": "restore_pre_assistant_config",
  "confirmation": "RESTORE_MANAGED_CONFIGURATION"
}
```

动作只允许：

- `uninstall_assistant`
- `restore_pre_assistant_config`
- `delete_assistant_data`
- `open_official_app_management`

请求不接收路径、命令、脚本或任意删除范围。修改性动作要求匹配固定确认 token。

`LifecycleActionResultV1`：

```json
{
  "schemaVersion": 1,
  "actionId": "restore_pre_assistant_config",
  "status": "completed|not_needed|handoff_started",
  "changed": true,
  "appExitRequested": false,
  "summary": "string",
  "before": {
    "managedConfigPresent": true,
    "assistantDataPresent": true,
    "officialAppInstalled": true
  },
  "after": {
    "managedConfigPresent": false,
    "assistantDataPresent": true,
    "officialAppInstalled": true
  }
}
```

## 4. 配置恢复

“恢复原配置”不是简单覆盖整个 `config.toml`：

- 先恢复任何未完成配置事务。
- 新建 `lifecycle_restore_config` 事务并快照四个受管文件。
- 只移除助手当前和旧版管理块、provider 与模型目录字段。
- 使用同目录原子替换保留其他用户 TOML。
- 修改后再次确认受管配置已消失，再提交事务。
- 任意写入、验证或提交失败都回滚；回滚失败继续使用 `ROLLBACK_FAILED`。

重复调用时没有受管配置则返回 `not_needed`，不生成重复 provider 或破坏快照。

## 5. 数据删除

助手数据根目录只包含助手自己的运行状态、事务快照、主题缓存和保存的 Router Key。

核心在删除前再次读取真实 `config.toml`。只要受管配置仍存在，就返回
`LIFECYCLE_DATA_IN_USE`，不删除任何文件。这样避免留下引用已删除 helper、模型目录
或密钥的失效 Codex 配置。

数据已不存在时返回 `not_needed`。

## 6. 助手与 ChatGPT 卸载

Windows：

- 只接受当前助手可执行文件同目录的 `uninstall.exe`。
- `run_lifecycle_action` 启动 NSIS 卸载器并返回 `handoff_started` 回执。
- 前端将回执写入页面后调用 `complete_assistant_uninstall_handoff`，助手才请求退出。
- 两阶段交接不依赖猜测 WebView 渲染耗时，退出前可以稳定展示和记录回执。
- NSIS 默认卸载只删除程序和注册项。
- `%LOCALAPPDATA%\CodexAssistant`、`~/.codex/config.toml` 和
  `OpenAI.Codex` Store 包不属于 NSIS 卸载范围。

macOS：

- 在 Finder 中定位当前签名 `.app`，由用户移到废纸篓。
- 不删除 Application Support 数据或 Codex 配置。

ChatGPT：

- Windows 打开系统“已安装的应用”。
- macOS 在 Finder 中定位官方 ChatGPT 应用。
- 最终卸载由操作系统单独确认，助手不直接移除官方包。

## 7. 稳定错误

| code | 含义 |
| --- | --- |
| `LIFECYCLE_ACTION_INVALID` | 动作 ID 不在固定 allowlist |
| `LIFECYCLE_CONFIRMATION_REQUIRED` | 修改性动作缺少精确确认 |
| `LIFECYCLE_DATA_IN_USE` | 受管配置仍依赖助手数据 |
| `ASSISTANT_UNINSTALLER_MISSING` | 当前不是完整安装版或卸载入口缺失 |
| `LIFECYCLE_STATUS_FAILED` | 无法读取足够状态 |
| `LIFECYCLE_ACTION_FAILED` | 没有更具体分类的生命周期动作失败 |

## 8. 验证矩阵

自动测试：

1. 默认卸载保留三个外部对象。
2. 修改性动作必须精确确认。
3. 受管配置存在时拒绝删除数据。
4. 未知或畸形动作 ID 被拒绝。
5. 官方应用管理是独立、非删除性 handoff。

Windows UI E2E：

1. 普通配置完成后四个状态来自 `LifecycleStatusV1`。
2. 旧 `factory_reset` 按钮和命令不存在。
3. 直接删除数据返回 `LIFECYCLE_DATA_IN_USE`，文件不变。
4. 页面确认后恢复配置，收据为 `managedConfig true -> false`，用户 profile 保留。
5. 页面确认后删除数据，收据为 `assistantData true -> false`，ChatGPT 仍可信安装。
6. 对精确候选执行 NSIS 静默卸载，确认助手注册和程序删除，但配置、助手数据 sentinel
   和 `OpenAI.Codex` 包保留；随后重装同一候选。

脚本：

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\windows-lifecycle-e2e.ps1 `
  -TestDefaultUninstall `
  -InstallerPath C:\path\CodexAssistant-0.8.8-windows-arm64-setup.exe `
  -ExpectedSha256 <sha256>
```

## 9. 当前证据

2026-07-28：

- macOS 宿主格式、严格 Clippy、前端语法和 Rust 全量测试通过：67 通过、0 失败、
  1 个需要真实 Ollama 的测试忽略。
- Windows ARM64/x64 目标测试结果相同，两个 NSIS 候选均构建成功。
- ARM64 原生和 x64 兼容层均通过安装不自启、首次响应、单实例、完整 UI E2E、
  交互卸载 handoff、默认卸载保留和精确候选重装。
- ARM64 候选：5,623,641 字节，SHA256
  `515e135549ca02715a557f7a695bc96405cb5433a8f2ada9fc2ae5c0c568337c`。
- x64 候选：6,007,909 字节，SHA256
  `46eb890c23cabfd69bac0daf785392453114dd77b263ab2ff1c2b97bb92cf427`。
- 两个生命周期 E2E 均返回 `interactiveHandoffVerified=true`、
  `assistantRemoved=true`、`configPreserved=true`、`dataPreserved=true`、
  `officialAppPreserved=true` 和 `candidateReinstalled=true`。

## 10. 后续门禁

- Windows x64 真实硬件的完整卸载和重装。
- 签名 macOS 候选的 Finder/废纸篓行为与数据保留。
- Windows 卸载取消、占用文件、损坏卸载器和升级中断。
- MDM/Intune 静默卸载退出码。
- `WP-604` 签名自更新和旧版本保留。
