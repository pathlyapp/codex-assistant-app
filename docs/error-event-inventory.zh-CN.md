# M0 错误与事件清单

本清单是 M1 契约和验收夹具的事实源。Tauri command 统一返回
`ErrorEnvelopeV1`；`String` 仅允许保留在 Rust 内部实现层。前端不得依赖中文字符串
包含关系决定动作。

## 1. 稳定错误分类

| 稳定 code | 当前来源示例 | stage | recoverable | suggestedAction |
| --- | --- | --- | --- | --- |
| `APP_NOT_INSTALLED` | 未检测到 ChatGPT | `preflight` | true | `install_app` |
| `APP_PACKAGE_UNTRUSTED` | 包名、publisher 或来源不符合要求 | `install_chatgpt` | false | `open_diagnostics` |
| `APP_INSTALL_FAILED` | winget 失败或安装后仍未检测到 | `install_chatgpt` | true | `retry_install` |
| `APP_RESTART_REQUIRED` | 系统完成安装前需要重启 | `install_chatgpt` | true | `restart_system` |
| `UNSUPPORTED_PLATFORM` | 当前平台不支持自动安装或启动 | `preflight` | false | `open_install_guide` |
| `ROUTER_URL_INVALID` | Router 地址格式不正确 | `validate_router_models` | true | `edit_gateway` |
| `ROUTER_DNS_FAILED` | 主机名解析失败 | `validate_router_models` | true | `edit_gateway` |
| `ROUTER_CONNECTION_REFUSED` | 目标端口拒绝连接 | `validate_router_models` | true | `check_router` |
| `ROUTER_TIMEOUT` | 连接或读取超时 | `validate_router_models` | true | `retry_router` |
| `ROUTER_TLS_FAILED` | TLS、证书或企业 CA 失败 | `validate_router_models` | true | `open_diagnostics` |
| `ROUTER_AUTH_FAILED` | HTTP 401/403 | `validate_router_models` | true | `edit_key` |
| `ROUTER_VM_LOOPBACK` | Windows ARM64 VM 将 `127.0.0.1` 指向虚拟机自身 | `validate_router_models` | true | `edit_gateway` |
| `ROUTER_LOCAL_SERVICE_MISSING` | 本机 Ollama 未安装或未启动 | `validate_router_models` | true | `check_router` |
| `ROUTER_OLLAMA_HOST_UNREACHABLE` | 宿主机 Ollama 未监听可访问接口或桥接未启动 | `validate_router_models` | true | `edit_gateway` |
| `ROUTER_MODELS_INVALID` | `/models` 为空或结构不兼容 | `validate_router_models` | true | `check_router` |
| `ROUTER_MODEL_UNAVAILABLE` | 所选模型不在返回列表 | `validate_router_models` | true | `select_model` |
| `ROUTER_RESPONSES_UNSUPPORTED` | `/responses` 不能完成最小请求 | `validate_router_response` | true | `check_router` |
| `CONFIG_PERMISSION_DENIED` | 目录或文件写入被拒绝 | `configure_codex` | true | `repair_permissions` |
| `CONFIG_PARSE_FAILED` | 现有配置不是有效 TOML | `configure_codex` | true | `restore_config` |
| `CONFIG_OVERRIDDEN` | 项目或管理员配置覆盖用户配置 | `verify` | true | `show_effective_source` |
| `CONFIG_VERIFY_FAILED` | 写入后内容、模型或应用复核失败 | `verify` | true | `restore_config` |
| `ROLLBACK_FAILED` | 自动恢复失败；活动 journal 和事务 manifest 保留 | `rollback` | false | `contact_support` |
| `SECRET_STORE_FAILED` | DPAPI、Keychain 或 helper 失败 | `configure_codex` | true | `open_diagnostics` |
| `PROXY_AUTH_REQUIRED` | 企业代理要求认证 | `validate_router_models` | true | `configure_proxy` |
| `APPEARANCE_UNSUPPORTED` | 平台、版本或主题类型不支持 | `appearance_apply` | false | `open_diagnostics` |
| `APPEARANCE_IMAGE_INVALID` | 图片格式、尺寸或内容不符合要求 | `appearance_import` | true | `choose_image` |
| `APPEARANCE_GALLERY_UNAVAILABLE` | 在线主题库网络或响应失败 | `appearance_gallery` | true | `retry_gallery` |
| `APPEARANCE_PACKAGE_INVALID` | 主题包完整性或安全校验失败 | `appearance_apply` | true | `retry_gallery` |
| `APPEARANCE_STORAGE_FAILED` | 本地主题目录无法写入 | `appearance_*` | true | `repair_permissions` |
| `APPEARANCE_STATE_FAILED` | 外观状态或内置主题读取失败 | `appearance_status/presets` | true | `retry_appearance` |
| `APPEARANCE_APPLY_FAILED` | ChatGPT 主题启动或注入失败 | `appearance_apply` | true | `retry_appearance` |
| `REPAIR_NOT_AVAILABLE` | 当前状态已经没有请求的自动修复动作 | `repair_execute` | true | `refresh_repair_plan` |
| `REPAIR_PLAN_STALE` | 执行前复核发现方案已随状态变化 | `repair_execute` | true | `refresh_repair_plan` |
| `REPAIR_PLAN_FAILED` | 无法读取足够状态生成修复方案 | `repair_plan` | true | `retry_repair_plan` |
| `REPAIR_EXECUTION_FAILED` | 修复失败且没有更具体的领域错误码 | `repair_execute` | true | `retry_repair` |
| `LIFECYCLE_ACTION_INVALID` | 生命周期动作不在固定 allowlist | `lifecycle_action` | false | `refresh_lifecycle_status` |
| `LIFECYCLE_CONFIRMATION_REQUIRED` | 修改性动作缺少精确确认 | `lifecycle_action` | true | `confirm_lifecycle_action` |
| `LIFECYCLE_DATA_IN_USE` | 受管配置仍依赖助手运行数据 | `lifecycle_action` | true | `restore_pre_assistant_config` |
| `ASSISTANT_UNINSTALLER_MISSING` | 完整安装版卸载入口缺失 | `lifecycle_action` | true | `open_system_apps` |
| `LIFECYCLE_STATUS_FAILED` | 无法读取应用与数据边界状态 | `lifecycle_status` | true | `retry_lifecycle_status` |
| `LIFECYCLE_ACTION_FAILED` | 生命周期动作失败且无更具体分类 | `lifecycle_action` | true | `retry_lifecycle_action` |
| `OPERATION_BUSY` | 另一项修改性操作正在执行 | 当前 command | true | `wait_for_operation` |
| `UPDATE_NOT_CONFIGURED` | 构建未内置可信 endpoint 与公钥 | `update_check` | false | `open_diagnostics` |
| `UPDATE_BUSY` | 更新步骤或其它修改性操作正在执行 | `update_*` | true | `wait_for_update` |
| `UPDATE_CHECK_FAILED` | 更新清单网络、格式或服务失败 | `update_check` | true | `retry_update_check` |
| `UPDATE_NOT_AVAILABLE` | 尚未检查到可下载的新版本 | `update_download/install` | true | `check_for_update` |
| `UPDATE_DOWNLOAD_FAILED` | 更新包下载、大小或网络失败 | `update_download` | true | `retry_update_download` |
| `UPDATE_SIGNATURE_INVALID` | Minisign、Base64 或 detached signature 无效 | `update_download` | false | `export_diagnostics` |
| `UPDATE_NOT_DOWNLOADED` | 尚未得到已验证更新包 | `update_install` | true | `download_update` |
| `UPDATE_INSTALL_FAILED` | 平台安装器未完成 | `update_install` | true | `retry_update_install` |
| `UPDATE_RECEIPT_FAILED` | 更新收据无法原子写入 | `update_install/update_health` | true | `open_diagnostics` |
| `UPDATE_STATE_UNAVAILABLE` | 更新状态锁或收据不可用 | `update_*` | true | `restart_assistant` |
| `INTERNAL_TASK_FAILED` | 后台任务 join/panic 或未知失败 | 当前 command | true | `retry` |

