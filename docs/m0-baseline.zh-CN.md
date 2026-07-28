# M0 发布与测试基线

## 1. 基线定义

本项目同时维护两个不同用途的基线：

| 基线 | 标识 | 用途 |
| --- | --- | --- |
| 已发布基线 | `v0.8.4@2588247` | 客户可下载包、SHA256、安装与回退对照 |
| 开发基线 | `main@571db73`，应用版本 `0.8.8` | M0/M1 代码改动和回归测试起点 |

不得用开发版本号替代 Release 标签，也不得把未发布的 `main` 描述为客户可下载版本。

## 2. Release 产物基线

`v0.8.4` Release 工作流应生成：

- `CodexAssistant-0.8.4-windows-x64-setup.exe`
- `CodexAssistant-0.8.4-windows-arm64-setup.exe`
- `CodexAssistant-0.8.4-macos-arm64.app.zip`
- `package-manifest.json`
- SHA256 校验文件（以 Release 实际资产名称为准）

当前证据：

| 项目 | 状态 | 证据/缺口 |
| --- | --- | --- |
| Git 标签 | 已验证 | 本地存在 `v0.8.4`，指向 `2588247` |
| GitHub Release | 已验证 | 发布流程已创建 prerelease |
| 三平台构建工作流 | 已验证 | `.github/workflows/build.yml` 定义 x64、ARM64、macOS ARM64 |
| 清单由流程生成 | 已验证 | `generate-release-manifest.mjs` 在聚合任务执行 |
| Release 资产 SHA256 | 已验证 | GitHub Release 页面公开 verified digest；见下表 |
| Windows 签名 | 未实现 | M7 商业发布门禁 |
| macOS 签名与公证 | 未实现 | M7 商业发布门禁 |

`v0.8.4` GitHub Release 摘要：

| 资产 | SHA256 |
| --- | --- |
| `CodexAssistant-0.8.4-macos-arm64.app.zip` | `c544c57b4b70efb4a2c9169a3e0401dddddd83ca5588ddfb797445fa6578a0d7` |
| `CodexAssistant-0.8.4-windows-arm64-setup.exe` | `0ae488771cb7150239b28289ad76e1d947f8947b79ab1e7b5901619a323974ca` |
| `CodexAssistant-0.8.4-windows-x64-setup.exe` | `4d9091ca51f9f9fbff1ed969c86e23b24cf3f39d1b482410da4abf1baba3dee5` |
| `package-manifest.json` | `248176b0fd2511149f7bb2b26b9ab0215f874ddcb901d6010d76041751974016` |
| `SHA256SUMS.txt` | `ade1ee15447de0bf46d2d84e93ac0457099b5455baf3d5152ffe2e22716c2dc5` |

## 3. 开发基线质量结果

首次检查时间：2026-07-28，分支建立前后的代码均基于 `571db73`。

