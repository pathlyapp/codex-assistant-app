# M8 Codex 账号导入工作包

## 1. 目标与范围

本工作包落实 `WP-801` 至 `WP-804`，为侧边栏新增“账号与数据”页面。页面定位是
“账号配额 + 本地数据一览”，而不是单纯导入账号：

- 云端独有、本地拿不到的信息：账号身份（邮箱、套餐）与配额用量——必须联网拉取，
  这是“导入”的真正价值。
- 本地已有的信息：会话、任务与存储占用——本来就在 `~/.codex`，无需“导入”，
  页面只做概览，不读取会话内容。

用户目标只有三个：

1. 确认当前 Codex CLI 登录的是哪个 ChatGPT 账号（邮箱、套餐）、配额还剩多少。
2. 一键把云端独有的账号信息与实时用量导入本地快照，随时回看。
3. 看到本机 Codex 数据概览（会话数、最近会话、存储占用），登录失效时知道下一步
   （运行 `codex login`）。

本轮只做“读取 + 导入快照”，不做：

- 不刷新 OAuth token（access_token 过期时提示用户重新 `codex login`，刷新流程留待后续工作包）。
- 不修改 `~/.codex/auth.json` 的任何字节，不写回、不删除。
- 不把账号快照纳入诊断包或复制诊断摘要（邮箱属于隐私信息，沿用 M5/M6 的脱敏边界）。
- 不实现多账号切换、账号退出登录或在线登录流程。

## 2. 事实源

- 登录状态只来自 `~/.codex/auth.json`（尊重 `CODEX_HOME` 环境变量），不根据进程或 UI 猜测。
- 账号邮箱、姓名、套餐只来自 `tokens.id_token` 的 JWT claims（本地解码，不验签、不联网）。
- 配额用量只来自真实 `GET https://chatgpt.com/backend-api/wham/usage`，请求头与 Codex CLI 一致
  （`Authorization: Bearer <access_token>`、`chatgpt-account-id`）。
- 已导入的历史快照只来自助手数据目录 `runtime/account-snapshot.json`，由原子写入产生。
- 前端账号页只消费 `CodexAccountStatusV1`，不自行推导登录状态。

## 3. 登录状态判定

| loginState | 判定条件 | 页面行为 |
| --- | --- | --- |
| `chatgpt` | `auth.json` 存在且含 `tokens.id_token` / `tokens.access_token` | 展示账号信息，可导入 |
| `api_key` | `auth.json` 存在、`OPENAI_API_KEY` 非空且无 `tokens` | 说明 API Key 模式无账号信息可导入 |
| `not_logged_in` | `auth.json` 不存在或两者皆空 | 引导运行 `codex login` |

`auth.json` 解析失败视为错误态（stage=`account_status`），给出文件路径与修复建议，
不回退到 `not_logged_in`，避免掩盖损坏的登录文件。

## 4. 接口契约

新增两个 Tauri command，错误统一走 `ErrorEnvelopeV1`：

| Command | stage | 说明 |
| --- | --- | --- |
| `get_codex_account_status` | `account_status` | 读取登录状态 + id_token 档案 + 本地快照，不联网 |
| `import_codex_account` | `account_import` | 联网拉取用量，原子写入快照，返回最新状态 |

`CodexAccountStatusV1`（camelCase）：

- `schemaVersion` / `loginState` / `authMode` / `authPath` / `lastRefresh` / `message`
- `profile`：`email`、`name`、`planType`、`accountId`、`tokenExpiresAt`（均可空）
- `snapshot`：`importedAt`、账号字段、`usage`（`allowed`、`limitReached`、
  `primaryWindow`/`secondaryWindow` 的 `usedPercent`、`limitWindowSeconds`、
  `resetAt`，`credits` 的 `hasCredits`/`unlimited`/`balance`）
- `localData`：`sessionCount`、`archivedSessionCount`、`latestSessionAt`、
  `recentThreads`（来自 `session_index.jsonl`，按 `updatedAt` 倒序取前 3）、
  `totalBytes`/`sessionsBytes`/`logsBytes`、`codexHome`；纯文件元数据统计，
  不跟随符号链接，遍历条目有上限

导入语义：

- 档案快照（邮箱/套餐）必然写入；用量拉取失败时快照照常保存，`message` 说明
  “账号信息已导入，用量获取失败：<原因>”，不让网络故障否决整次导入。
