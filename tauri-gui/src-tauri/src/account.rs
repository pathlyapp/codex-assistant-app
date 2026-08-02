use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::config_transaction::atomic_write;
use crate::contracts::SCHEMA_VERSION_V1;

pub const DEFAULT_ACCOUNT_API_BASE: &str = "https://chatgpt.com/backend-api";
const ACCOUNT_SNAPSHOT_FILE: &str = "account-snapshot.json";
const MAX_AUTH_FILE_BYTES: u64 = 64 * 1024;
const MAX_USAGE_RESPONSE_BYTES: u64 = 1024 * 1024;
const USAGE_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAccountStatusV1 {
    pub schema_version: u8,
    pub login_state: String,
    pub auth_mode: Option<String>,
    pub auth_path: String,
    pub last_refresh: Option<String>,
    pub profile: Option<AccountProfileV1>,
    pub snapshot: Option<AccountSnapshotV1>,
    pub snapshot_path: String,
    pub local_data: LocalDataV1,
    pub message: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalDataV1 {
    pub session_count: u64,
    pub archived_session_count: u64,
    pub latest_session_at: Option<String>,
    pub recent_threads: Vec<RecentThreadV1>,
    pub total_bytes: u64,
    pub sessions_bytes: u64,
    pub logs_bytes: u64,
    pub codex_home: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentThreadV1 {
    pub id: String,
    pub name: String,
    pub updated_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountProfileV1 {
    pub email: Option<String>,
    pub name: Option<String>,
    pub plan_type: Option<String>,
    pub account_id: Option<String>,
    pub token_expires_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSnapshotV1 {
    pub schema_version: u8,
    pub imported_at: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub plan_type: Option<String>,
    pub account_id: Option<String>,
    #[serde(default)]
    pub usage: Option<AccountUsageV1>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUsageV1 {
    pub fetched_at: String,
    pub allowed: bool,
    pub limit_reached: bool,
    pub primary_window: Option<UsageWindowV1>,
    pub secondary_window: Option<UsageWindowV1>,
    pub credits: Option<UsageCreditsV1>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindowV1 {
    pub used_percent: u8,
    pub limit_window_seconds: u64,
    pub reset_after_seconds: u64,
    pub reset_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageCreditsV1 {
    pub has_credits: bool,
    pub unlimited: bool,
    pub balance: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexAuthFile {
    #[serde(rename = "OPENAI_API_KEY")]
    openai_api_key: Option<String>,
    #[serde(default)]
    auth_mode: Option<String>,
    #[serde(default)]
    tokens: Option<CodexAuthTokens>,
    #[serde(default)]
    last_refresh: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexAuthTokens {
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    account_id: Option<String>,
    // refresh_token 从不读取：助手只消费 access_token，不实现刷新流程
}

#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    exp: Option<i64>,
    #[serde(rename = "https://api.openai.com/auth", default)]
    auth: Option<IdTokenAuthClaims>,
}

#[derive(Debug, Deserialize)]
struct IdTokenAuthClaims {
    #[serde(default)]
    chatgpt_account_id: Option<String>,
    #[serde(default)]
    chatgpt_plan_type: Option<String>,
}

enum AuthState {
    ChatGpt {
        auth_mode: Option<String>,
        last_refresh: Option<String>,
        profile: AccountProfileV1,
        access_token: String,
        account_id: Option<String>,
    },
    ApiKey {
        auth_mode: Option<String>,
    },
    NotLoggedIn,
}

pub fn snapshot_path(install_root: &Path) -> PathBuf {
    install_root.join(ACCOUNT_SNAPSHOT_FILE)
}

pub fn collect_account_status(
    codex_home: &Path,
    install_root: &Path,
) -> Result<CodexAccountStatusV1, String> {
    let auth_path = codex_home.join("auth.json");
    let snapshot = read_snapshot(install_root);
    let snapshot_path = snapshot_path(install_root).to_string_lossy().to_string();
    let local_data = collect_local_data(codex_home);
    match read_auth_state(&auth_path)? {
        AuthState::ChatGpt {
            auth_mode,
            last_refresh,
            profile,
            ..
        } => {
            let message = match (&snapshot, &profile.email) {
                (Some(snapshot), _) => format!("上次导入于 {}", snapshot.imported_at),
                (None, Some(email)) => format!("已登录 {email}，可导入到本地"),
                (None, None) => "已登录 ChatGPT 账号，可导入到本地".to_string(),
            };
            Ok(CodexAccountStatusV1 {
                schema_version: SCHEMA_VERSION_V1,
                login_state: "chatgpt".to_string(),
                auth_mode,
                auth_path: auth_path.to_string_lossy().to_string(),
                last_refresh,
                profile: Some(profile),
                snapshot,
                snapshot_path,
                local_data,
                message,
            })
        }
        AuthState::ApiKey { auth_mode } => Ok(CodexAccountStatusV1 {
            schema_version: SCHEMA_VERSION_V1,
            login_state: "api_key".to_string(),
            auth_mode,
            auth_path: auth_path.to_string_lossy().to_string(),
            last_refresh: None,
            profile: None,
            snapshot,
            snapshot_path,
            local_data,
            message: "当前是 API Key 登录方式，没有可导入的 ChatGPT 账号信息".to_string(),
        }),
        AuthState::NotLoggedIn => Ok(CodexAccountStatusV1 {
            schema_version: SCHEMA_VERSION_V1,
            login_state: "not_logged_in".to_string(),
            auth_mode: None,
            auth_path: auth_path.to_string_lossy().to_string(),
            last_refresh: None,
            profile: None,
            snapshot,
            snapshot_path,
            local_data,
            message: "未检测到 Codex 登录。请先在终端运行 codex login，再回到此页导入".to_string(),
        }),
    }
}

pub fn import_codex_account(
    codex_home: &Path,
    install_root: &Path,
    api_base: &str,
) -> Result<CodexAccountStatusV1, String> {
    let auth_path = codex_home.join("auth.json");
    let (profile, access_token, account_id) = match read_auth_state(&auth_path)? {
        AuthState::ChatGpt {
            profile,
            access_token,
            account_id,
            ..
        } => (profile, access_token, account_id),
        AuthState::ApiKey { .. } => {
            return Err(
                "当前是 API Key 登录方式，没有可导入的 ChatGPT 账号信息。如需使用账号配额，请运行 codex login 改用账号登录"
                    .to_string(),
            )
        }
        AuthState::NotLoggedIn => {
            return Err("未检测到 Codex 登录。请先在终端运行 codex login，再回到此页导入".to_string())
        }
    };

    let usage = match fetch_usage(api_base, &access_token, account_id.as_deref()) {
        Ok(usage) => Some(usage),
        Err(UsageFetchError::AuthExpired(message)) => {
            // 不用过期凭证覆盖上一份好快照
            return Err(message);
        }
        Err(UsageFetchError::Transient(message)) => {
            let imported_at = rfc3339_now()?;
            let snapshot = build_snapshot(&profile, None, &imported_at);
            persist_snapshot(install_root, &snapshot)?;
            let mut status = collect_account_status(codex_home, install_root)?;
            status.message = format!("账号信息已导入，但用量获取失败：{message}");
            return Ok(status);
        }
    };

    let imported_at = rfc3339_now()?;
    let snapshot = build_snapshot(&profile, usage.as_ref(), &imported_at);
    persist_snapshot(install_root, &snapshot)?;
    let mut status = collect_account_status(codex_home, install_root)?;
    status.message = "已把当前账号信息导入到本地".to_string();
    Ok(status)
}

fn build_snapshot(
    profile: &AccountProfileV1,
    usage: Option<&AccountUsageV1>,
    imported_at: &str,
) -> AccountSnapshotV1 {
    AccountSnapshotV1 {
        schema_version: SCHEMA_VERSION_V1,
        imported_at: imported_at.to_string(),
        email: profile.email.clone(),
        name: profile.name.clone(),
        plan_type: profile.plan_type.clone(),
        account_id: profile.account_id.clone(),
        usage: usage.cloned(),
    }
}

fn persist_snapshot(install_root: &Path, snapshot: &AccountSnapshotV1) -> Result<(), String> {
    let data = serde_json::to_string_pretty(snapshot)
        .map_err(|error| format!("生成账号快照失败: {error}"))?;
    atomic_write(&snapshot_path(install_root), format!("{data}\n").as_bytes())
        .map_err(|error| format!("保存账号快照失败: {error}"))
}

fn read_snapshot(install_root: &Path) -> Option<AccountSnapshotV1> {
    let path = snapshot_path(install_root);
    let data = fs::read(&path).ok()?;
    serde_json::from_slice(&data).ok()
}

const MAX_WALK_ENTRIES: u64 = 100_000;
const MAX_SESSION_INDEX_BYTES: u64 = 1024 * 1024;
const RECENT_THREAD_LIMIT: usize = 3;

#[derive(Default)]
struct WalkStats {
    jsonl_files: u64,
    bytes: u64,
    latest_mtime: Option<i64>,
    entries: u64,
    truncated: bool,
}

fn collect_local_data(codex_home: &Path) -> LocalDataV1 {
    let mut sessions = WalkStats::default();
    walk_stats(&codex_home.join("sessions"), &mut sessions, true);
    let mut archived = WalkStats::default();
    walk_stats(&codex_home.join("archived_sessions"), &mut archived, true);
    let mut total = WalkStats::default();
    walk_stats(codex_home, &mut total, false);
    let logs_bytes = top_level_log_bytes(codex_home);

    let latest_mtime = [sessions.latest_mtime, archived.latest_mtime]
        .into_iter()
        .flatten()
        .max();

    LocalDataV1 {
        session_count: sessions.jsonl_files,
        archived_session_count: archived.jsonl_files,
        latest_session_at: latest_mtime.and_then(rfc3339_from_unix),
        recent_threads: read_recent_threads(&codex_home.join("session_index.jsonl")),
        total_bytes: total.bytes,
        sessions_bytes: sessions.bytes + archived.bytes,
        logs_bytes,
        codex_home: codex_home.to_string_lossy().to_string(),
    }
}

fn walk_stats(dir: &Path, stats: &mut WalkStats, count_jsonl: bool) {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if stats.entries >= MAX_WALK_ENTRIES {
            stats.truncated = true;
            return;
        }
        let Ok(entries) = fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            stats.entries += 1;
            if stats.entries >= MAX_WALK_ENTRIES {
                stats.truncated = true;
                return;
            }
            // symlink_metadata：不跟随符号链接，避免统计目录外内容
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                stack.push(entry.path());
                continue;
            }
            if !metadata.is_file() {
                continue;
            }
            stats.bytes += metadata.len();
            let is_jsonl = entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("jsonl"));
            if count_jsonl && is_jsonl {
                stats.jsonl_files += 1;
            }
            if count_jsonl && is_jsonl {
                if let Ok(modified) = metadata.modified() {
                    let seconds = modified
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|duration| duration.as_secs() as i64)
                        .unwrap_or_default();
                    stats.latest_mtime = Some(
                        stats
                            .latest_mtime
                            .map_or(seconds, |latest: i64| latest.max(seconds)),
                    );
                }
            }
        }
    }
}

fn top_level_log_bytes(codex_home: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(codex_home) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.contains(".sqlite"))
        })
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .map(|metadata| metadata.len())
        .sum()
}

