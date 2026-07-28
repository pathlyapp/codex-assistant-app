# M2 官方应用安装 Adapter

本文记录 `WP-202` 第一工作包的实现边界和验收证据。目标是把官方应用安装从业务
状态机中拆出稳定协议，并保持当前已验证行为不变；在 `DEC-011` 关闭前，不擅自启用
Web Installer、MSIX 下载或新的 fallback 顺序。

## 1. SPEC 追踪

| 需求 | 本工作包覆盖 | 尚未覆盖 |
| --- | --- | --- |
| `INST-020` | adapter 不接收用户选择的 OS/CPU，平台由编译目标决定 | 真实 x64 干净设备 |
| `INST-022` | 当前 adapter 固定 Microsoft Store Product ID 和 `msstore` source | Web 下载最终域名 allowlist |
| `INST-023` | 保留 12 分钟超时和 15 秒 heartbeat | 明确百分比、取消、断点重试 |
| `INST-024` | 安装后必须再次通过 Store 包可信检测 | 下载文件 Authenticode/架构门禁 |
| `INST-025` | 建立 `OfficialAppInstaller` trait 和策略列表 | MSIX、Web Installer adapter |
| `INST-026` | 命令成功后等待并复核可信包；异常注册立即失败 | 未安装到成功的干净设备 E2E |
| `REL-030` | 安装逻辑移出 `lib.rs`，进入独立 Rust 模块 | secret store、diagnostics 继续拆分 |
| `REL-031` | Windows 平台实现由 adapter 隔离 | 多 adapter 策略与 fallback |

## 2. 决策边界

`DEC-011` 仍为 `Proposed`。当前事实只证明 OpenAI 下载页返回 Microsoft 签名的
`Store Installer` 引导程序，不证明它在无 Store 登录、Store 禁用、企业代理或取消
场景中优于 winget。

因此当前策略只有：

```text
Windows policy
└── winget-store
    ├── Product ID: 9PLM9XGG6VKS
    ├── Source: msstore
    └── Post-condition: trusted OpenAI.Codex package
```

以下能力只记录为后续 adapter，不计入当前完成度：

- Store-signed MSIX；
- OpenAI 下载页 Web Installer；
- adapter 之间的自动 fallback；
- 自有下载器的进度、取消和断点重试。

## 3. 协议

`official_installer.rs` 定义四个核心概念：

1. `OfficialAppInstaller`：平台安装实现必须提供 kind、source、可用性和安装操作。
2. `OfficialInstallerAvailability`：返回 adapter、source、available 和可展示 detail。
3. `OfficialInstallReceipt`：返回 adapter、source、产品 ID、应用版本、检测来源和
   trusted 结果。
4. `WINDOWS_INSTALLER_POLICY`：只列出已获准启用的 adapter，顺序属于产品决策。

业务层只调用：

```text
preferred_installer_availability()
install_official_chatgpt()
```

业务状态机不再拼 winget 参数、查找 winget 路径或轮询应用包。

## 4. 安装不变量

adapter 必须满足以下失败关闭规则：

1. 已安装且可信的官方应用继续由 `official_app.rs` 判定并跳过安装。
2. winget 缺失时 preflight 失败，不显示“已准备完成”。
3. 安装命令只使用精确 Product ID `9PLM9XGG6VKS`、`-e` 和 `msstore` source。
4. 命令退出成功不等于安装成功。
5. 命令结束后最多等待 90 秒，并反复执行真实包检测。
6. 检测到 `needs_repair` 立即失败，不继续等待或进入配置。
7. 只有 `installed + trusted` 才生成成功收据。
8. 安装过程不启动 ChatGPT；启动仍由完成页用户动作控制。

## 5. 结构化结果

安装成功阶段把收据放入 `StageOutcome.details.installer`：

```json
{
  "adapter": "winget-store",
  "source": "microsoft-store",
  "productId": "9PLM9XGG6VKS",
  "appVersion": "26.721.4979.0",
  "appSource": "microsoft-store",
  "appTrusted": true
}
```

