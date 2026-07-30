# WP-604A 助手更新客户端

本文定义 `WP-604A` 的实现和验收边界，关联 `LIFE-004`、`LIFE-005`、
`SECR-008`、`REL-011`、`REL-032` 和 `REL-043`。

## 1. 目标

在不放宽 `WP-700` 商业发布门禁的前提下，先完成助手侧可测试的更新能力：

- 根据当前平台、架构和版本查询动态更新清单。
- 下载更新包并使用编译进客户端的 Tauri Minisign 公钥验证签名。
- 由用户明确确认后安装并重启助手。
- 用版本化状态、稳定错误码、进度事件和本地收据描述全过程。
- 提供仅监听回环地址的模拟服务，验证可用更新、无更新、服务失败和无效签名。

更新只作用于 Codex 助手，不更新官方 ChatGPT，也不修改 Router、Key、Codex 配置、
主题或诊断数据。普通用户不选择操作系统和 CPU 架构。

## 2. 非目标和剩余门禁

`WP-604A` 不等于完整 `WP-604`，也不等于客户可发布：

- 不实现 Windows Authenticode、macOS Developer ID、公证和 stapling。
- 不实现客户 CDN、短期下载 URL、版本撤销、灰度和审计。
- 不把 GitHub Release 或模拟服务作为正式更新服务。
- 不承诺安装程序已经开始替换文件后仍能自动恢复旧版本。
- 不实现独立于主程序的恢复代理或旧版本槽位。
- 不接管官方 ChatGPT 自身的更新。

完整 `WP-604` 仍依赖 `WP-701`、`WP-702` 和后续 `WP-604B` 恢复机制。未通过这些
门禁的构建只能用于内部测试，`customerReady` 必须保持 `false`。

## 3. 架构

```text
诊断页更新面板
  -> Tauri command
  -> AssistantUpdaterState
  -> tauri-plugin-updater
  -> HTTPS 动态清单
  -> 平台/架构匹配
  -> 下载 + Minisign 验证
  -> 用户确认
  -> 平台安装器
  -> 重启后健康确认收据
```

信任根由构建时环境写入二进制：

- `CODEX_ASSISTANT_UPDATE_ENDPOINT`
- `CODEX_ASSISTANT_UPDATE_PUBKEY`
- `CODEX_ASSISTANT_UPDATE_CHANNEL`

Release 构建不读取运行时同名环境变量，客户端也不提供修改更新地址或公钥的 UI。
正式 endpoint 必须为 HTTPS。只有显式 `updater-mock` feature 可使用
`127.0.0.1`、`localhost` 或 `::1` 的 HTTP 地址，LAN HTTP 仍被拒绝。

普通构建没有同时提供 endpoint 和公钥时，状态必须是 `not_configured`，不会伪装成
已启用，也不会发起下载。

## 4. 客户端协议

Rust command：

- `get_assistant_update_status`
- `check_for_assistant_update`
- `download_assistant_update`
- `install_assistant_update`
- `confirm_assistant_update_health`

事件：

- `assistant-update-status`

`UpdateStatusV1` 主要字段：

```json
{
  "schemaVersion": 1,
  "phase": "not_configured|idle|checking|up_to_date|available|downloading|ready_to_install|installing|restart_required|failed",
  "currentVersion": "0.9.0",
  "channel": "internal-test|beta|stable",
  "platform": "windows|darwin",
  "architecture": "x86_64|aarch64",
  "configured": true,
  "availableVersion": "0.9.1",
  "downloadedBytes": 0,
  "totalBytes": null,
  "progressPercent": null,
  "verification": "not_started|verified|failed",
  "canCheck": true,
  "canDownload": false,
  "canInstall": false,
  "requiresRestart": false,
  "blockerCode": null,
  "lastUpdate": null
}
```

`check` 和 `download` 在后台异步执行，前端只消费状态，不用计时器伪造进度。
`download()` 返回前由 Tauri 完成签名校验，只有此后才能进入
`ready_to_install/verified`。

## 5. 并发与安装语义

更新状态机拒绝并发 check/download/install。安装阶段还与配置、官方应用安装、修复、
生命周期和外观修改共用 Rust 全局修改操作门禁。前端禁用按钮只是交互提示，不能替代
核心门禁。

清单中的实际下载地址会再次执行传输规则校验：正式构建只接受 HTTPS，mock 构建只
额外接受回环 HTTP，并拒绝 URL 内嵌凭据。下载完成并通过 Tauri 签名验证后，客户端
只接纳不超过 256 MiB 的安装包；Tauri 当前 API 会先在内存中完成下载，因此生产 CDN
还必须限制对象大小。发布说明最多展示 4,000 个字符并只通过 `textContent` 渲染。
endpoint、签名和下载包均不得进入诊断日志正文。

安装前原子写入：

```text
<app-data>/updates/update-state.json
```

状态从 `pending_install` 开始。安装失败时记录 `install_failed`；新版本启动且核心状态
读取成功后，`confirm_assistant_update_health` 将相同目标版本标记为 `healthy`。
收据不包含 Key、Router URL、用户名或项目路径。

在下载、签名验证和收据写入失败时，平台安装器不会启动，当前版本保持不变。平台安装器
开始替换文件后的强制回退不属于 A 阶段保证，必须由后续外部恢复机制关闭。

## 6. 构建

构建包装器 `tools/build-updater.mjs` 会：

1. 校验 endpoint、channel、公钥和私钥是否存在。
2. 正式构建拒绝非 HTTPS，mock 构建拒绝非回环 HTTP。
3. 生成权限为 `0600` 的临时 Tauri 配置，写入公开 key 和 endpoint。
4. 调用 Tauri 生成更新包与 detached `.sig`。
5. 无论成功或失败都删除临时配置。

