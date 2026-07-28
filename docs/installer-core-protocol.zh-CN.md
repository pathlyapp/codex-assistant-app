# installer-core 协议

0.8.8 开发基线的 installer-core 完全由 Rust 实现，不调用 Go sidecar。前端只消费
Tauri command 和结构化事件，不理解 Appx、DPAPI 或 TOML 写入细节。

## 命令

### `get_system_status`

返回 `SystemStatusV1`。`schemaVersion` 当前为 `1`，Rust 统一计算 `overall` 和
`recommendedAction`：

```json
{
  "schemaVersion": 1,
  "overall": "ready|action_required|blocked",
  "app": {},
  "router": {},
  "config": {},
  "recommendedAction": {
    "id": "open_chatgpt",
    "label": "打开 ChatGPT"
  }
}
```

V1 同时保留 0.8.x 扁平字段一个兼容周期。前端适配器只允许从 `overall` 派生
`ready`，不得再组合应用、配置和 Router 三个布尔值判断整体成功。

状态内容：

- 官方 ChatGPT 是否安装及版本。
- 当前平台与 CPU 架构。
- Codex managed config 是否完整。
- 已配置 Router/模型。
- Router `/v1/models` 是否真实可访问，以及最近一次 `/v1/responses` 验证时间。
- Key 是否已配置，不返回 Key 内容。
- 是否存在可恢复的已提交事务快照。
- 最近事务 ID；中断事务无法恢复时，配置状态为 `rollback_failed`，整体状态为
  `blocked`，主动作进入诊断。

### `discover_models(request)`

输入 `gateway`、可选 `key`、`useSavedKey`，返回规范化 URL 和真实模型列表。只支持 `http`/`https`，拒绝查询参数和其它协议。

对于 `127.0.0.1:11434`，连接失败与模型列表为空会返回 Ollama 专用诊断。Windows ARM64 会提示虚拟机回环地址只指向 Windows 自身；远端 `:11434` 连接失败时会明确提示 Parallels 桥接地址和监听条件。

## 进程语义

- NSIS 安装文件复制和注册阶段不启动助手。
- 非静默安装只允许 Tauri 官方完成页在用户点击完成后启动助手。
- 静默或被动安装不启动助手。
- `tauri-plugin-single-instance` 保证重复启动只聚焦已有主窗口。

### `start_setup(options)`

```text
preflight
-> install_chatgpt
-> validate_router
-> validate_router_response
-> configure_codex
-> verify
```

- `preflight`：创建目录，检查配置写入条件，检测官方 ChatGPT 或 winget。
- `install_chatgpt`：已安装则跳过；否则通过 Microsoft Store ID `9PLM9XGG6VKS` 安装并再次检测。
- `validate_router`：请求 `/v1/models`，核对选择的模型；连接失败即停止。
- `validate_router_response`：使用相同网络、代理、CA 和认证路径发送固定低成本
  `/v1/responses` 流请求；要求收到结构有效、模型一致的 `response.completed`，不保存
  输出正文。
- `configure_codex`：加密 Key、写状态/model catalog、备份并使用结构化 TOML API 更新用户级 `config.toml`；只替换助手管理的 provider，保留 ChatGPT 其它设置。
- `verify`：重新读取状态和配置，再次请求模型列表，核对 Responses 验证证据，并再次确认官方 ChatGPT；全部通过后提交配置事务。

任何阶段失败都会产生 `failed` 事件、`ErrorEnvelopeV1` 和非成功结果，不允许用
warning 代替完成条件。只有 `/models` 和 `/responses` 均通过后才允许写配置并把
Router 状态标记为 `responses_verified`；旧状态没有验证证据时只显示
`models_verified`，整体状态不得为 `ready`。

`configure_codex` 写入前必须先创建配置事务。事务覆盖用户 `config.toml`、助手运行
状态、模型目录和可选加密 Key；在 `runtime/snapshots/<transactionId>/manifest.json`
中记录操作、时间、助手版本、原目标路径、是否存在、备份文件及 SHA256。所有 JSON
和 TOML 在替换前完成解析校验，使用同目录临时文件、同步落盘和原子替换。

创建快照后任一阶段失败都会追加一个 `rollback` 事件：

- 回滚成功：状态为 `restored`，事务为 `rolled_back`，删除活动 journal。
- 回滚失败：最终错误切换为 `ROLLBACK_FAILED`，结果退出码为 `2`，保留活动 journal、
  事务 manifest、目标路径和快照路径。
