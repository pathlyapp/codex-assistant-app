# Windows VM 验收计划

## PD Windows ARM64 本地构建环境

Parallels `prlctl exec` 默认可能使用 SYSTEM 环境。构建可以使用 SYSTEM 的临时目录，
但安装和 UI 验收必须增加 `--current-user`，否则注册表和安装目录会落到
`C:\Windows\System32\config\systemprofile`，不代表客户用户环境。

当前 VM 的 Build Tools 位于 `C:\BuildTools`。本地构建统一使用脚本加载 MSVC/LLVM、
核对 Rust target、执行测试并输出带 SHA256 的标准命名候选包：

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\windows-build.ps1 -Architecture auto
```

在 ARM64 构建机交叉生成 x64 候选时，先执行一次
`rustup target add x86_64-pc-windows-msvc`，再传入 `-Architecture x64`。只调用
`cargo` 或 `npm` 而不加载对应环境会导致 `ring` 找不到 `clang`，或者链接阶段找不到
MSVC/Windows SDK。不要在 macOS 与 Windows 之间共用同一个 Cargo `target` 目录；
应先将源码复制到 VM 本地临时目录再构建。

2026-07-28 的 M1 候选证据：

- Windows 11 Pro ARM64。
- Rust 测试：28 passed，0 failed，1 ignored。
- NSIS：`CodexAssistant-0.8.8-windows-arm64-setup.exe`。
- ARM64 SHA256：`24ca87048a003cb8f23e801f3c138a502b8b81ada31fe5795ab2965bbf895491`。
- x64 SHA256：`bf5673b79248e6c0c23da49c09c2acdf095c82c7ddefe918ee56047fae4ecf0f`。
- 两份候选源提交：`af09091`。
- 0.8.4 -> 0.8.8 当前用户升级成功。
- ARM64 原生和 x64 兼容层均通过静默安装不自启、首次响应和重复启动单实例。
- x64 首次冒烟曾持续出现两个主窗口；加入 Windows 前置互斥/聚焦兜底后通过。真实
  x64 机器仍是发布门禁，兼容层结果不得替代。

同日 `9885c16` 的 M1 契约收口候选证据：

- Windows ARM64/x64 目标测试均为 31 passed、0 failed、1 ignored。
- ARM64 SHA256：
  `44af1a88df24f38b58f046e33413273b2cada5a35da70466ce040de81acb4b12`。
- x64 SHA256：
  `745b5e12054f8a09ca9a201d96487279dc6f311340547234d93e3b52dcfe41ea`。
- 两架构均通过静默安装不自启、首次响应和重复启动单实例；x64 仍仅为 ARM64
  兼容层证据。
- UI E2E 首次发现 Windows ARM64 Ollama 专用提示在 envelope 迁移后丢失；新增
  `ROUTER_VM_LOOPBACK` 后回归通过。
- 修复后的当前用户 E2E：配置期间 ChatGPT 进程数为 0，Router 模型选择、配置写入、
  备份入口和恢复 round-trip 通过。Router 为 Parallels 专用网卡上的隔离测试服务，
  只作为 `/models` 流程证据。
- E2E 额外断言首页状态徽标、配置页 Gateway 和诊断摘要与同一次
  `SystemStatusV1` 一致。

同日 `c120943` 的 M3 Responses 探针候选证据：

- Windows ARM64/x64 目标测试均为 40 passed、0 failed、1 ignored。
- ARM64 SHA256：
  `12ac72e27e1a12920a1173dd80cac1a860fc402648233d424aeb32e5770f2a1b`。
- x64 SHA256：
  `69c9ab9b885f6cff181c2e5d00dfdb717771802a23c5f01dc720062452324346`。
- ARM64 原生安装通过静默不自启、首次响应和重复启动单实例。
- x64 在 ARM64 Windows 兼容层完成相同安装冒烟；真实 x64 机器仍是发布门禁。
- 正常 UI E2E 写入 RFC3339 `responsesVerifiedAt` 和 `responsesProtocol=sse`，
  `SystemStatusV1` 为 `responses_verified/ready`，配置期间 ChatGPT 进程数为 0。
- `responses-404` 和 `disconnect` 均停在 `validate_router_response`；既有
  `config.toml` 哈希不变，旧验证证据撤销，状态退回
  `action_required/models_verified/ready=false`。
- 故障服务恢复为正常模式后，同一用户直接重试成功并重新进入 `ready`。

同日应用源提交 `595dbdc` 的 M4 配置事务候选证据：

- Windows ARM64/x64 目标测试均为 45 passed、0 failed、1 ignored。
- ARM64 SHA256：
  `15160a53519c6c169a556b01308ef98dc6ad5a9e5299b4296248626c7849f88f`。
- x64 SHA256：
  `8ff8178ed8161ac373342e2192a53d283e522a408a7463991aa0edf3a15ea160`。
- ARM64 原生和 x64 兼容层安装冒烟均通过；最终恢复 ARM64 原生安装。真实 x64
  机器仍是发布门禁。
- 正常 UI E2E 核对事务 manifest 的 schema、transaction ID、操作、应用版本、四个
  受管文件和已有文件 SHA256；提交后活动 journal 已删除。
- `verify-models-fail` 在 Responses 完成后让最终 `/models` 返回 503，setup 稳定停在
  `verify`；`config.toml`、运行状态、模型目录和加密 Key 的存在状态/SHA256 全部恢复，
  事务为 `rolled_back`，活动 journal 已删除。
- Router 恢复后同一用户无需清理直接重试成功；`-TestRestore` 通过并生成独立
  `operation=restore` 的已提交事务。

M3 起不再使用只实现 `/models` 的临时服务。macOS 宿主机可在 Parallels 专用网卡
地址上启动仓库内受控 Router：

```bash
python3 tools/router-test-server.py \
  --host 10.211.55.2 \
  --port 11435 \
  --model codex-assistant-test \
  --mode normal
