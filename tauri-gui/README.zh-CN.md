# Codex 助手 Tauri GUI

当前产品主线是 Tauri 2 + Rust + 本地 Web UI。用户安装一次「Codex 助手」，以后通过它检测/安装官方 ChatGPT、配置 Codex Router、诊断环境和管理可选外观。

## 当前流程

```text
preflight
-> install_chatgpt
-> validate_router
-> configure_codex
-> verify
```

- 不包含扫码购买、套餐、支付、激活码或 Mock Token。
- Router UI 只要求 URL、可选 Access Key 和模型。
- 默认使用本机 Ollama：`http://127.0.0.1:11434/v1`。
- Windows ARM64/虚拟机环境会对本机 Ollama 失败给出架构与宿主机地址提示，不会暗示预设按钮负责安装 Ollama。
- `/v1/models` 必须真实可访问，配置和模型必须复核通过才显示成功。
- Windows Access Key 使用当前用户 DPAPI 加密，`config.toml` 只保存 auth helper 命令。
- Codex provider 固定使用 Responses API。
- Router 连接测试成功前不允许选择模型或应用配置。
- 配置期间不启动 ChatGPT；完成页由用户确认后重启并打开，以加载新配置。
- 写入前自动备份 `config.toml`，首页和诊断页支持恢复最近一次配置。
- ChatGPT 个性化是独立可选模块，失败不影响 Router 配置。
- 配置使用结构化 TOML 更新，保留 ChatGPT 的其它用户设置。
- 应用非官方外观前，用户必须先完成官方 ChatGPT 的一次性 Windows 初始化。
- NSIS 只在完成页启动助手；助手启用 Tauri 官方单实例插件，重复启动会聚焦已有窗口。

Parallels Windows VM 访问 macOS 本机 Ollama 时，先在 macOS 安装仅绑定虚拟网卡的常驻桥接：

```bash
python3 tools/parallels-ollama-proxy.py install
python3 tools/parallels-ollama-proxy.py status
```

在 Windows 中使用脚本输出的 Router URL，本机为 `http://10.211.55.2:11434/v1`。

## 开发

```bash
cd tauri-gui
npm install
npm run dev
```

## 测试与打包

```bash
cd src-tauri
cargo test
cd ..
npm run build:mac
```

Windows 分发包必须在 Windows VM 或 Windows CI 中执行 `npm run build:windows`，并按 `../docs/windows-vm-test-plan.zh-CN.md` 做端到端验证。

助手更新签名构建使用 `npm run build:update:mac` 或
`npm run build:update:windows`；本地回环 PoC 使用对应的 `build:update:mock:*`。
环境变量、签名密钥和模拟服务步骤见
`../docs/wp-604a-updater-client.zh-CN.md`。未配置更新信任根的普通构建不会检查或
下载更新。

## 状态文件

```text
%LOCALAPPDATA%\CodexAssistant\runtime\config.json
%LOCALAPPDATA%\CodexAssistant\runtime\models.json
%LOCALAPPDATA%\CodexAssistant\runtime\router-key.secret
%LOCALAPPDATA%\CodexAssistant\runtime\appearance.json
%USERPROFILE%\.codex\config.toml
```

安装后的稳定 Token Helper 入口：

```text
codex-assistant.exe --codex-assistant-token-helper <config.json>
```