| 检查 | 首次结果 | 处理 |
| --- | --- | --- |
| Rust 格式检查 | 失败 | 最新主题库提交存在 rustfmt 差异；在当前特性分支修复 |
| Rust Clippy `-D warnings` | 失败 | 1 个平台条件死代码、1 个大错误类型、1 个无效转换 |
| Rust 单元测试 | 通过 | 23 passed，0 failed，1 ignored |
| 前端版本一致性 | 待执行 | M0 修复后执行 |
| macOS 构建 | 待执行 | M1 契约接入后执行 |
| Windows x64/ARM64 构建 | 待执行 | CI 和 PD VM 验证 |
| Rust 格式复检 | 通过 | 当前分支 rustfmt check 通过 |
| Rust Clippy 复检 | 通过 | 当前分支 `-D warnings` 通过 |
| Rust 契约测试 | 通过 | 28 passed，0 failed，1 ignored |
| 前端版本/语法 | 通过 | 版本 `0.8.8` 一致，`main.js` 语法检查通过 |
| macOS ARM64 应用构建 | 通过 | `Codex 助手.app` 成功生成；未签名、未公证 |
| Windows ARM64 单元测试 | 通过 | PD Windows 11 ARM64：28 passed，0 failed，1 ignored |
| Windows ARM64 NSIS 构建 | 通过 | `CodexAssistant-0.8.8-windows-arm64-setup.exe` |
| Windows ARM64 候选 SHA256 | 通过 | `24ca87048a003cb8f23e801f3c138a502b8b81ada31fe5795ab2965bbf895491`（`af09091`） |
| Windows ARM64 升级 | 通过 | 当前用户从 0.8.4 静默升级到 0.8.8 |
| Windows ARM64 精确候选复装 | 通过 | `af09091` 对应候选登记版本 0.8.8；静默安装后没有 `codex-assistant.exe` 进程 |
| Windows ARM64 启动与单实例 | 通过 | 精确候选首次启动 `Responding=true`；重复启动仍为 1 个进程且正常响应 |
| Windows x64 交叉测试与 NSIS | 通过 | ARM64 构建机目标 `x86_64-pc-windows-msvc`：28 passed，0 failed，1 ignored |
| Windows x64 初始单实例 | 失败 | x64 兼容层持续出现 2 个有窗口进程；定位为 Tauri 插件隐藏窗口查找失败后的降级缺口 |
| Windows x64 候选 SHA256 | 通过 | `bf5673b79248e6c0c23da49c09c2acdf095c82c7ddefe918ee56047fae4ecf0f`（`af09091`） |
| Windows x64 兼容层冒烟 | 通过 | 静默安装不自启、版本 0.8.8、首次响应；修复后重复启动保持 1 个进程 |
| M1 错误契约测试 | 通过 | 31 passed，0 failed，1 ignored；覆盖 Windows/macOS 网络、代理、安装可信性、配置覆盖、外观错误和敏感信息脱敏 |
| M1 后 macOS ARM64 应用构建 | 通过 | `Codex 助手.app` 成功生成；未签名、未公证 |
| M1 最新 Windows ARM64 候选 | 通过 | `CodexAssistant-0.8.8-windows-arm64-m1-9885c16-setup.exe`；SHA256 `44af1a88df24f38b58f046e33413273b2cada5a35da70466ce040de81acb4b12` |
| M1 最新 Windows x64 候选 | 通过 | `CodexAssistant-0.8.8-windows-x64-m1-9885c16-setup.exe`；SHA256 `745b5e12054f8a09ca9a201d96487279dc6f311340547234d93e3b52dcfe41ea` |
| M1 最新 ARM64 安装冒烟 | 通过 | 静默安装不自启、版本 0.8.8、首次响应、重复启动 1 个进程 |
| M1 最新 x64 兼容层冒烟 | 通过 | ARM64 OS 上静默安装不自启、首次响应、重复启动 1 个进程；不替代真实 x64 |
| Windows UI E2E 首次执行 | 失败 | envelope 通用文案覆盖 Windows ARM64/VM Ollama 专用提示，测试在写配置前阻断 |
| Windows UI E2E 修复后 | 通过 | `ROUTER_VM_LOOPBACK` 保留专用提示；首页/配置/诊断状态一致；配置期间 ChatGPT 进程为 0；无 Key `/models` 配置成功；备份入口可见；配置与运行状态恢复 round-trip 通过 |
| M3 Router 客户端与探针测试 | 通过 | 40 passed，0 failed，1 ignored；覆盖 SSE/JSON 完成、流中断、模型不一致、models 成功但 responses 失败、共享认证路径、输出正文不留存和旧证据精准撤销 |
| M3 受控测试 Router 冒烟 | 通过 | 仓库内测试服务的 `/models` 与 `/responses` 真实 HTTP/SSE 请求通过 |
| M3 Windows ARM64 正常 UI E2E | 通过 | `responsesProtocol=sse`；状态为 `responses_verified/ready`；配置期间 ChatGPT 进程为 0 |
| M3 Windows ARM64 404/断流 UI E2E | 通过 | 两类失败均停在 `validate_router_response`；Codex 配置哈希不变；旧证据撤销；状态退回 `models_verified/ready=false`；恢复正常后重试成功 |
| M3 Windows ARM64 候选 | 通过 | `c120943`；SHA256 `12ac72e27e1a12920a1173dd80cac1a860fc402648233d424aeb32e5770f2a1b`；原生安装、静默不自启、首次响应、单实例通过 |
| M3 Windows x64 候选 | 部分通过 | `c120943`；SHA256 `69c9ab9b885f6cff181c2e5d00dfdb717771802a23c5f01dc720062452324346`；目标测试和 ARM64 兼容层安装冒烟通过，真实 x64 待验证 |

首次失败不得从历史中删除。修复后的通过结果应作为新行附加到本节，而不是覆盖原始证据。

## 4. Windows VM 验收矩阵

以下六类场景是 WP-001 的最低门禁：

| 场景 | x64 | ARM64/PD | 必须保存的证据 |
| --- | --- | --- | --- |
| 全新安装助手 | 部分验证 | 部分验证 | x64 已在 ARM64 兼容层冒烟；ARM64 已完成升级和单实例；仍需两架构干净用户全新安装 |
| ChatGPT 已安装时跳过 | 待验证 | 部分验证 | 当前用户完整 setup 在官方 ChatGPT 已安装时通过；仍需显式 stage 断言 |
| Router 成功 | 待验证 | 部分验证 | ARM64 当前用户通过隔离 Router 完成 `/models`、`/responses`、配置写入和 `ready` 状态；仍需真实 Ollama/LM Studio 与 x64 |
| Router 连接拒绝/超时 | 待验证 | 部分验证 | ARM64 UI 验证 `ROUTER_VM_LOOPBACK`；Responses 404/断流验证配置不变和证据撤销；仍需模型阶段 timeout/拒绝 envelope |
| 配置写入与恢复 | 待验证 | 部分验证 | ARM64 当前用户写入、备份入口和完整 restore round-trip 通过；仍需干净用户 |
| 重复启动与重复提交 | 部分验证 | 部分验证 | 两架构候选重复启动只有一个进程；重复 setup 提交仍待验证 |

测试机器记录必须包含：

- Windows 版本和系统架构。
- PD/虚拟机版本与网络模式。
- 官方 ChatGPT 包名、版本、publisher 和来源。
- Router 类型、地址类别（回环/宿主机/远端）和模型。
- 助手安装包文件名和 SHA256。

不得记录真实 Access Key。

## 5. 完成判定

WP-001 只有在以下条件同时满足时才能标记“已验证”：

1. Release 清单和各资产 SHA256 已归档。（已满足）
2. Windows x64 和 ARM64 至少完成适用场景。
3. macOS ARM64 构建可安装并能检测已有官方 ChatGPT。
4. 失败结果可以按相同步骤复现。
5. 测试日志通过密钥和隐私路径检查。