```

Windows UI E2E 继续使用 `http://10.211.55.2:11435/v1`，并必须断言：

- 失败与成功步骤使用 DOM 中稳定的 `data-task-id`，不得依赖中文显示文本或日志字面量。
- 运行状态包含 `responsesVerifiedAt` 和 `responsesProtocol`。
- `SystemStatusV1.router.state == responses_verified` 且整体为 `ready`。
- 将测试服务切换到 `responses-404` 或 `disconnect` 后，setup 在写配置前失败；
  `config.toml` 哈希保持不变，旧验证证据撤销且整体不再 `ready`。
- 测试服务不记录探针输出、Access Key 或请求正文。

失败路径自动验收：

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\windows-e2e.ps1 `
  -RouterUrl http://10.211.55.2:11435/v1 `
  -ExpectSetupFailure
```

该模式要求测试前已有相同 Router/模型的可信 Responses 证据，用于证明重新验证失败会
撤销旧证据但不会改写现有 Codex 配置。每个失败场景后必须恢复正常 Router 并运行一次
无 `-ExpectSetupFailure` 的 E2E，证明用户可直接重试恢复。

写入后自动回滚验收：

```bash
python3 tools/router-test-server.py \
  --host 10.211.55.2 \
  --port 11435 \
  --model codex-assistant-test \
  --mode verify-models-fail
```

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\windows-e2e.ps1 `
  -RouterUrl http://10.211.55.2:11435/v1 `
  -ExpectRollback
```

该模式要求测试前已有可信 Responses 配置。脚本必须验证失败步骤为 `verify`、四个受管
文件指纹与操作前一致、事务 manifest 为 `rolled_back`、活动 journal 已删除，且
`SystemStatusV1.config.lastTransactionId` 指向本次回滚。完成后恢复 normal Router，
再运行普通 E2E 和 `-TestRestore`，证明无需人工清理即可重试且恢复本身可撤销。

标准产物：

```text
tauri-gui\artifact\CodexAssistant-<version>-windows-<arch>-setup.exe
```

## 1. 构建

```powershell
cd C:\path\to\codex-gateway-poc-installer\tauri-gui
powershell -ExecutionPolicy Bypass -File .\tools\windows-build.ps1 -Architecture auto
```

脚本输出 JSON 证据，至少包含版本、目标架构、原生架构、文件大小和 SHA256。Tauri
原始 NSIS 产物仍位于：

```text
src-tauri\target\<rust-target>\release\bundle\nsis\
```

## 2. 安装 Codex 助手

```powershell
.\CodexAssistantSetup-0.8.4-arm64.exe
```

验收：

- 安装过程无黑色控制台。
- 安装文件复制期间不得出现 Codex 助手窗口；完成页点击“完成”后只打开一个助手窗口，不自动打开 ChatGPT。
- 再次双击助手快捷方式只聚焦已有窗口，`codex-assistant.exe` 进程数保持 1。
- 开始菜单和桌面快捷方式存在。
- 首页检测项来自真实系统状态，不长期停留“等待中”。

自动冒烟验证：

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\windows-installer-smoke.ps1 `
  -InstallerPath C:\path\to\CodexAssistant-0.8.8-windows-arm64-setup.exe `
  -ExpectedSha256 24ca87048a003cb8f23e801f3c138a502b8b81ada31fe5795ab2965bbf895491
```

冒烟脚本必须由交互用户执行；检测到 SYSTEM 会直接失败。它按当前用户卸载注册表
定位安装目录，并验证候选摘要、静默安装不自启、首次启动有响应和重复启动单实例。

## 3. Ollama 准备

在 Windows VM 内运行 Ollama 时：

```powershell
curl http://127.0.0.1:11434/v1/models
```

Ollama 在 macOS 宿主机时，先获得 VM 可访问的宿主机地址：

```powershell
ipconfig
curl http://<HOST-IP>:11434/v1/models
```

Router URL 必须使用能够从 Windows VM 访问的地址。

macOS Ollama 只监听 `127.0.0.1` 时，在 macOS 仓库根目录安装常驻桥接：