fn read_recent_threads(index_path: &Path) -> Vec<RecentThreadV1> {
    let Ok(data) = fs::read(index_path) else {
        return Vec::new();
    };
    let data = &data[..data.len().min(MAX_SESSION_INDEX_BYTES as usize)];
    let mut threads: Vec<RecentThreadV1> = data
        .split(|byte| *byte == b'\n')
        .filter_map(|line| serde_json::from_slice::<Value>(line).ok())
        .filter_map(|line| {
            let id = line.get("id").and_then(Value::as_str)?.trim().to_string();
            if id.is_empty() {
                return None;
            }
            Some(RecentThreadV1 {
                id,
                name: line
                    .get("thread_name")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .unwrap_or_default()
                    .to_string(),
                updated_at: line
                    .get("updated_at")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            })
        })
        .collect();
    threads.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    threads.truncate(RECENT_THREAD_LIMIT);
    threads
}

fn read_auth_state(auth_path: &Path) -> Result<AuthState, String> {
    let metadata = match fs::metadata(auth_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(AuthState::NotLoggedIn)
        }
        Err(error) => return Err(format!("读取 Codex 登录文件失败: {error}")),
    };
    if !metadata.is_file() {
        return Err("Codex 登录路径不是文件，请检查 ~/.codex/auth.json".to_string());
    }
    if metadata.len() > MAX_AUTH_FILE_BYTES {
        return Err(format!(
            "Codex 登录文件超过 {} KiB 安全上限，已拒绝解析",
            MAX_AUTH_FILE_BYTES / 1024
        ));
    }
    ensure_within_codex_home(auth_path)?;
    let data = fs::read(auth_path).map_err(|error| format!("读取 Codex 登录文件失败: {error}"))?;
    let auth: CodexAuthFile = serde_json::from_slice(&data).map_err(|error| {
        format!(
            "Codex 登录文件内容损坏，无法解析（{}）。可备份后运行 codex login 重新登录",
            error
        )
    })?;

    if let Some(tokens) = auth.tokens {
        let has_session = tokens
            .id_token
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
            || tokens
                .access_token
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty());
        if has_session {
            let claims = tokens
                .id_token
                .as_deref()
                .map(decode_id_token_claims)
                .transpose()?;
            let access_token = tokens
                .access_token
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    "Codex 登录文件缺少 access_token，请运行 codex login 重新登录".to_string()
                })?
                .to_string();
            let auth_claims = claims.as_ref().and_then(|claims| claims.auth.as_ref());
            return Ok(AuthState::ChatGpt {
                auth_mode: auth.auth_mode.clone(),
                last_refresh: auth.last_refresh.clone(),
                profile: AccountProfileV1 {
                    email: claims.as_ref().and_then(|claims| claims.email.clone()),
                    name: claims.as_ref().and_then(|claims| claims.name.clone()),
                    plan_type: auth_claims.and_then(|claims| claims.chatgpt_plan_type.clone()),
                    account_id: tokens.account_id.clone().or_else(|| {
                        auth_claims.and_then(|claims| claims.chatgpt_account_id.clone())
                    }),
                    token_expires_at: claims
                        .as_ref()
                        .and_then(|claims| claims.exp)
                        .and_then(rfc3339_from_unix),
                },
                access_token,
                account_id: tokens
                    .account_id
                    .or_else(|| auth_claims.and_then(|claims| claims.chatgpt_account_id.clone())),
            });
        }
    }
    if auth
        .openai_api_key
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        return Ok(AuthState::ApiKey {
            auth_mode: auth.auth_mode,
        });
    }
    Ok(AuthState::NotLoggedIn)
}

