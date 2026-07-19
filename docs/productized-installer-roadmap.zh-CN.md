# Codex 助手产品方案

## 定位

Codex 助手是官方 ChatGPT/Codex 的安装引导、Router 配置、诊断和个性化工具。它不是官方客户端，不离线分发 ChatGPT，不代理普通 ChatGPT 会话。

当前产品不包含购买和激活。Router URL 与 Key 由用户或交付人员填写。

## 信息架构

### 状态

首屏只回答三个问题：

- ChatGPT 是否安装。
- Codex Router 配置是否有效。
- Router 当前是否可连接。

主按钮根据状态显示“开始配置”或“打开 ChatGPT”。

### Router 配置

- Router URL。
- 无需 Key 开关。
- 遮罩的 Access Key。
- 从 `/v1/models` 获取的模型选择。
- 清晰的测试、应用、进度和失败状态。

不暴露操作系统、CPU 架构、wire API 或 provider ID 给普通用户。

### 个性化

- 官方外观。
- 专注深色。
- 后续增加自定义背景与主题商店。

个性化是可选模块，必须具备兼容性门禁、一键恢复和失败回滚，不能影响 Router。

### 诊断

- 真实系统检查结果。
- 配置路径。
- 一键复制诊断信息。
- 运行日志复制/导出并脱敏。

## 服务边界

ChatGPT 与 Codex 共用桌面壳，但 Router 只作用于 Codex：

- Chat/Work 账号、订阅、记录由 ChatGPT 云端管理。
- Codex provider、model、MCP、权限和项目行为来自 `$CODEX_HOME`。
- 安装器只修改用户级 `~/.codex/config.toml` 的 managed block。
- 不修改 `chatgpt_base_url`。

## 里程碑

### M1：真实核心流程

- 删除支付/激活。
- 官方 ChatGPT 检测与 winget 安装。
- HTTPS Router、模型发现、真实复核。
- Windows DPAPI 与 Rust auth helper。
- 状态首页和清晰向导。

### M2：可靠个性化

- 回环 CDP 和官方包身份校验。
- 官方/专注主题切换。
- 自动恢复官方启动方式。
- ChatGPT 版本兼容性矩阵。

### M3：交付工程化

- Windows x64/ARM64 CI。
- 代码签名、自动更新和崩溃报告。
- 企业代理、证书、Intune 部署说明。
- 安装/配置/主题的遥测必须显式征得用户同意且不采集 Key。

### M4：平台服务

- 用户登录后下发 Router 配置（不是支付）。
- Key 轮换、撤销和设备管理。
- Router 健康、模型目录和版本策略。
- 客服诊断包与远程配置策略。

## 发布门禁

- 所有核心阶段具有自动测试和 Windows VM 实测证据。
- Router 断开不能显示成功。
- 官方 ChatGPT 在配置完成前不启动。
- Key 不出现在配置、日志或诊断文本。
- 主题失败会恢复官方启动，且不改变 Codex Router。
- 最小窗口无重叠和横向溢出。
