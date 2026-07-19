# Windows VM 验收计划

目标产物：

```text
CodexAssistantSetup-0.8.4-arm64.exe
```

## 1. 构建

```powershell
cd C:\path\to\codex-gateway-poc-installer\tauri-gui
npm ci
cargo test --manifest-path .\src-tauri\Cargo.toml
npm run build:windows
```

NSIS 产物位于：

```text
src-tauri\target\release\bundle\nsis\
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
powershell -ExecutionPolicy Bypass -File .\tools\windows-installer-smoke.ps1
```

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
6. 观察五个阶段依次完成。
7. 完成前 ChatGPT 不得启动。
8. 完成页点击“重启并打开 ChatGPT”，确认弹窗后才允许关闭并重新启动应用。

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
