# installer-core 协议

0.8.4 的 installer-core 完全由 Rust 实现，不调用 Go sidecar。前端只消费 Tauri command 和结构化事件，不理解 Appx、DPAPI 或 TOML 写入细节。

## 命令

### `get_system_status`

返回：

- 官方 ChatGPT 是否安装及版本。
- 当前平台与 CPU 架构。
- Codex managed config 是否完整。
- 已配置 Router/模型。
- Router `/v1/models` 是否真实可访问。
- Key 是否已配置，不返回 Key 内容。
- 是否存在可恢复的 `config.toml.bak.<timestamp>`。

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
-> configure_codex
-> verify
```

- `preflight`：创建目录，检查配置写入条件，检测官方 ChatGPT 或 winget。
- `install_chatgpt`：已安装则跳过；否则通过 Microsoft Store ID `9PLM9XGG6VKS` 安装并再次检测。
- `validate_router`：请求 `/v1/models`，核对选择的模型；连接失败即停止。
- `configure_codex`：加密 Key、写状态/model catalog、备份并使用结构化 TOML API 更新用户级 `config.toml`；只替换助手管理的 provider，保留 ChatGPT 其它设置。
- `verify`：重新读取状态和配置，再次请求 Router，并再次确认官方 ChatGPT。

任何阶段失败都会产生 `failed` 事件和非成功结果，不允许用 warning 代替完成条件。

### `launch_chatgpt`

只在用户主动点击后调用。Windows 使用已验证 Appx 的 PackageFamilyName/AppId；配置过程不会调用此命令。若用户已选择主题，则通过安全主题启动链路打开。

### `restart_chatgpt`

只在用户确认后调用。先关闭已验证的 ChatGPT 进程，短暂等待后再通过官方 Appx 标识启动，保证新的 Codex 配置被重新加载。

### `restore_codex_config`

恢复时间戳最新的完整快照，包括 `config.toml`、助手运行状态、模型目录和可选 DPAPI Key。恢复前先为当前状态生成一个新的完整快照，因此再次恢复可以撤销本次操作；前端随后执行用户已确认的 ChatGPT 重启。

### `get_appearance_status` / `apply_appearance`

当前主题：

- `official`：官方外观。
- `focus`：专注深色。

非官方主题会重启 ChatGPT，使用回环 CDP 注入可撤销样式。只接受 `ws://127.0.0.1:<port>/devtools/page/<id>` 和 `app://` 页面。Windows 还必须验证监听进程路径属于官方 `OpenAI.Codex` Store package。若官方 ChatGPT 尚停留在 “Finish Windows setup”，命令返回失败并保持 `official` 状态。

## 事件

`installer-stage`：

```json
{
  "stage": "validate_router",
  "label": "验证 Router",
  "status": "running|complete|skipped|failed",
  "message": "用户可读状态",
  "current": 3,
  "total": 5,
  "recoverable": false,
  "details": {}
}
```

`installer-log`：脱敏文本日志。

`installer-finished`：最终结果及已完成阶段。前端对重复 event/invoke 返回做幂等处理。

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
