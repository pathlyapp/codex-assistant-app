# M2 官方应用可信检测

## 1. 目标

本工作包落实 `WP-201` 的第一阶段，解决“检测到一个同名程序就认为 ChatGPT 已安装”的
误判风险。助手必须把以下状态分开：

- `not_installed`：当前用户没有注册统一版 ChatGPT/Codex 官方包。
- `installed`：包身份、签名来源、注册状态、版本和启动入口均可验证。
- `needs_repair`：包存在，但身份、签名来源、注册状态、清单或程序文件异常。
- `unsupported`：当前平台不在支持范围。

只有 `installed + trusted=true` 可以参与整体 `ready` 判定和启动 ChatGPT。

## 2. 官方事实基线

核对日期：2026-07-28。

OpenAI 当前下载页说明，新版 ChatGPT 桌面应用同时包含 Chat、Work 和 Codex，并把 Windows
下载指向 Microsoft 的 `9PLM9XGG6VKS` 产品；旧版 ChatGPT Classic 使用独立产品
`9NT1R1C2HH7J`。本产品面向新版统一应用，不把 Classic 当作 Codex 就绪证据。

官方参考：

- <https://chatgpt.com/download/>
- <https://help.openai.com/en/articles/20001276/>
- <https://help.openai.com/en/articles/9982051-using-the-chatgpt-windows-app>

Windows PD ARM64 当前用户实测统一版包证据：

| 字段 | 可信值 |
| --- | --- |
| Store Product ID | `9PLM9XGG6VKS` |
| Package Name | `OpenAI.Codex` |
| Package Family | `OpenAI.Codex_2p2nqsd0c76g0` |
| Publisher | `CN=50BDFD77-8903-4850-9FFE-6E8522F64D5B` |
| Publisher ID | `2p2nqsd0c76g0` |
| Signature Kind | `Store` |
| Package Status | `Ok` |
| App ID | `App` |
| Executable | `app/ChatGPT.exe` |
| 实测版本 | `26.721.4979.0` |

版本号不是固定 allowlist，检测只验证其结构；包身份和发布者才是稳定信任边界。

## 3. Windows 检测规则

助手在当前交互用户上下文调用 Appx API，并只查询 `OpenAI.Codex`。不得使用宿主机进程、
窗口标题、开始菜单显示名或模糊快捷方式作为安装证据。

进入 `installed` 必须同时满足：

1. Package Name、Package Family、Publisher 和 Publisher ID 与可信值完全一致。
2. `SignatureKind=Store`。
3. `Status=Ok`。
4. 架构为 `X64` 或 `Arm64`，由 Store 和 Windows 选择兼容包，UI 不暴露架构选择。
5. 版本由 3 至 4 个数字段组成。
6. App ID 是受限安全标识。
7. 清单中的相对程序路径无绝对路径、盘符或 `..`，且目标为 `ChatGPT.exe`。
8. 注册后的完整程序文件真实存在。

任何一项不满足都进入 `needs_repair`，不得直接启动，不得在原有配置流程中静默覆盖。
推荐动作固定为“查看修复方案”，诊断区显示状态、可信性和来源，但不暴露用户密钥。

## 4. macOS 当前边界

当前实现验证新版统一应用的：

- Bundle ID：`com.openai.codex`。
- Team ID：`2DC432GLL2`。
- 版本。
- `CFBundleExecutable` 对应的真实程序文件。

严格的深层签名完整性和 notarization 门禁尚未关闭。当前开发机上的现有
`/Applications/ChatGPT.app` 在 `codesign --verify --deep --strict` 下返回签名异常，
需要用全新官方下载样本复核是本机安装状态、更新过程还是产品包行为。在该事实关闭前，
不得把严格校验直接变成所有 macOS 用户的阻断条件。

## 5. 核心契约

`SystemStatusV1.app` 由 Rust 核心直接输出：

```json
{
  "state": "installed",
  "installed": true,
  "name": "ChatGPT",
  "version": "26.721.4979.0",
  "detail": "Microsoft Store 官方应用 · 26.721.4979.0",
  "trusted": true,
  "source": "microsoft-store"
}
```

兼容字段 `appInstalled` 暂时保留，但前端必须使用 `app.state`、`app.trusted` 和
`recommendedAction` 决定异常交互。`needs_repair` 的整体状态为 `blocked`。

## 6. 测试与验收

自动测试覆盖：

- 正确 Store 包进入 `installed`。
- Publisher 不匹配进入 `needs_repair`。
- 注册状态、App ID 或程序文件异常进入 `needs_repair`。
- 清单使用目录穿越路径时拒绝。
- `SystemStatusV1` 不允许不可信包进入 `ready`。
- 前端把 `needs_repair` 显示为错误态并只引导到诊断，不继续下载或配置。

Windows 验收必须在 `--current-user` 上下文执行：

1. 读取已安装统一版应用，断言 `installed + trusted + microsoft-store + version`。
2. 已安装健康包不触发 `winget`，安装阶段记录为 `skipped`。
3. 安装和配置期间 ChatGPT 进程数保持为 0。
4. ARM64 原生和 x64 目标测试均通过。

本工作包不修改官方应用安装来源和优先级；`WP-202`、`DEC-011` 仍需单独评审 Web
Installer、Store/winget、签名验证、取消和 fallback。