```bash
python3 tauri-gui/tools/parallels-ollama-proxy.py install
```

脚本自动读取 `bridge100` 地址，只绑定 Parallels 虚拟网卡，并转发到 macOS `127.0.0.1:11434`。本机当前的 Windows Router URL 是 `http://10.211.55.2:11434/v1`，不是 macOS Wi-Fi 地址 `192.168.50.130`。桥接会在用户登录 macOS 后自动运行。

```bash
python3 tauri-gui/tools/parallels-ollama-proxy.py status
python3 tauri-gui/tools/parallels-ollama-proxy.py uninstall
```

安装桥接前必须先启动 Ollama。只有开发用 Parallels VM 需要此桥接；客户 Windows 设备应连接正式 Router 或 Windows 本机 Ollama。

在 Windows ARM64 VM 中选择“填写本机 Ollama 地址”后测试 `127.0.0.1`，必须显示 Windows ARM64/虚拟机专用提示，不能只显示通用连接失败。宿主机接口开放后，再测试宿主机地址并确认返回真实模型。

## 4. 核心流程

1. 从首页进入“服务配置”。
2. 选择“使用本机 Ollama”或填写宿主机 URL。
3. 保持“无需 Key”。
4. 点击“测试并读取模型”，必须显示真实模型列表；此前模型选择和应用按钮必须禁用。
5. 选择模型并点击“应用并验证”。
6. 观察六个阶段依次完成，其中“读取 Router 模型”和“验证实际请求”必须分开显示。
7. 完成前 ChatGPT 不得启动。
8. 完成页点击“重启并打开 ChatGPT”，确认弹窗后才允许关闭并重新启动应用。

第 5 步写入前必须生成事务快照；第 6 步最终复核失败时允许追加第七个
`rollback` 阶段。只有 rollback 为 `restored` 或原事务成功提交后才能结束操作；
`ROLLBACK_FAILED` 必须阻断 ready 并保留 manifest/活动 journal。

再次修改配置后，首页应出现“恢复上次配置”。点击后必须先快照当前状态，完整恢复上次的 `config.toml`、运行状态、模型目录和可选 DPAPI Key，并在用户确认后重启 ChatGPT。

失败场景：关闭 Ollama 后重新配置，`validate_router` 必须失败，不能显示成功页。

## 5. 文件与 Key

```powershell
Get-Content "$env:USERPROFILE\.codex\config.toml"
Get-Content "$env:LOCALAPPDATA\CodexAssistant\runtime\config.json"
```

必须存在：

```text
model_provider = "codex_assistant_router"
wire_api = "responses"
```

安装器必须保留原有 ChatGPT/Codex 用户配置，例如顶层 `notify` 和 `[desktop]`，且不得留下旧版 managed marker 或重复 provider。

使用测试 Key 再配置后：

- `router-key.secret` 不得包含可读明文。
- `config.toml` 不得包含 Key。
- Token helper 应输出原 Key，并且只在相同 Windows 用户下可解密。

```powershell
& "$env:LOCALAPPDATA\Codex 助手\codex-assistant.exe" `
  --codex-assistant-token-helper `
  "$env:LOCALAPPDATA\CodexAssistant\runtime\config.json"
```

## 6. 主题换肤

1. 先保存 ChatGPT 中未发送内容。
2. 首次安装后先打开官方 ChatGPT，完成 “Finish Windows setup” 初始化，再关闭 ChatGPT。
3. 选择“专注深色”。
4. 点击“应用并重启 ChatGPT”。
5. ChatGPT 重启并出现深色工作界面。
6. 关闭助手后主题在当前 ChatGPT 会话中仍保留。
7. 返回助手选择“自定义背景”，导入 PNG/JPEG/WebP 图片并应用。
8. ChatGPT 重启后背景图片和半透明工作界面都实际生效。
9. 重新导入一张超过 8 MB 或伪造扩展名的文件，助手必须拒绝且不得覆盖上次有效图片。
10. 返回助手选择“官方外观”。
11. ChatGPT 正常重启，注入样式消失，调试端口关闭。

ARM64 自动回归示例：

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\windows-e2e.ps1 `
  -RouterUrl http://10.211.55.2:11434/v1 `
  -ApplyAppearance custom `
  -ThemeImagePath .\src-tauri\icons\128x128.png
```

异常场景：未完成官方 Windows 初始化、占用 9335 至 9345 端口或禁用 CDP 时，助手必须显示失败并恢复官方外观，不能报告主题已应用。

## 7. 回归

- x64 Windows。
- ARM64 Windows（由 Windows/winget 自动选择架构，UI 不提供架构选项）。
- ChatGPT 已安装与未安装。
- Router 无 Key、有 Key、错误 Key、离线。
- 820×640 和默认 940×720 窗口尺寸无横向溢出、文本遮挡或按钮越界。
- 日志复制/导出不包含 Key。
- 使用官方内置 Codex 对测试 Router 发起一次 `/responses` 请求，确认返回内容且 provider 为 `codex_assistant_router`。