私钥只进入 Tauri 子进程环境，不写入配置、日志或 Git。`.local-update/` 和所有
`target/` 产物由 `.gitignore` 排除。

生成本地测试密钥：

```bash
cd tauri-gui
npm exec tauri signer generate -- \
  --ci \
  --write-keys .local-update/updater.key
```

macOS 本地 mock 签名构建：

```bash
export TAURI_SIGNING_PRIVATE_KEY_PATH=.local-update/updater.key
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=
export CODEX_ASSISTANT_UPDATE_PUBKEY="$(cat .local-update/updater.key.pub)"
export CODEX_ASSISTANT_UPDATE_ENDPOINT='http://127.0.0.1:4317/updates/{{target}}/{{arch}}/{{current_version}}'
export CODEX_ASSISTANT_UPDATE_CHANNEL=internal-test
npm run build:update:mock:mac
```

Windows PowerShell：

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY_PATH = ".local-update\updater.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
$env:CODEX_ASSISTANT_UPDATE_PUBKEY = Get-Content ".local-update\updater.key.pub" -Raw
$env:CODEX_ASSISTANT_UPDATE_ENDPOINT = "http://127.0.0.1:4317/updates/{{target}}/{{arch}}/{{current_version}}"
$env:CODEX_ASSISTANT_UPDATE_CHANNEL = "internal-test"
npm run build:update:mock:windows
```

正式签名产物使用 `build:update:mac` 或 `build:update:windows`，endpoint 必须是 HTTPS。
CI 必须从受托 secret 或签名服务提供私钥，不能生成临时客户密钥。

## 7. 本地模拟服务

示例：

```bash
node tools/updater-test-server.mjs \
  --artifact "src-tauri/target/release/bundle/macos/Codex 助手.app.tar.gz" \
  --signature "src-tauri/target/release/bundle/macos/Codex 助手.app.tar.gz.sig" \
  --version 0.9.1 \
  --target darwin \
  --arch aarch64 \
  --port 4317
```

可选 `--mode`：

- `available`
- `none`
- `invalid-signature`
- `error`

服务固定只允许回环监听，动态清单路径为：

```text
/updates/{windows|darwin}/{x86_64|aarch64}/{currentVersion}
```

完整安装 E2E 必须使用两个真实版本：先安装低版本客户端，再在隔离工作树中把所有版本
字段升级后生成高版本签名包。禁止只修改清单版本、却用低版本二进制声明安装成功。

## 8. 验收矩阵

| 场景 | A 阶段要求 |
| --- | --- |
| 未配置更新服务 | 显示未启用，不发网络请求 |
| 当前已是最新 | 返回 `up_to_date`，无下载按钮 |
| 平台或架构不匹配 | 返回 204，不提供错误包 |
| 有新版本 | 显示真实版本和发布说明 |
| 下载进度 | 使用真实字节数和百分比 |
| 签名有效 | 进入 `ready_to_install/verified` |
| 签名无效 | `UPDATE_SIGNATURE_INVALID`，禁止安装 |
| 服务失败 | `UPDATE_CHECK_FAILED`，当前版本不变 |
| 下载失败 | `UPDATE_DOWNLOAD_FAILED`，当前版本不变 |
| 与配置/修复并发 | 核心返回稳定 busy 错误 |
| 用户取消安装确认 | 不调用安装 command |
| 新版本健康启动 | 收据从 `pending_install` 变为 `healthy` |

当前代码级退出证据：

- macOS 宿主和 Windows ARM64 均为 78 通过、0 失败、1 个本地 Ollama live test
  忽略；Rust 格式和严格 Clippy 通过。
- Node 发布策略 6/6、updater 构建门禁 4/4、回环模拟服务 5/5 通过；基础 Tauri
  配置不得包含不完整 updater 节点，普通构建仅在完整信任根存在时注册插件。
- macOS ARM64 真实 `.app.tar.gz` 为 8,858,305 字节，SHA256
  `1e392972ae056deb64444cbf6a6ffe48854c16a3442ea3b9bc2aa1c7e9da8b8b`；
  `.sig` 为 412 字节，SHA256
  `b58e96d4bb37c30fe4b29cd3a698fd513df2f9159a9927fb038d465ef943b98d`。
- Windows ARM64 普通 NSIS 为 6,121,282 字节，SHA256
  `a4ece75c4ee0ad84991d0a65f94f2d09297b4321a38b9c1f1ca1f7569a47cc81`；
  安装不提前启动、首次启动响应和双启动单实例烟雾测试通过。
- Windows ARM64 updater E2E 使用真实 `0.9.0` 与 `0.9.1` 二进制。基线 SHA256 为
  `672f98a834eb93bddad8ce116eb26598236109dfbcdec3b6558b38679ddd4a95`；
  更新包 SHA256 为
  `a76d3c741a51159a7638db92c3df790bde34e1af88d1e6612e2063f9b62995c0`；
  detached signature SHA256 为
  `2c74e6d56ed155d5a13260ba270dec1808818eb06e7f3b915d38e2843c216dfb`。
- E2E 通过真实页面完成
  `available -> ready_to_install/verified -> install -> restart -> healthy`，下载
  进度为 100%，系统注册版本和产品版本均为 `0.9.1`，重启进程可响应。
- 测试后虚拟机已恢复并打开普通 `0.9.0`，未保留 mock 信任根。

`WP-604A` 客户端阶段据此标记为已验证。真实 Windows x64、受签名客户包、HTTPS
客户服务、撤销审计和平台安装开始后的失败自动回退继续保持独立门禁。