主题、图库和外观错误使用独立 `appearance_*` stage，不改变核心
`SystemStatus.overall`。

## 2. 当前工作流事件

事件名：`installer-stage`

当前内部阶段：

| 顺序 | stage | 当前状态 | 主要缺口 |
| --- | --- | --- | --- |
| 1 | `preflight` | `running/complete/failed` | 取消语义转入 M5 |
| 2 | `install_chatgpt` | `running/complete/skipped/failed` | 安装来源和可信结果未结构化 |
| 3 | `validate_router` | `running/complete/failed` | `/models`、模型选择和响应大小受限 |
| 4 | `validate_router_response` | `running/complete/failed` | SSE/JSON 完成、模型一致性和流中断已验证；相同 Router/模型失败会撤销旧验证证据但不改配置 |
| 5 | `configure_codex` | `running/complete/failed` | 快照成功后才写四个受管文件；事件包含 transaction ID |
| 6 | `verify` | `running/complete/failed` | 从磁盘复查配置、模型和 Responses 证据；成功后提交事务 |
| 7（按需） | `rollback` | `running/restored/failed` | 写入后失败自动恢复；失败时 `ROLLBACK_FAILED` 覆盖原错误 |

当前事件字段：`schemaVersion`、`operationId`、`stage`、`label`、`status`、
`message`、`current`、`total`、`cancellable`、`recoverable`、`details`。
合法状态为 `waiting|running|complete|skipped|failed|restored`，非法转换已有单元
测试阻断。`rollback` 只在快照后失败时追加，不改变六个主阶段的总数；前端必须优先
展示 rollback 失败，不能继续显示被覆盖的原始 verify/config 错误。

