# M6 脱敏诊断包实施契约

## 1. 目标与边界

本工作包实现 `WP-601` 的第一阶段：用户在“帮助与诊断”页面一键导出可交给客服的
结构化诊断包。诊断包由 Rust 核心生成，前端不得自行拼装文件内容，也不得传入原始
日志、技术错误、配置文件或凭据。

本阶段不实现日志上传、遥测、自更新、卸载数据策略和远程客服接口。这些能力必须在
后续工作包中复用本契约，不得建立第二套未脱敏导出通道。

对应需求：

- `DIAG-002`：包含平台、官方应用、有效配置来源、Router、代理/CA、权限和事务摘要。
- `DIAG-003`：诊断页提供一键导出，保留一键复制摘要。
- `DIAG-004`：固定 manifest、状态、近期日志和校验和格式。
- `DIAG-005`：导出前再次扫描疑似密钥和用户路径，命中后阻止导出。
- `DIAG-006`：文件名、manifest 和状态使用同一个 support ID。
- `SECR-001`, `SECR-007`, `REL-012`：凭据不进入诊断，有界日志缓冲。

## 2. 命令契约

Tauri command：

```text
export_diagnostics(request) -> DiagnosticBundle
```

前端允许传入的 request 字段只有：

- `supportId`：最近一次结构化错误的 support ID；格式不合法时由核心重新生成。
- `errorCode`：稳定错误码。
- `errorStage`：稳定阶段 ID。
- `suggestedAction`：稳定恢复动作 ID。

这三个错误字段只接受 ASCII 字母、数字、下划线、短横线和点，单字段最多 64 个字符。
前端不得传入 `technical.detail`、错误堆栈、Access Key、请求或响应正文。

成功返回：

```json
{
  "fileName": "diagnostics-CA-....zip",
  "contentBase64": "<zip bytes>",
  "byteLength": 1234,
  "sha256": "<lowercase sha256>",
  "supportId": "CA-...",
  "savedPath": "<system Downloads path>"
}
```

Rust 将文件原子写入系统 Downloads 目录，写入前后都复核字节数和 SHA256。前端只
显示成功收据，不参与文件内容、保存目录或命名。

## 3. 文件格式

ZIP 必须且只能包含以下四个根目录文件：

```text
diagnostics-<support-id>.zip
├── manifest.json
├── status.json
├── recent.log
└── checksums.txt
```

`manifest.json` 包含：

- schema version、support ID、生成时间。
- 助手版本、平台和架构。
- `status.json` 与 `recent.log` 的字节数和 SHA256。

`status.json` 包含：

- 助手版本、平台和架构。
- 官方应用的状态、可信结果、来源和版本。
- 配置状态、有效来源、备份可用性和最近事务 ID。
- Router 状态、连接结果、协议、host、端口和最近验证时间。
- 只记录“哪些”代理或 CA 环境变量存在，不记录变量值。
- 只记录配置目录和运行目录是否存在、是否只读，不记录目录路径。
- 最近事务的 ID、操作、结果、完成时间和已脱敏失败摘要。
- 最近错误的稳定 code、stage 和 suggested action。

`recent.log` 来自 Rust 内存环形缓冲。单行最多 8 KiB，总量最多 256 KiB；进入缓冲前
执行一次核心脱敏，导出时再次执行。没有日志时写入固定英文占位行。

`checksums.txt` 采用常见的：

```text
<sha256>  manifest.json
<sha256>  status.json
<sha256>  recent.log
```

校验和文件不递归包含自身哈希。

## 4. 数据最小化

允许：

- 版本、平台、架构和稳定状态 ID。
- 安全的 Router protocol、host 和 port。
- HTTP/连接结果的脱敏摘要。
- 环境变量名称是否存在。
- 事务结果和 support ID。

禁止：

- 完整 `config.toml`、运行时状态文件和主题内容。
- Access Key、Bearer Token、DPAPI/Keychain 解密结果。
- URL userinfo 和查询参数中的凭据。
- 用户名、用户主目录和项目路径。
- 用户 prompt、模型响应正文和业务文件内容。
- 原始 command `technical.detail`、崩溃转储和任意前端文本。

## 5. 二次扫描与失败行为

在写 ZIP 前，核心对 `manifest.json`、`status.json` 和 `recent.log` 再次扫描：

- 未脱敏 Bearer。
- `key/token/api_key/access_key` 的赋值或 JSON 字符串值。
- URL userinfo。
- Windows、macOS、Linux 原始用户主目录。
- 常见 `sk-`、`ghp_`、`xoxb-` 凭据前缀。

任何命中都必须停止导出，不返回部分 ZIP。command 使用：

```text
code: DIAGNOSTIC_SECRET_DETECTED
stage: diagnostics_export
suggestedAction: retry_diagnostics
```

UI 显示安全文案，不显示匹配值、位置或原始文本。

下载目录创建、写入或写后完整性复核失败时返回 `DIAGNOSTIC_EXPORT_FAILED`，不得显示
成功提示。

## 6. 验收门禁

单元测试：

- ZIP 恰好包含四个文件。
- 返回字节数和整体 SHA256 正确。
- `checksums.txt` 与三个文本文件逐项一致。
- Key、URL 凭据和三平台用户目录经过两次脱敏后不存在。
- 未脱敏密钥与路径扫描会失败。
- request/response 字段保持 camelCase。
- `DIAGNOSTIC_SECRET_DETECTED` 映射稳定。

Windows E2E：

- 从真实 Tauri WebView2 调用 `export_diagnostics`。
- Base64 可解码，整体长度和 SHA256 与收据一致。
- 内存中打开 ZIP 并核对四个文件。
- 对候选 Key、Bearer、用户目录和常见凭据前缀执行扫描。
- 逐项复算三个文本文件 SHA256。

发布前仍需：

- Windows ARM64 原生 E2E。
- Windows x64 目标编译、测试与至少兼容层 E2E；真实 x64 仍是独立门禁。
- macOS 正式签名候选上的导出 E2E。

## 7. 后续工作包

后续 M6 按以下顺序推进：

1. `WP-602`：错误码驱动的定向修复动作和修复结果。
2. `WP-603`：安装、升级、修复、卸载及三种数据保留策略。
3. `WP-604`：签名更新清单、旧版本保留和更新失败回退。
4. `WP-605`：本地阶段质量指标；默认不上传，任何上传都需要独立隐私评审和用户同意。

客服上传服务只能接收本格式，并在服务端再次校验 schema、文件 allowlist、大小、
SHA256 和疑似密钥；不得接受任意压缩包或原始日志。
