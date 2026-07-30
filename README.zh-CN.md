# Codex 助手

Codex 助手是面向普通用户的桌面配置工具，不是 OpenAI 官方客户端，也不重新分发或修改 ChatGPT。它负责：

- 检测官方 ChatGPT，缺失时通过 Microsoft Store / winget 引导安装。
- 配置 Codex 模式使用的 OpenAI-compatible Router。
- 安全保存可选 Access Key，并为 Codex 提供稳定的 auth helper。
- 真实检查 ChatGPT、`~/.codex/config.toml` 和 Router 状态。
- 提供可选、可恢复的 ChatGPT 本地外观主题。

扫码购买、套餐选择、支付轮询、激活码和 Mock device token 已从产品与代码主线删除。

## 产品流程

用户下载并安装：

```text
CodexAssistantSetup-0.8.4-arm64.exe
```

安装后打开 Codex 助手：

```text
读取真实状态（首页只显示 ChatGPT、Router、Codex 配置）
-> 填写 Router URL / Key
-> 获取 /v1/models
-> 选择模型
-> 检测或安装官方 ChatGPT
-> 写入 Codex 配置
-> 再次复核
-> 用户确认后重启并打开 ChatGPT
```

普通 ChatGPT 对话、账号、订阅和聊天记录不受 Router 设置影响。Router 只作用于 Codex 模式及读取同一 `$CODEX_HOME` 配置的 Codex CLI/IDE 入口。

## 当前实现

主工程：`tauri-gui/`

- Tauri 2 提供原生窗口和 NSIS 安装包。
- Rust 负责 Appx 检测、winget、HTTPS Router、配置写入、DPAPI、诊断、启动和主题 CDP。
- 本地 Web UI 负责状态首页、Router 向导、结构化进度、诊断和个性化交互。
- 配置阶段为 `preflight -> install_chatgpt -> validate_router -> configure_codex -> verify`。
- 任一真实检查失败都会显示失败，不允许“只写配置”伪装成完整成功。
- `config.toml` 使用结构化 TOML 更新，只接管助手自己的 provider，并保留 ChatGPT 的 `notify`、`desktop` 等其它设置。
- Router 测试成功前模型选择和应用按钮保持禁用，模型列表不使用占位数据。
- 每次写入前保留配置备份，首页和诊断页可一键恢复最近一次配置。
- 配置过程不启动 ChatGPT；成功页由用户确认后执行可靠重启，以加载新配置。
- NSIS 安装阶段不提前启动助手；用户进入完成页并点击完成后才启动。助手使用单实例运行，重复双击只聚焦已有窗口。
- 主导航提供“主题换肤”，支持官方外观、专注深色和用户自选 PNG/JPEG/WebP 背景。
- 自定义图片最大 8 MB，导入时验证真实格式、尺寸和像素上限，并保存在助手管理的本地目录。
- 大图通过会话内 Blob URL 加载，避免 base64 CSS 长度限制；背景只铺一次，并为侧栏、菜单和输入区提供独立可读性层。

## 默认 Ollama 演示

默认 Router：

```text
http://127.0.0.1:11434/v1
```

先确认 Ollama 返回模型：

```powershell
curl http://127.0.0.1:11434/v1/models
```

若 Ollama 在 Parallels 宿主机运行，需要将 Router URL 改为 Windows VM 可访问的宿主机地址，不能在 VM 内继续使用 `127.0.0.1`。

当前 ARM64 验收环境中，winget 能发现 `Ollama.Ollama 0.32.1`，但没有适用的 Windows ARM64 安装器。因此本机地址测试会显示架构与虚拟机提示。宿主机 Ollama 必须监听 Parallels 虚拟网卡或通过仅绑定该网卡的端口转发开放，再填写类似：

```text
http://10.211.55.2:11434/v1
```

不要为虚拟机填写 macOS 的 `127.0.0.1`；也不建议无防火墙限制地把 Ollama 暴露到物理局域网。

本仓库提供仅用于 Parallels 演示的虚拟网卡转发工具。保持 macOS Ollama 正常运行，再执行：

```bash
python3 tauri-gui/tools/parallels-ollama-proxy.py install
```

脚本会自动读取 `bridge100` 并安装当前用户 LaunchAgent。然后在 Windows 助手中填写脚本输出的地址，本机为 `http://10.211.55.2:11434/v1`。不要填写 macOS Wi-Fi 地址 `192.168.50.130`，因为 Ollama 并未监听该接口。

