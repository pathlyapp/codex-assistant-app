# Codex 助手 SPEC 推进台账

本文件记录代码仓库内可随提交审计的里程碑状态。完整需求、实施计划和追踪矩阵位于
并列目录 `../codex-assistant-spec/`。当两处状态不一致时，以外部 `SPEC.md` 的目标
行为和 `TRACEABILITY_MATRIX.md` 的最新证据为准，并在同一工作日修正本台账。

## 当前推进

| 项目 | 当前值 |
| --- | --- |
| 开始日期 | 2026-07-28 |
| 开发基线 | `main@e84c3c0` |
| 发布基线 | `v0.8.4@2588247` |
| 当前分支 | `feat/m5-guided-setup` |
| 当前 PR | [#18](https://github.com/theivanxu/codex-assistant/pull/18) |
| 当前应用版本 | `0.8.8` |
| 下一内部候选 | `0.9.0-alpha.1` |
| 当前主里程碑 | M0/M1/M3/M4 外部门禁；M5 小白用户向导 |

## 里程碑状态

| 里程碑 | 状态 | 当前结论 | 退出条件 |
| --- | --- | --- | --- |
| M0 规格与基线 | 进行中 | SPEC、追踪矩阵、在线 M0-M7 milestone/Epic 和质量基线已建立 | 真实 x64 与干净用户跨平台 E2E 完成 |
| M1 状态、错误和工作流契约 | 待验证 | PR #15 已合并；全部 Tauri command 使用 V1 envelope，状态、阶段事件、跨平台错误 fixture 和脱敏已接入 | Windows UI 错误交互复核，补齐取消与跨重启恢复边界 |
| M2 官方应用安装可靠性 | 未开始 | Windows 使用 Store/winget，结果会二次检测 | 可信来源、架构、安装结果和恢复路径可审计 |
| M3 Router 真实响应验证 | 待验证 | PR #16 已合并；`/models` 与 `/responses` 共用 Rust 客户端；Windows ARM64 正常、404、流中断和故障恢复 UI E2E 已通过 | 真实 Ollama/LM Studio、企业代理和私有 CA 兼容验证 |
| M4 配置事务与跨平台密钥 | 进行中 | PR #17 已合并；事务清单、原子写入、验证失败自动回滚、中断恢复和可逆手动恢复已通过 Windows ARM64 E2E | 文件系统故障矩阵、真实断电恢复、有效来源检测和 macOS Keychain |
| M5 小白用户交互 | 进行中 | 四步用户向导、真实前置状态、成功摘要、回滚提示、常驻复制诊断和响应式断点已接入 | Windows 成功/失败/回滚 E2E，125%/150% 缩放、键盘和屏幕阅读器验收 |
| M6 诊断与生命周期 | 未开始 | 有基础诊断和手动恢复 | 脱敏诊断包、定向修复、升级/卸载 E2E |
| M7 签名与企业交付 | 未开始 | CI 可构建三平台，Release 有 SHA256，尚未签名 | Windows 签名、macOS 公证、企业网络与发布门禁 |

状态只允许使用：`未开始`、`进行中`、`受阻`、`待验证`、`已验证`。

## 当前工作包

| 工作包 | 状态 | 分支/证据 | 下一动作 |
| --- | --- | --- | --- |
| WP-000 需求追踪 | 已验证 | M0-M7 milestone #1-#8；Epic #7-#14；PR/SPEC Issue 模板 | 后续实现 Issue 继续使用需求 ID |
| WP-001 基线测试冻结 | 进行中 | `docs/m0-baseline.zh-CN.md`；Release SHA 已归档 | 完成 Windows VM 剩余场景和 macOS 干净用户 E2E |
| WP-002 错误与事件清单 | 已验证 | `docs/error-event-inventory.zh-CN.md`；稳定映射与夹具测试 | 后续新增错误必须同步清单和测试 |
| WP-101 SystemStatusV1 | 已验证 | 组合单测；Windows E2E 断言首页、配置页和诊断页消费同一核心状态 | 0.8.x 兼容字段按版本策略后续移除 |
| WP-102 ErrorEnvelopeV1 | 待验证 | 全部 command 返回稳定 code/support ID；跨平台 fixture；统一脱敏 | Windows UI 逐类错误和推荐动作验收 |
| WP-103 WorkflowV1 | 进行中 | schema、operation ID、cancellable、合法转换与前端去重 | 未完成事务检测和取消语义转入 M4/M5 |
| WP-301 统一网络客户端 | 待验证 | `router_client.rs`；models/responses 共用代理、CA、超时和 Bearer 路径 | Windows 企业代理与私有 CA 夹具 |
| WP-302 模型发现 | 已验证 | 限制响应大小和模型数量；去重、空 ID、提交二次核对及 Windows ARM64 UI E2E | 保持跨平台回归 |
| WP-303 Responses Probe | 待验证 | 固定 `Return OK.`、16 token、SSE/JSON、完成事件、模型一致性、正文不留存；Windows ARM64 正常/404/断流 E2E | 真实 Ollama/LM Studio 兼容服务 |
| WP-304 Router 错误 UX | 待验证 | 404 和流中断停在稳定步骤 ID；配置不变、旧证据撤销、状态退回 `models_verified` | 真实服务、代理和私有 CA 错误动作验收 |
| WP-402 配置事务 v2 | 待验证 | PR #17；事务 manifest/SHA256、同目录原子替换、启动恢复、自动回滚、可逆恢复；Windows ARM64 正常/故障/重试/恢复 E2E | 权限/磁盘满/替换失败、真实进程中断、macOS 平台 E2E |
| WP-501 状态首页 | 待验证 | 首页只显示整体主动作、三项真实状态和当前服务；消费 `SystemStatusV1/recommendedAction` | 小尺寸、缩放和长错误文案实机复核 |
| WP-502 四步向导 | 待验证 | `docs/m5-guided-setup.zh-CN.md`；Windows 正常、Responses 404、verify 回滚和直接重试 E2E | 实机缩放、键盘和读屏 |
| WP-503 服务配置 | 待验证 | Router/Key/模型使用真实发现和规范化结果；原有 E2E 选择器保持稳定 | 企业代理/CA 与 macOS 实机 |
| WP-504 错误和恢复 | 待验证 | 失败唯一主动作、日志自动展开、常驻复制诊断；Responses 失败和 verify 回滚 E2E | 其它错误码推荐动作和恢复失败注入 |
| WP-505 平台体验 | 进行中 | 系统字体、浅色模式、原生标题栏、焦点返回和 720/560px 响应式断点 | 820x640、940x720、125%/150%、键盘和屏幕阅读器 |

## 证据规则

每个工作包关闭前必须至少记录：

1. 对应需求 ID 和用户可见行为。
2. 自动测试命令与结果。
3. 受影响平台构建结果。
4. 失败、重试、恢复或回滚证据。
5. 日志与诊断脱敏检查。
6. 对外安装包名称、SHA256 和签名状态（涉及发布时）。

“代码存在”不是“已验证”。没有可重复证据的能力保持为“进行中”或“待验证”。

## 推进日志

### 2026-07-28

- 将本地 `main` 快进至远端 `571db73`，纳入 ChatGPT 安装、恢复出厂和 DreamSkin
  主题库三个最新功能提交。
- 从 `main@571db73` 创建 `feat/m1-core-contracts`。
- 确认开发代码版本为 `0.8.8`，最新已发布 Release 仍为 `v0.8.4`。
- 建立 SPEC Issue 模板和 PR 需求追踪门禁。
- 开始冻结 M0 质量基线；Rust 单元测试为 23 通过、0 失败、1 个依赖本地 Ollama
  的测试忽略。
- 发现开发基线存在格式差异和 3 个 Clippy 门禁错误，纳入本分支先行修复。
- 修复上述门禁；rustfmt、Clippy `-D warnings` 和版本一致性检查通过。
- 新增 `SystemStatusV1`、`ErrorEnvelopeV1`、`StageEventV1` 及强类型阶段/状态。
- 前端整体状态只信任核心 `overall`，主动作只信任 `recommendedAction`。
- 阶段事件和完成结果使用同一 `operationId`，前端忽略过期或重复结果。
- Rust 测试增加到 29 项：28 通过，1 个本地 Ollama live test 忽略。
- macOS ARM64 release 构建成功，生成 `Codex 助手.app`。
- PD Windows 11 ARM64 原生测试通过，并成功构建 NSIS 候选安装包。
- 使用当前 Windows 用户将助手从 0.8.4 静默升级到 0.8.8；安装后未自动启动。
- 候选版界面正常渲染且状态由 V1 核心输出；重复启动保持单实例。
- Windows ARM64 候选包 SHA256：
  `24ca87048a003cb8f23e801f3c138a502b8b81ada31fe5795ab2965bbf895491`
  （源提交 `af09091`）。
- 精确候选复装后登记版本为 0.8.8，静默安装未自启；首次启动响应正常，重复启动
  仍保持 1 个进程。
- 新增可重复 Windows 构建和精确候选冒烟脚本；禁止从历史目录猜测安装包。
- Windows x64 目标测试和 NSIS 交叉构建通过；首次兼容层双启动暴露持续双实例，
  增加 Windows 原生前置互斥/聚焦兜底后回归通过。
- Windows x64 候选包 SHA256：
  `bf5673b79248e6c0c23da49c09c2acdf095c82c7ddefe918ee56047fae4ecf0f`
  （源提交 `af09091`；ARM64 x64 兼容层验收，真实 x64 机器仍待验证）。
- 截图中的程序兼容性助手告警来自第三方深信服 LSA 模块，不属于助手安装或运行错误。
- 从 GitHub `v0.8.4` Release 核对并归档三平台包、manifest 和 SHA256SUMS 的
  verified digest。
- 在 GitHub 建立 M0-M7 八个 milestone（#1-#8）和八个 P0 Epic（#7-#14），
  每个 Epic 记录需求 ID、已有证据、剩余工作与退出门禁。
- 建立 M1 PR [#15](https://github.com/theivanxu/codex-assistant/pull/15)，关联需求 ID、
  候选 SHA、失败恢复证据和 M1 milestone。
- 将 5 个外观 command 迁移到 `ErrorEnvelopeV1`，以 `appearance_*` stage 与核心状态
  隔离。
- 增加 Windows/macOS 网络错误、代理、安装可信性、配置覆盖、外观错误及敏感信息
  脱敏夹具；Rust 测试为 32 项：31 通过，1 个本地 Ollama live test 忽略。
- 最新 Windows ARM64/x64 候选对应提交 `9885c16`；SHA256 分别为
  `44af1a88df24f38b58f046e33413273b2cada5a35da70466ce040de81acb4b12` 和
  `745b5e12054f8a09ca9a201d96487279dc6f311340547234d93e3b52dcfe41ea`。
- ARM64 原生与 x64 兼容层均再次通过静默安装不自启、首次响应和单实例；VM 最终恢复
  为原生 ARM64 安装。
- Windows UI E2E 首次发现通用 envelope 覆盖 VM Ollama 专用提示；新增
  `ROUTER_VM_LOOPBACK` 等稳定错误码后重跑通过。
- 修复后 E2E 确认配置期间 ChatGPT 进程为 0、Router 模型选择和写入成功、备份入口
  可见、恢复 round-trip 完成。使用隔离测试 Router，不将其描述为生产 Router 验证。
- Windows E2E 增加跨页面状态一致性断言，首页徽标、配置页 Gateway 和诊断摘要均与
  同一次 `SystemStatusV1` 事实源一致；`WP-101` 转为已验证。
- PR [#15](https://github.com/theivanxu/codex-assistant/pull/15) 全量 CI 通过后 squash
  合并为 `main@35980c0`，远端 M1 特性分支已删除；M1 Epic 保持开放以追踪剩余门禁。
- 从 `main@35980c0` 创建 `feat/m3-responses-probe`，开始 M3。
- 新增统一 Rust Router 客户端；`/models` 与 `/responses` 使用相同 Agent、代理、CA、
  超时和 Bearer 路径。
- 配置流程扩展为六阶段；只有固定低成本 Responses 探针收到有效完成事件且返回模型
  一致，才写入配置和 RFC3339 验证时间。
- Rust 测试增加到 41 项：40 通过、0 失败、1 个本地 Ollama live test 忽略；覆盖
  SSE、JSON、流中断、模型不一致、models 成功但 responses 404、认证路径一致和正文
  不留存，并验证旧 Responses 证据只对相同 Router/模型撤销。
- 新增 `tools/router-test-server.py`，提供正常、JSON、404、流中断、失败和错误模型
  六种受控模式；本机真实 HTTP/SSE 冒烟通过。
- 建立 M3 PR [#16](https://github.com/theivanxu/codex-assistant/pull/16)，关联
  `ROUT-005` 至 `ROUT-011`、M3 milestone 和 Epic #10。
- Windows 11 ARM64 当前用户 UI E2E 已验证正常 Responses 完成后状态为
  `responses_verified/ready`，且配置期间 ChatGPT 进程数保持 0。
- 同一已验证配置切换到 `responses-404` 和 `disconnect` 后，均停在稳定步骤
  `validate_router_response`；`config.toml` SHA256 不变，旧 Responses 证据被撤销，
  状态退回 `action_required/models_verified/ready=false`；恢复正常 Router 后重试成功。
- 源提交 `c120943` 的 Windows ARM64 候选 SHA256 为
  `12ac72e27e1a12920a1173dd80cac1a860fc402648233d424aeb32e5770f2a1b`；
  原生安装、静默不自启、首次响应和单实例通过。
- 同一提交的 Windows x64 候选 SHA256 为
  `69c9ab9b885f6cff181c2e5d00dfdb717771802a23c5f01dc720062452324346`；
  x64 目标测试和 ARM64 兼容层安装冒烟通过，不替代真实 x64 机器门禁。
- PR [#16](https://github.com/theivanxu/codex-assistant/pull/16) 全量 CI 通过后 squash
  合并为 `main@e048346`；M3 Epic 保持开放追踪真实服务和企业网络门禁。
- 从 `main@e048346` 创建 `feat/m4-config-transaction`，建立 M4 PR
  [#17](https://github.com/theivanxu/codex-assistant/pull/17)。
- 新增配置事务 v2：每次 setup/restore 在修改任何受管文件前生成
  `runtime/snapshots/<transactionId>/manifest.json`，记录时间、版本、四个目标文件、
  原始存在状态、备份文件和 SHA256。
- `config.toml`、运行状态、模型目录和加密 Key 改为同目录临时文件、完整解析校验、
  `fsync` 和原子替换；Windows 使用 `MoveFileExW` 的 replace/write-through 语义。
- 最终磁盘与 Router 复核失败会追加 `rollback` 阶段并自动恢复。回滚失败保留活动
  journal、manifest 和 `ROLLBACK_FAILED`；下次状态/配置检查会恢复未完成事务。
- Rust/macOS 宿主和 Windows ARM64/x64 目标测试均为 45 通过、0 失败、1 个本地
  Ollama live test 忽略；Windows PowerShell 构建/E2E 脚本语法检查通过。
- 应用源提交 `595dbdc` 的 Windows ARM64 候选 SHA256 为
  `15160a53519c6c169a556b01308ef98dc6ad5a9e5299b4296248626c7849f88f`；
  x64 候选 SHA256 为
  `8ff8178ed8161ac373342e2192a53d283e522a408a7463991aa0edf3a15ea160`。
- ARM64 原生和 x64 兼容层安装冒烟均通过；最终 VM 恢复原生 ARM64。真实 x64
  机器仍是发布门禁。
- ARM64 正常 UI E2E 写入并提交事务，运行状态与 `SystemStatusV1` 暴露同一事务 ID。
  故障模式在 Responses 成功后让最终 `/models` 返回 503，稳定停在 `verify`；
  四个受管文件哈希全部恢复，状态为 `rolled_back`，活动 journal 已删除。
- 恢复正常 Router 后同一用户无需清理直接重试成功；手动恢复 round-trip 通过并生成
  独立 `operation=restore` 的已提交事务。配置与恢复期间均未自动启动 ChatGPT。
- PR #17 全量 CI 通过后 squash 合并为 `main@e84c3c0`，远端 M4 特性分支已删除。
- 从 `main@e84c3c0` 创建 `feat/m5-guided-setup`，开始 M5 小白用户交互收口。
- 配置页改为固定四步用户向导，环境、官方应用、服务连接和最终验证分别消费真实
  `SystemStatusV1` 与内部阶段；步骤完成态不再按序号推断。
- 成功结果增加官方应用、规范化 Router、模型、最近验证和恢复能力；失败结果只保留
  一个主重试动作，常驻复制诊断，并区分自动恢复成功与失败。
- 复制诊断增加用户目录、URL userinfo、Bearer 和 query key/token 二次脱敏；确认
  对话框关闭后恢复触发控件焦点，结果出现后主动聚焦。
- 标准 `1280x720` 浏览器渲染无横向溢出；Rust 测试 45 通过、0 失败、1 个本地
  Ollama live test 忽略。
- PowerShell 5.1 首次解析新增中文断言时受系统代码页影响；E2E 改为断言稳定英文
  `data-recovery-state/data-summary-key`，不再依赖显示文案或脚本 BOM。
- 源提交 `6495f52` 的 Windows ARM64/x64 候选 SHA256 分别为
  `6ec1628bc678e1db3816120e622d9384a6b88b828ade4e87f02093dd7ebdc315` 和
  `984c67b0a28c472b9fdbe8c0bdd0fd37de8f1a66d7c2fe038647cd1f929078d3`。
- 两 Windows 目标测试均为 45 通过、0 失败、1 忽略；ARM64 原生与 x64 兼容层
  静默安装不自启、版本、首次响应和单实例冒烟通过，VM 最终恢复 ARM64 原生候选。
- ARM64 正常 UI E2E 断言四步、真实前置状态、成功摘要和事务提交；Responses 404
  稳定停在 `validate_router_response` 且配置不变，verify 故障自动回滚并显示
  `recoveryState=restored`，四个受管文件指纹恢复。两种失败恢复 normal 后均可直接
  重试，`-TestRestore` 通过，所有配置过程中 ChatGPT 进程为 0。

## GitHub 追踪

| 里程碑 | Milestone | Epic |
| --- | --- | --- |
| M0 | [M0 规格与基线](https://github.com/theivanxu/codex-assistant/milestone/1) | [#7](https://github.com/theivanxu/codex-assistant/issues/7) |
| M1 | [M1 真实状态与错误契约](https://github.com/theivanxu/codex-assistant/milestone/2) | [#8](https://github.com/theivanxu/codex-assistant/issues/8) |
| M2 | [M2 官方应用安装可靠性](https://github.com/theivanxu/codex-assistant/milestone/3) | [#9](https://github.com/theivanxu/codex-assistant/issues/9) |
| M3 | [M3 Router 真实可用验证](https://github.com/theivanxu/codex-assistant/milestone/4) | [#10](https://github.com/theivanxu/codex-assistant/issues/10) |
| M4 | [M4 配置事务与跨平台密钥](https://github.com/theivanxu/codex-assistant/milestone/5) | [#11](https://github.com/theivanxu/codex-assistant/issues/11) |
| M5 | [M5 小白用户交互](https://github.com/theivanxu/codex-assistant/milestone/6) | [#12](https://github.com/theivanxu/codex-assistant/issues/12) |
| M6 | [M6 诊断与生命周期](https://github.com/theivanxu/codex-assistant/milestone/7) | [#13](https://github.com/theivanxu/codex-assistant/issues/13) |
| M7 | [M7 商业交付](https://github.com/theivanxu/codex-assistant/milestone/8) | [#14](https://github.com/theivanxu/codex-assistant/issues/14) |