## 3. 当前命令边界

核心命令已返回 `ErrorEnvelopeV1`：

- `get_system_status`
- `get_repair_plan`
- `run_repair`
- `discover_models`
- `start_setup`
- `install_chatgpt_app`
- `launch_chatgpt`
- `restart_chatgpt`
- `restore_codex_config`
- `disconnect_router`
- `get_lifecycle_status`
- `run_lifecycle_action`
- `complete_assistant_uninstall_handoff`

外观命令也使用同一 envelope，并以 `appearance_*` stage 隔离：

- `get_appearance_status`
- `apply_appearance`
- `import_theme_image`
- `list_preset_themes`
- `list_gallery_themes`

诊断导出使用 `export_diagnostics` 命令和 `diagnostics_export` stage。二次扫描发现疑似
凭据或用户目录时返回 `DIAGNOSTIC_SECRET_DETECTED`，并阻止生成可下载文件。

助手更新命令也使用同一 envelope：

- `get_assistant_update_status`
- `check_for_assistant_update`
- `download_assistant_update`
- `install_assistant_update`
- `confirm_assistant_update_health`

更新状态通过独立 `assistant-update-status` 事件发布，不混入 setup 的
`installer-stage`。更新失败不得改变核心 `SystemStatus.overall`，也不得把
ChatGPT 或 Router 状态标记为失败。

底层文件、网络和平台函数仍可在 Rust 内部使用 `Result<_, String>`，但字符串错误必须
在 command 边界转换并经过统一脱敏后才能进入前端。

## 4. 脱敏约束

`technical` 和日志中禁止出现：

- Bearer Token、Access Key、URL userinfo 和查询参数中的凭据。
- DPAPI/Keychain 解密结果。
- 请求体中的用户业务文本。
- 可直接识别个人的完整用户目录；导出诊断时应缩写为用户目录标记。

允许保留：

- 安全的 HTTP 状态码。
- 已脱敏 host、端口和路径类别。
- Router 返回的安全 request ID。
- support ID、operation ID、stage 和稳定 code。