```bash
python3 tauri-gui/tools/parallels-ollama-proxy.py status
python3 tauri-gui/tools/parallels-ollama-proxy.py uninstall
```

## 安全边界

- Windows Key 使用当前用户 DPAPI 加密保存在 `%LOCALAPPDATA%\CodexAssistant\runtime`。
- `config.toml` 不保存 Key 明文，只调用安装后的 `codex-assistant.exe --codex-assistant-token-helper`。
- 每次写配置前生成同一时间戳的完整快照，覆盖 `config.toml`、运行状态、模型目录和可选 DPAPI Key。
- 主题调试端口只绑定/连接 `127.0.0.1`，只接受 `app://` 页面目标。
- Windows 主题模式会验证端口所有进程是已验证的软件包内 `ChatGPT.exe`。
- 主题注入不修改 ChatGPT 安装文件；恢复官方外观会关闭调试会话并正常重启。
- 自定义背景只读取用户主动选择的本地图片，不上传到 Router 或第三方服务。
- 首次使用必须先在官方 ChatGPT 中完成 Windows 初始化；未完成时助手会阻止主题注入并保持官方外观。

## 主题换肤

从侧边栏进入“主题换肤”，选择“专注深色”或导入一张自己的背景，然后点击“应用并重启 ChatGPT”。主题只作用于助手以回环 CDP 启动的当前 ChatGPT 会话；选择“官方外观”即可恢复。

该功能参考 [Codex-Dream-Skin](https://github.com/Fei-Away/Codex-Dream-Skin/tree/main) 的本地会话注入思路，但由本项目独立实现图片校验、持久化、软件包身份校验和 UI。参考项目中另有授权限制的人物素材不会随本产品分发。

## 构建

macOS 本机编译验证：

```bash
cd tauri-gui
npm ci
npm run build:mac
```

Windows 可分发包必须在 Windows VM/CI 构建：

```powershell
cd .\tauri-gui
npm ci
npm run build:windows
```

Windows x64 与 ARM64 必须分别构建。下载页或更新服务根据设备架构返回正确安装包，产品 UI 不要求普通用户选择架构。构建产物不进入 Git，通过 GitHub Actions Artifacts 和 Releases 分发。

## 发布

三个版本字段必须保持一致：

- `tauri-gui/package.json`
- `tauri-gui/src-tauri/Cargo.toml`
- `tauri-gui/src-tauri/tauri.conf.json`

推送 `v*` 标签会触发发布工作流。工作流分别构建 Windows x64、Windows ARM64 和 macOS ARM64，自动生成 schema v2 `package-manifest.json`、`SHA256SUMS.txt` 和固定格式发布说明，并将安装包上传为仅供内部测试的 GitHub Pre-release。当前清单固定 `customerReady=false`；在代码签名、公证和签名清单验证完成前，客户渠道会被硬阻断。源码仓库不手工维护发布清单和二进制文件。

```bash
git tag v0.8.4
git push origin v0.8.4
```

正式面向客户发布前，还需要接入 Windows 和 macOS 代码签名；签名证书只允许保存在 GitHub Secrets 或受控签名服务中。

助手更新客户端、签名产物构建、本地模拟服务和剩余商业门禁见
`docs/wp-604a-updater-client.zh-CN.md`。普通构建没有内置更新 endpoint 和公钥时会
明确显示“未启用”，不会连接临时服务或自动安装。当前能力仍属于内部测试，不改变
`customerReady=false` 门禁。

测试要求见 `docs/windows-vm-test-plan.zh-CN.md`。

## 写入位置

```text
%LOCALAPPDATA%\CodexAssistant\runtime\config.json
%LOCALAPPDATA%\CodexAssistant\runtime\models.json
%LOCALAPPDATA%\CodexAssistant\runtime\router-key.secret
%LOCALAPPDATA%\CodexAssistant\runtime\appearance.json
%LOCALAPPDATA%\CodexAssistant\runtime\themes\custom-theme.json
%LOCALAPPDATA%\CodexAssistant\runtime\themes\custom-background-*.png|jpg|webp
%USERPROFILE%\.codex\config.toml
```

官方 ChatGPT 安装命令：

```powershell
winget install --id 9PLM9XGG6VKS -e -s msstore `
  --accept-source-agreements `
  --accept-package-agreements
```