fn ensure_within_codex_home(auth_path: &Path) -> Result<(), String> {
    let canonical = fs::canonicalize(auth_path)
        .map_err(|error| format!("解析 Codex 登录文件路径失败: {error}"))?;
    let Some(home) = auth_path.parent() else {
        return Ok(());
    };
    let canonical_home =
        fs::canonicalize(home).map_err(|error| format!("解析 Codex 目录失败: {error}"))?;
    if !canonical.starts_with(&canonical_home) {
        return Err("Codex 登录文件指向 Codex 目录之外，已拒绝读取".to_string());
    }
    Ok(())
}

fn decode_id_token_claims(token: &str) -> Result<IdTokenClaims, String> {
    let mut segments = token.split('.');
    let (Some(_header), Some(payload), Some(_signature)) =
        (segments.next(), segments.next(), segments.next())
    else {
        return Err(
            "Codex 登录文件中的 id_token 不是有效 JWT，请运行 codex login 重新登录".to_string(),
        );
    };
    let decoded = URL_SAFE_NO_PAD.decode(payload).map_err(|_| {
        "Codex 登录文件中的 id_token 编码损坏，请运行 codex login 重新登录".to_string()
    })?;
    serde_json::from_slice(&decoded).map_err(|_| {
        "Codex 登录文件中的 id_token 内容损坏，请运行 codex login 重新登录".to_string()
    })
}