该结构用于后续进度 UI、诊断包和 adapter 取证。不得在其中记录用户账号、Key、下载
URL 查询参数或本地用户目录。

## 6. 错误契约

- winget 缺失、adapter 不可用、安装命令失败：`APP_INSTALL_FAILED`。
- Windows 退出码 `3010` 或等价 restart 文案：`APP_RESTART_REQUIRED`。
- 安装后包身份、Publisher、签名、清单或程序文件异常：
  `APP_PACKAGE_UNTRUSTED`。
- 非 Windows 自动安装：`UNSUPPORTED_PLATFORM`。

preflight 和独立“安装 ChatGPT”命令必须得到相同稳定错误码。

## 7. 自动验证

本地 macOS 主目标：

- `cargo fmt --all -- --check`；
- `cargo clippy --all-targets -- -D warnings`；
- Rust 测试 53 通过、0 失败、1 个本地 Ollama live test 忽略；
- `node --check frontend/main.js`；
- 版本一致性检查为 `0.8.8`。

Windows 11 ARM64 PD VM：

- ARM64 与 x64 目标 `cargo clippy --all-targets -- -D warnings` 均通过；
- ARM64 与 x64 目标测试均为 53 通过、0 失败、1 个 live Ollama test 忽略；
- 两目标均成功生成 NSIS 安装包；
- ARM64 原生和 x64 兼容层静默安装退出码为 0，安装过程中助手未自启；
- 两候选首次启动均响应，重复启动后进程数保持 1；
- ARM64 原生和 x64 兼容层 UI E2E 均确认：
  - 官方应用状态在首页、配置页和诊断页一致；
  - `install_chatgpt` 阶段为 `skipped`；
  - 配置事务为 `committed`；
  - 配置期间 ChatGPT 进程数为 0。

源提交 `3650c99` 的候选：

| 架构 | 文件 | 字节 | SHA256 |
| --- | --- | ---: | --- |
| ARM64 | `CodexAssistant-0.8.8-windows-arm64-m2-adapter-3650c99-setup.exe` | 5,512,709 | `314dd79dbfcee201c4c02bdad7aaa3bd934af43769d47d3742924b5ee1059e15` |
| x64 | `CodexAssistant-0.8.8-windows-x64-m2-adapter-3650c99-setup.exe` | 5,879,618 | `e50f94512725e55f4219d586ac27841f367354d5dcb644ac67b3991cba0f00c0` |

候选保存在仓库外 `../codex-assistant-dist/`，不提交 Git。VM 最终恢复 ARM64
原生候选，隔离测试 Router 已停止。

新增单测固定：

- adapter ID；
- 当前策略只启用 `winget-store`；
- winget 参数和 Store Product ID；
- 安装收据 camelCase 序列化；
- winget/preflight 错误映射和 Windows 3010 映射。

macOS 到 Windows 的直接交叉 Clippy 受本机缺少 Windows C SDK 阻断，不能作为
Windows 失败证据；上述 PD VM 原生工具链结果取代该尝试。真实 x64 硬件仍是发布
门禁，ARM64 上的 x64 兼容层不能替代它。

## 8. 退出条件

本工作包只有在以下条件全部满足后才能从“进行中”转为“待验证”：

1. [x] Windows ARM64 和 x64 目标 Clippy/测试通过。
2. [x] 已安装可信应用时不调用 winget。
3. [ ] 缺失 winget 时 preflight 在真实 VM 返回 `APP_INSTALL_FAILED`。
4. [ ] 从未安装状态完成安装并返回 adapter/source/product ID/trusted 收据。
5. [ ] 注入安装后可信检测失败并证明配置阶段不开始。
6. [x] 已安装应用跳过路径的配置期间 ChatGPT 进程保持为 0。

`WP-202` 整体关闭仍需要 `DEC-011`、Web/MSIX 签名与域名门禁、取消/重试、至少两种
adapter 的真实设备证据。