- 401/403 判定为登录过期，文案引导 `codex login`；401 时不写快照（避免用过期凭证
  覆盖上一份好快照），直接返回错误。
- 用量接口地址可由 `CODEX_ASSISTANT_ACCOUNT_API_BASE` 覆盖，供测试与代理环境使用。

## 5. 安全与隐私

- access_token、id_token、refresh_token 一律不写入快照、不进入日志、不进入前端；
  错误文本沿用统一 redactor（`Bearer ` 等标记脱敏）。
- 快照文件只含展示安全字段（邮箱、姓名、套餐、用量百分比、时间）。
- 读取 `auth.json` 不跟随符号链接逃出 `CODEX_HOME`；文件上限 64 KiB，超出拒绝解析。
- 账号页与快照不纳入 `export_diagnostics` 诊断包；诊断包隐私边界不变。
- HTTP 仅 HTTPS（测试覆盖地址除外），超时 15 秒，响应上限 1 MiB。

## 6. 页面结构（侧边栏第二位）

“账号与数据”导航项位于“首页”之后、“模型服务”之前：

- 顶部状态卡：登录状态徽标 + 一句话结论 + 主动作“导入到本地”。
- 账号信息区：邮箱、姓名、套餐、账号 ID、登录刷新时间。
- 用量区：主窗口进度条（已用百分比 + 重置时间）、可选次级窗口、额度信息。
- 本地数据区：会话总数、已归档、最近活跃、存储占用、最近 3 个会话标题。
- 快照区：上次导入时间与快照保存位置。
- 未登录/API Key 两种空态，各自给出下一步说明。

## 7. 自动验收

`cargo test`（`account` 模块）必须断言：

1. id_token JWT 解码提取邮箱/套餐，畸形 token 给出可读错误且不 panic。
2. 三种 loginState 判定与损坏 `auth.json` 的错误路径。
3. 用量响应解析覆盖：完整窗口、无次级窗口、`rate_limit` 为空、401 映射为重新登录。
4. 导入经本地 HTTP 服务 round-trip：请求头带 `Authorization` 与 `chatgpt-account-id`，
   快照落盘且全文不含任何 token 原文。
5. 快照写入-读取 round-trip；损坏快照被忽略而非报错。
6. 本地数据统计：嵌套会话目录计数、归档计数、字节汇总、会话索引解析（跳过坏行、
   倒序取前 3）、目录缺失时安全返回零值。

人工验收（macOS 实机）：

1. 已登录 ChatGPT 账号时导入成功，页面展示邮箱、套餐与用量进度。
2. 用量接口不可达时档案仍导入，页面提示用量获取失败。
3. 退出登录（移走 `auth.json`）后页面显示空态引导。

## 8. 当前状态

| 项目 | 状态 | 证据/缺口 |
| --- | --- | --- |
| 登录状态三态判定 | 待验证 | `account.rs` 单测覆盖 chatgpt/api_key/not_logged_in、损坏/超限/符号链接逃逸；缺 Windows 实机 |
| 账号信息与用量导入 | 待验证 | 本机真实账号（Plus）live 导入通过：`cargo test -- --ignored live_import`（需代理环境变量）；401 与 5xx 路径单测覆盖 |
| 账号页 UI | 待验证 | `node --test tools/account-view.test.mjs` 6/6 通过（登录态渲染、本地数据概览/零态、导入触发、失败保留、双空态）；820x640/940x720 实机渲染未验收 |
| 快照安全 | 待验证 | 单测断言快照全文不含 access/id/refresh token 与 Bearer；诊断包未纳入账号快照 |

2026-08-02 首轮证据：`cargo test` 94 通过 / 2 忽略（live 用例与既有 Ollama live 用例）；
`cargo clippy --all-targets -- -D warnings` 与 `cargo fmt --check` 干净；ureq 启用
`socks-proxy`/`proxy-from-env` 以支持本机代理环境。

2026-08-02 二轮：按体验反馈把页面从“账号导入”重定位为“账号与数据”，新增
`WP-804` 本地数据概览；`cargo test` 97 通过，前端逻辑测试 6/6。

## 9. 未来工作

- access_token 过期时用 refresh_token 静默刷新（OAuth refresh 流程 + 回写保护）。
- macOS Keychain / Windows DPAPI 保护快照文件。
- 账号页纳入引导流程（首次配置前确认账号套餐）。