enum UsageFetchError {
    AuthExpired(String),
    Transient(String),
}

fn fetch_usage(
    api_base: &str,
    access_token: &str,
    account_id: Option<&str>,
) -> Result<AccountUsageV1, UsageFetchError> {
    let endpoint = format!("{}/wham/usage", api_base.trim_end_matches('/'));
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(USAGE_TIMEOUT)
        .timeout_write(Duration::from_secs(8))
        .try_proxy_from_env(true)
        .build();
    let mut request = agent
        .get(&endpoint)
        .set("Accept", "application/json")
        .set(
            "User-Agent",
            concat!("codex_cli_rs/", env!("CARGO_PKG_VERSION")),
        )
        .set("Authorization", &format!("Bearer {access_token}"));
    if let Some(account_id) = account_id.filter(|value| !value.trim().is_empty()) {
        request = request.set("chatgpt-account-id", account_id);
    }
    let response = request.call().map_err(|error| match error {
        ureq::Error::Status(401 | 403, _) => UsageFetchError::AuthExpired(
            "Codex 登录状态已过期。请在终端运行 codex login 重新登录后，再回到此页导入".to_string(),
        ),
        ureq::Error::Status(code, _) => {
            UsageFetchError::Transient(format!("用量接口返回 HTTP {code}"))
        }
        ureq::Error::Transport(error) => {
            UsageFetchError::Transient(format!("无法连接用量接口: {error}"))
        }
    })?;
    let mut body = Vec::new();
    response
        .into_reader()
        .take(MAX_USAGE_RESPONSE_BYTES + 1)
        .read_to_end(&mut body)
        .map_err(|error| UsageFetchError::Transient(format!("读取用量响应失败: {error}")))?;
    if body.len() as u64 > MAX_USAGE_RESPONSE_BYTES {
        return Err(UsageFetchError::Transient(
            "用量响应超过安全大小限制".to_string(),
        ));
    }
    parse_usage_response(&body).map_err(UsageFetchError::Transient)
}