- 下次调用状态或开始配置时会检测 `transaction.active.json`。未完成写入会自动回滚；
  已落盘但尚未清理 journal 的 commit 会补写摘要后完成收口。

若相同 Router URL 和模型曾通过验证，但本次 `validate_router_response` 失败，核心只
撤销运行状态中的 `responsesVerifiedAt/responsesProtocol`，不改写既有
`config.toml`、模型目录或密钥。状态立即退回 `models_verified`，用户修复 Router 后
可以直接重试；新 Router 或新模型的失败不会错误清除其它组合的证据。

### `launch_chatgpt`

只在用户主动点击后调用。Windows 使用已验证 Appx 的 PackageFamilyName/AppId；配置过程不会调用此命令。若用户已选择主题，则通过安全主题启动链路打开。

### `restart_chatgpt`

只在用户确认后调用。先关闭已验证的 ChatGPT 进程，短暂等待后再通过官方 Appx 标识启动，保证新的 Codex 配置被重新加载。

### `restore_codex_config`

恢复最近的已提交事务快照，包括 `config.toml`、助手运行状态、模型目录和可选 DPAPI
Key。恢复前先为当前状态生成 `operation=restore` 的新事务；恢复成功后提交新事务，
失败则自动恢复操作前状态，因此再次恢复可以撤销本次操作。旧版时间戳快照只作为迁移
回退读取。前端随后执行用户已确认的 ChatGPT 重启。

### 外观命令

`get_appearance_status`、`apply_appearance`、`import_theme_image`、
`list_preset_themes` 和 `list_gallery_themes` 使用与核心命令相同的
`ErrorEnvelopeV1`，但 stage 固定在 `appearance_*` 命名空间。

当前主题：

- `official`：官方外观。
- `focus`：专注深色。

非官方主题会重启 ChatGPT，使用回环 CDP 注入可撤销样式。只接受 `ws://127.0.0.1:<port>/devtools/page/<id>` 和 `app://` 页面。Windows 还必须验证监听进程路径属于官方 `OpenAI.Codex` Store package。若官方 ChatGPT 尚停留在 “Finish Windows setup”，命令返回失败并保持 `official` 状态。

## 事件

`installer-stage`：

```json
{
  "schemaVersion": 1,
  "operationId": "uuid",
  "stage": "validate_router_response",
  "label": "验证实际请求",
  "status": "waiting|running|complete|skipped|failed|restored",
  "message": "用户可读状态",
  "current": 4,
  "total": 6,
  "cancellable": false,
  "recoverable": false,
  "details": {}
}
```

setup 的六个主阶段失败后可能追加第七个 `rollback` 阶段。该阶段不改变前六阶段的
`current/total`，通过稳定 `stage=rollback` 和事务 ID 记录恢复结果。

`installer-log`：脱敏文本日志。

`installer-finished`：包含相同 `operationId`、最终结果、可选 `ErrorEnvelopeV1` 及
已完成阶段。前端按 `operationId` 忽略过期或重复 event/invoke 返回。

核心命令失败返回：

```json
{
  "schemaVersion": 1,
  "code": "ROUTER_CONNECTION_REFUSED",
  "stage": "validate_router_models",
  "title": "Router 拒绝连接",
  "message": "请确认服务已启动、地址和端口正确，并允许当前设备访问。",
  "recoverable": true,
  "suggestedAction": "check_router",
  "supportId": "CA-<timestamp>-<random>",
  "technical": {
    "detail": "redacted"
  }
}
```

外观命令失败不得改变核心 `SystemStatus.overall`。

## Codex 配置

固定 provider：

```toml
model = "<selected model>"
model_provider = "codex_assistant_router"
model_catalog_json = "...\\models.json"

[model_providers.codex_assistant_router]
name = "Codex Assistant Router"
base_url = "http://127.0.0.1:11434/v1"
wire_api = "responses"
```

有 Key 时增加：

```toml
[model_providers.codex_assistant_router.auth]
command = "...\\codex-assistant.exe"
args = ["--codex-assistant-token-helper", "...\\config.json"]
```

`chatgpt_base_url` 不属于模型 Router，安装器不得修改。

更新由 `toml_edit` 完成。安装器不得覆盖顶层 `notify`、`[desktop]` 或其它非托管字段；旧版 marker/provider 只用于迁移清理。