fn parse_usage_response(body: &[u8]) -> Result<AccountUsageV1, String> {
    let payload: Value =
        serde_json::from_slice(body).map_err(|_| "用量接口未返回有效 JSON".to_string())?;
    let rate_limit = payload.get("rate_limit");
    let allowed = rate_limit
        .and_then(|value| value.get("allowed"))
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let limit_reached = rate_limit
        .and_then(|value| value.get("limit_reached"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let credits = payload
        .get("credits")
        .filter(|value| value.is_object())
        .map(|value| UsageCreditsV1 {
            has_credits: value
                .get("has_credits")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            unlimited: value
                .get("unlimited")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            balance: value
                .get("balance")
                .and_then(Value::as_str)
                .map(str::to_string),
        });
    Ok(AccountUsageV1 {
        fetched_at: rfc3339_now()?,
        allowed,
        limit_reached,
        primary_window: rate_limit.and_then(|value| parse_window(value.get("primary_window"))),
        secondary_window: rate_limit.and_then(|value| parse_window(value.get("secondary_window"))),
        credits,
    })
}

fn parse_window(value: Option<&Value>) -> Option<UsageWindowV1> {
    let value = value.filter(|value| value.is_object())?;
    let used_percent = value.get("used_percent").and_then(Value::as_f64)?;
    Some(UsageWindowV1 {
        used_percent: used_percent.round().clamp(0.0, 100.0) as u8,
        limit_window_seconds: value
            .get("limit_window_seconds")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        reset_after_seconds: value
            .get("reset_after_seconds")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        reset_at: value
            .get("reset_at")
            .and_then(Value::as_i64)
            .and_then(rfc3339_from_unix),
    })
}

fn rfc3339_now() -> Result<String, String> {
    rfc3339_from_unix(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or_default(),
    )
    .ok_or_else(|| "生成当前时间失败".to_string())
}

fn rfc3339_from_unix(seconds: i64) -> Option<String> {
    time::OffsetDateTime::from_unix_timestamp(seconds)
        .ok()?
        .format(&time::format_description::well_known::Rfc3339)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{
        io::{BufRead, BufReader, Write},
        net::{TcpListener, TcpStream},
        sync::mpsc::{self, Receiver},
        thread::{self, JoinHandle},
    };
    use uuid::Uuid;

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "codex-assistant-account-{name}-{}",
            Uuid::new_v4().simple()
        ))
    }

    fn make_jwt(payload: Value) -> String {
        let header = URL_SAFE_NO_PAD.encode(json!({"alg":"RS256","typ":"JWT"}).to_string());
        let body = URL_SAFE_NO_PAD.encode(payload.to_string());
        format!("{header}.{body}.test-signature")
    }

    fn write_auth(root: &Path, auth: Value) -> PathBuf {
        fs::create_dir_all(root).expect("create codex home");
        let path = root.join("auth.json");
        fs::write(&path, auth.to_string()).expect("write auth.json");
        path
    }

    fn chatgpt_auth() -> Value {
        json!({
            "auth_mode": "chatgpt",
            "OPENAI_API_KEY": null,
            "tokens": {
                "id_token": make_jwt(json!({
                    "email": "tester@example.com",
                    "name": "体验者",
                    "exp": 1_900_000_000,
                    "https://api.openai.com/auth": {
                        "chatgpt_account_id": "acct-jwt-1",
                        "chatgpt_plan_type": "plus"
                    }
                })),
                "access_token": "access-token-secret-1",
                "refresh_token": "refresh-token-secret-1",
                "account_id": "acct-1"
            },
            "last_refresh": "2026-08-01T10:00:00Z"
        })
    }

    fn usage_body() -> String {
        json!({
            "user_id": "user-1",
            "account_id": "acct-1",
            "email": "tester@example.com",
            "plan_type": "plus",
            "rate_limit": {
                "allowed": true,
                "limit_reached": false,
                "primary_window": {
                    "used_percent": 37.4,
                    "limit_window_seconds": 604800,
                    "reset_after_seconds": 1000,
                    "reset_at": 1_786_160_000
                },
                "secondary_window": null
            },
            "credits": {
                "has_credits": false,
                "unlimited": false,
                "balance": "0.00"
            }
        })
        .to_string()
    }

    fn spawn_usage_server(responses: Vec<String>) -> (String, Receiver<String>, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind usage server");
        let address = listener.local_addr().expect("usage server address");
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().expect("accept usage request");
                let request = read_request(&mut stream);
                sender.send(request).expect("capture request");
                stream
                    .write_all(response.as_bytes())
                    .expect("write response");
            }
        });
        (format!("http://{address}"), receiver, handle)
    }

    fn read_request(stream: &mut TcpStream) -> String {
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("set read timeout");
        let mut reader = BufReader::new(stream);
        let mut request = String::new();
        loop {
            let mut line = String::new();
            let count = reader.read_line(&mut line).expect("read request line");
            if count == 0 || line == "\r\n" {
                break;
            }
            request.push_str(&line);
        }
        request
    }

    fn http_json(status: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    #[test]
    fn jwt_decode_extracts_account_claims() {
        let token = make_jwt(json!({
            "email": "a@b.c",
            "name": "Tester",
            "exp": 1_900_000_000,
            "https://api.openai.com/auth": {
                "chatgpt_account_id": "acct-9",
                "chatgpt_plan_type": "team"
            }
        }));
        let claims = decode_id_token_claims(&token).expect("decode claims");
        assert_eq!(claims.email.as_deref(), Some("a@b.c"));
        let auth = claims.auth.expect("auth claims");
        assert_eq!(auth.chatgpt_plan_type.as_deref(), Some("team"));
        assert_eq!(auth.chatgpt_account_id.as_deref(), Some("acct-9"));
    }

    #[test]
    fn jwt_decode_rejects_malformed_tokens() {
        for token in ["not-a-jwt", "a.b", "a.!!!.c", "a.bm90LWpzb24.c"] {
            assert!(
                decode_id_token_claims(token).is_err(),
                "token {token} must fail"
            );
        }
    }

    #[test]
    fn login_state_covers_chatgpt_api_key_and_absent() {
        let root = test_root("states");
        let runtime = root.join("runtime");
        let codex_home = root.join("codex");

        let status = collect_account_status(&codex_home, &runtime).unwrap();
        assert_eq!(status.login_state, "not_logged_in");
        assert!(status.profile.is_none());

        write_auth(&codex_home, chatgpt_auth());
        let status = collect_account_status(&codex_home, &runtime).unwrap();
        assert_eq!(status.login_state, "chatgpt");
        let profile = status.profile.expect("profile");
        assert_eq!(profile.email.as_deref(), Some("tester@example.com"));
        assert_eq!(profile.plan_type.as_deref(), Some("plus"));
        assert_eq!(profile.account_id.as_deref(), Some("acct-1"));
        assert_eq!(status.last_refresh.as_deref(), Some("2026-08-01T10:00:00Z"));

        write_auth(
            &codex_home,
            json!({"auth_mode": "apikey", "OPENAI_API_KEY": "sk-test", "tokens": null}),
        );
        let status = collect_account_status(&codex_home, &runtime).unwrap();
        assert_eq!(status.login_state, "api_key");
        assert!(status.profile.is_none());

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn corrupt_auth_file_is_error_not_logged_out() {
        let root = test_root("corrupt");
        let codex_home = root.join("codex");
        fs::create_dir_all(&codex_home).unwrap();
        fs::write(codex_home.join("auth.json"), "{not json").unwrap();
        let error = collect_account_status(&codex_home, &root.join("runtime"))
            .expect_err("corrupt auth must fail");
        assert!(error.contains("损坏"), "unexpected error: {error}");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn auth_file_size_limit_is_enforced() {
        let root = test_root("oversize");
        let codex_home = root.join("codex");
        fs::create_dir_all(&codex_home).unwrap();
        fs::write(
            codex_home.join("auth.json"),
            vec![b' '; (MAX_AUTH_FILE_BYTES + 1) as usize],
        )
        .unwrap();
        let error = collect_account_status(&codex_home, &root.join("runtime"))
            .expect_err("oversize auth must fail");
        assert!(error.contains("上限"), "unexpected error: {error}");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn usage_parser_handles_full_and_sparse_payloads() {
        let usage = parse_usage_response(usage_body().as_bytes()).expect("parse usage");
        assert!(usage.allowed);
        assert!(!usage.limit_reached);
        let primary = usage.primary_window.expect("primary window");
        assert_eq!(primary.used_percent, 37);
        assert_eq!(primary.limit_window_seconds, 604800);
        assert!(primary.reset_at.is_some());
        assert!(usage.secondary_window.is_none());
        let credits = usage.credits.expect("credits");
        assert!(!credits.has_credits && !credits.unlimited);
        assert_eq!(credits.balance.as_deref(), Some("0.00"));

        let sparse = parse_usage_response(br#"{"rate_limit": null}"#).expect("parse sparse");
        assert!(sparse.allowed);
        assert!(sparse.primary_window.is_none());
        assert!(sparse.credits.is_none());

        assert!(parse_usage_response(b"not json").is_err());
    }

    #[test]
    fn usage_window_percent_is_rounded_and_clamped() {
        let window = parse_window(Some(&json!({
            "used_percent": 100.6,
            "limit_window_seconds": 300,
            "reset_after_seconds": 60,
            "reset_at": 1_786_160_000
        })))
        .expect("window");
        assert_eq!(window.used_percent, 100);
        assert!(parse_window(Some(&json!({"used_percent": null}))).is_none());
        assert!(parse_window(None).is_none());
    }

    #[test]
    fn import_round_trip_persists_snapshot_without_tokens() {
        let root = test_root("import");
        let runtime = root.join("runtime");
        let codex_home = root.join("codex");
        write_auth(&codex_home, chatgpt_auth());
        let (api_base, requests, server) =
            spawn_usage_server(vec![http_json("200 OK", &usage_body())]);

        let status =
            import_codex_account(&codex_home, &runtime, &api_base).expect("import should succeed");
        assert_eq!(status.login_state, "chatgpt");
        let snapshot = status.snapshot.expect("snapshot");
        assert_eq!(snapshot.email.as_deref(), Some("tester@example.com"));
        assert_eq!(snapshot.plan_type.as_deref(), Some("plus"));
        let usage = snapshot.usage.expect("usage");
        assert_eq!(usage.primary_window.expect("primary").used_percent, 37);

        let stored = fs::read_to_string(snapshot_path(&runtime)).expect("snapshot file");
        for secret in [
            "access-token-secret-1",
            "refresh-token-secret-1",
            "id_token",
            "access_token",
            "refresh_token",
            "Bearer",
        ] {
            assert!(!stored.contains(secret), "snapshot leaked {secret}");
        }

        let request = requests.recv().expect("usage request");
        server.join().expect("usage server");
        assert!(request.starts_with("GET /wham/usage "));
        assert!(request.contains("Authorization: Bearer access-token-secret-1"));
        assert!(request.contains("chatgpt-account-id: acct-1"));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn import_auth_expiry_does_not_overwrite_snapshot() {
        let root = test_root("expired");
        let runtime = root.join("runtime");
        let codex_home = root.join("codex");
        write_auth(&codex_home, chatgpt_auth());
        persist_snapshot(
            &runtime,
            &build_snapshot(
                &AccountProfileV1 {
                    email: Some("keep@example.com".to_string()),
                    name: None,
                    plan_type: Some("plus".to_string()),
                    account_id: None,
                    token_expires_at: None,
                },
                None,
                "2026-08-01T00:00:00Z",
            ),
        )
        .expect("seed snapshot");
        let (api_base, _requests, server) = spawn_usage_server(vec![http_json(
            "401 Unauthorized",
            r#"{"error":"expired"}"#,
        )]);

        let error = import_codex_account(&codex_home, &runtime, &api_base)
            .expect_err("expired token must fail");
        server.join().expect("usage server");
        assert!(error.contains("codex login"), "unexpected error: {error}");

        let status = collect_account_status(&codex_home, &runtime).unwrap();
        let snapshot = status.snapshot.expect("snapshot preserved");
        assert_eq!(snapshot.email.as_deref(), Some("keep@example.com"));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn import_keeps_profile_when_usage_unavailable() {
        let root = test_root("partial");
        let runtime = root.join("runtime");
        let codex_home = root.join("codex");
        write_auth(&codex_home, chatgpt_auth());
        let (api_base, _requests, server) =
            spawn_usage_server(vec![http_json("500 Internal Server Error", "{}")]);

        let status = import_codex_account(&codex_home, &runtime, &api_base)
            .expect("import should partially succeed");
        server.join().expect("usage server");
        assert!(
            status.message.contains("用量获取失败"),
            "{}",
            status.message
        );
        let snapshot = status.snapshot.expect("snapshot");
        assert_eq!(snapshot.email.as_deref(), Some("tester@example.com"));
        assert!(snapshot.usage.is_none());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn import_requires_chatgpt_login() {
        let root = test_root("requires-login");
        let runtime = root.join("runtime");
        let codex_home = root.join("codex");

        let error = import_codex_account(&codex_home, &runtime, "http://127.0.0.1:1")
            .expect_err("not logged in");
        assert!(error.contains("codex login"), "unexpected error: {error}");

        write_auth(
            &codex_home,
            json!({"auth_mode": "apikey", "OPENAI_API_KEY": "sk-test", "tokens": null}),
        );
        let error = import_codex_account(&codex_home, &runtime, "http://127.0.0.1:1")
            .expect_err("api key mode");
        assert!(error.contains("API Key"), "unexpected error: {error}");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn corrupt_snapshot_is_ignored() {
        let root = test_root("bad-snapshot");
        let runtime = root.join("runtime");
        fs::create_dir_all(&runtime).unwrap();
        fs::write(snapshot_path(&runtime), "{broken").unwrap();
        let status = collect_account_status(&root.join("codex"), &runtime).unwrap();
        assert_eq!(status.login_state, "not_logged_in");
        assert!(status.snapshot.is_none());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn local_data_scans_sessions_threads_and_storage() {
        let root = test_root("local-data");
        let codex_home = root.join("codex");
        let day = codex_home
            .join("sessions")
            .join("2026")
            .join("08")
            .join("02");
        fs::create_dir_all(&day).unwrap();
        fs::write(day.join("rollout-a.jsonl"), vec![b'x'; 100]).unwrap();
        fs::write(day.join("rollout-b.jsonl"), vec![b'x'; 50]).unwrap();
        fs::write(day.join("notes.txt"), b"not a session").unwrap();
        let archived = codex_home.join("archived_sessions");
        fs::create_dir_all(&archived).unwrap();
        fs::write(archived.join("rollout-c.jsonl"), vec![b'x'; 25]).unwrap();
        fs::write(codex_home.join("logs_2.sqlite"), vec![b'x'; 1000]).unwrap();
        fs::write(codex_home.join("logs_2.sqlite-wal"), vec![b'x'; 500]).unwrap();
        fs::write(
            codex_home.join("session_index.jsonl"),
            concat!(
                "{\"id\":\"t-old\",\"thread_name\":\"较早会话\",\"updated_at\":\"2026-07-30T10:00:00Z\"}\n",
                "{not json 会被跳过}\n",
                "{\"id\":\"t-new\",\"thread_name\":\"最新会话\",\"updated_at\":\"2026-08-02T03:00:00Z\"}\n",
                "{\"id\":\"t-mid\",\"thread_name\":\"\",\"updated_at\":\"2026-08-01T10:00:00Z\"}\n",
                "{\"id\":\"t-mid2\",\"thread_name\":\"次新会话\",\"updated_at\":\"2026-08-01T11:00:00Z\"}\n"
            ),
        )
        .unwrap();

        let data = collect_local_data(&codex_home);
        assert_eq!(data.session_count, 2);
        assert_eq!(data.archived_session_count, 1);
        assert_eq!(data.sessions_bytes, 188);
        assert_eq!(data.logs_bytes, 1500);
        assert!(data.total_bytes >= 188 + 1500);
        assert!(data.latest_session_at.is_some());
        assert_eq!(data.codex_home, codex_home.to_string_lossy().to_string());

        let threads = &data.recent_threads;
        assert_eq!(threads.len(), RECENT_THREAD_LIMIT);
        assert_eq!(threads[0].id, "t-new");
        assert_eq!(threads[1].id, "t-mid2");
        assert_eq!(threads[2].id, "t-mid");
        assert_eq!(threads[2].name, "");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn local_data_tolerates_missing_directories() {
        let root = test_root("local-data-empty");
        let data = collect_local_data(&root.join("nonexistent"));
        assert_eq!(data.session_count, 0);
        assert_eq!(data.total_bytes, 0);
        assert!(data.recent_threads.is_empty());
        assert!(data.latest_session_at.is_none());
    }

    #[test]
    fn status_includes_local_data() {
        let root = test_root("status-local");
        let codex_home = root.join("codex");
        let day = codex_home
            .join("sessions")
            .join("2026")
            .join("08")
            .join("02");
        fs::create_dir_all(&day).unwrap();
        fs::write(day.join("rollout-a.jsonl"), b"{}").unwrap();
        let status = collect_account_status(&codex_home, &root.join("runtime")).unwrap();
        assert_eq!(status.login_state, "not_logged_in");
        assert_eq!(status.local_data.session_count, 1);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    #[ignore = "live：需要本机真实 codex 登录与可用网络"]
    fn live_import_against_real_account() {
        let codex_home = std::env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").expect("HOME")).join(".codex"));
        let runtime = test_root("live");
        let status = import_codex_account(&codex_home, &runtime, DEFAULT_ACCOUNT_API_BASE)
            .expect("live import");
        assert_eq!(status.login_state, "chatgpt");
        let snapshot = status.snapshot.expect("live snapshot");
        assert!(snapshot.email.is_some());
        let stored = fs::read_to_string(snapshot_path(&runtime)).expect("live snapshot file");
        assert!(!stored.contains("access_token"));
        println!("live import message: {}", status.message);
        fs::remove_dir_all(&runtime).ok();
    }

    #[cfg(unix)]
    #[test]
    fn auth_symlink_escaping_codex_home_is_rejected() {
        use std::os::unix::fs::symlink;
        let root = test_root("symlink");
        let codex_home = root.join("codex");
        let outside = root.join("outside");
        fs::create_dir_all(&codex_home).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("real-auth.json"), chatgpt_auth().to_string()).unwrap();
        symlink(outside.join("real-auth.json"), codex_home.join("auth.json")).unwrap();
        let error = collect_account_status(&codex_home, &root.join("runtime"))
            .expect_err("escaping symlink must fail");
        assert!(error.contains("之外"), "unexpected error: {error}");
        fs::remove_dir_all(&root).ok();
    }
}
