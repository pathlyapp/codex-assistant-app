use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::VecDeque,
    env, fs,
    io::{Cursor, Write},
    path::Path,
    sync::{LazyLock, Mutex},
};
use url::Url;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use crate::{
    config_transaction::{atomic_write, TransactionStatus, TransactionSummary},
    contracts::{new_support_id, redact_technical_detail, SystemStatusV1},
};

const MAX_LOG_BYTES: usize = 256 * 1024;
const MAX_LOG_LINE_BYTES: usize = 8 * 1024;
const BUNDLE_SCHEMA_VERSION: u8 = 1;
const SECRET_DETECTED_MARKER: &str = "DIAGNOSTIC_SECRET_DETECTED";

static RECENT_LOGS: LazyLock<Mutex<LogBuffer>> = LazyLock::new(|| Mutex::new(LogBuffer::default()));
static SAFE_SUPPORT_ID: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^CA-[A-Z0-9-]{8,48}$").expect("support id regex"));
static SUSPECTED_SECRET: LazyLock<Vec<(Regex, bool)>> = LazyLock::new(|| {
    [
        (r#"(?i)\bBearer\s+\S+"#, true),
        (
            r#"(?i)\b(?:access[_-]?key|api[_-]?key|token|key)=[^&\s,;"']+"#,
            true,
        ),
        (
            r#"(?i)"(?:access[_-]?key|api[_-]?key|token|key)"\s*:\s*"[^"]+""#,
            true,
        ),
        (
            r#"(?i)\b(?:access[_-]?key|api[_-]?key|token|key)\s*:\s*[^\s,;"']+"#,
            true,
        ),
        (r#"(?i)https?://[^/\s:@]+(?::[^/\s@]*)?@"#, false),
        (r#"(?i)\b[A-Z]:\\Users\\[^\\\s/:*?"<>|]+"#, false),
        (r#"/Users/[^/\s]+"#, false),
        (r#"/home/[^/\s]+"#, false),
        (
            r#"(?i)\b(?:sk-[A-Za-z0-9_-]{8,}|ghp_[A-Za-z0-9]{8,}|xoxb-[A-Za-z0-9-]{8,})\b"#,
            false,
        ),
    ]
    .into_iter()
    .map(|(pattern, permits_redacted)| {
        (
            Regex::new(pattern).expect("diagnostic secret regex"),
            permits_redacted,
        )
    })
    .collect()
});

#[derive(Default)]
struct LogBuffer {
    lines: VecDeque<String>,
    bytes: usize,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticExportRequest {
    #[serde(default)]
    pub support_id: String,
    #[serde(default)]
    pub error_code: String,
    #[serde(default)]
    pub error_stage: String,
    #[serde(default)]
    pub suggested_action: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticBundle {
    pub file_name: String,
    pub content_base64: String,
    pub byte_length: usize,
    pub sha256: String,
    pub support_id: String,
    pub saved_path: String,
}

#[derive(Clone, Copy, Debug)]
pub struct PermissionSummary {
    pub config_parent_exists: bool,
    pub config_parent_read_only: bool,
    pub install_root_exists: bool,
    pub install_root_read_only: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticStatus<'a> {
    schema_version: u8,
    support_id: &'a str,
    generated_at: &'a str,
    assistant: AssistantStatus<'a>,
    official_app: OfficialAppStatus,
    config: ConfigStatus,
    router: RouterStatus,
    environment: EnvironmentStatus,
    last_transaction: Option<SafeTransaction>,
    last_error: Option<SafeError>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AssistantStatus<'a> {
    version: &'a str,
    platform: &'a str,
    architecture: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OfficialAppStatus {
    state: String,
    installed: bool,
    trusted: bool,
    source: String,
    version: Option<String>,
    detail: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigStatus {
    state: String,
    effective_source: String,
    backup_available: bool,
    last_transaction_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RouterStatus {
    state: String,
    reachable: bool,
    endpoint: Option<SafeEndpoint>,
    result: String,
    last_verified_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SafeEndpoint {
    protocol: String,
    host: String,
    port: Option<u16>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnvironmentStatus {
    proxy_variables_present: Vec<String>,
    ca_variables_present: Vec<String>,
    permissions: PermissionSummaryJson,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PermissionSummaryJson {
    config_parent_exists: bool,
    config_parent_read_only: bool,
    install_root_exists: bool,
    install_root_read_only: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SafeTransaction {
    transaction_id: String,
    operation: String,
    status: String,
    completed_at: Option<String>,
    failure: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SafeError {
    code: String,
    stage: String,
    suggested_action: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BundleManifest<'a> {
    schema_version: u8,
    support_id: &'a str,
    created_at: &'a str,
    assistant_version: &'a str,
    platform: &'a str,
    architecture: &'a str,
    files: Vec<ManifestFile>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestFile {
    name: &'static str,
    bytes: usize,
    sha256: String,
}

pub fn record_log(line: &str) {
    let redacted = truncate_utf8(&redact_technical_detail(line), MAX_LOG_LINE_BYTES);
    let line_bytes = redacted.len() + 1;
    let mut buffer = RECENT_LOGS
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    while buffer.bytes + line_bytes > MAX_LOG_BYTES {
        let Some(removed) = buffer.lines.pop_front() else {
            break;
        };
        buffer.bytes = buffer.bytes.saturating_sub(removed.len() + 1);
    }
    if line_bytes <= MAX_LOG_BYTES {
        buffer.bytes += line_bytes;
        buffer.lines.push_back(redacted);
    }
}

pub fn permission_summary(config_path: &Path, install_root: &Path) -> PermissionSummary {
    let config_parent = config_path.parent();
    PermissionSummary {
        config_parent_exists: config_parent.is_some_and(Path::exists),
        config_parent_read_only: config_parent
            .and_then(|path| path.metadata().ok())
            .is_some_and(|metadata| metadata.permissions().readonly()),
        install_root_exists: install_root.exists(),
        install_root_read_only: install_root
            .metadata()
            .ok()
            .is_some_and(|metadata| metadata.permissions().readonly()),
    }
}

pub fn build_bundle(
    status: &SystemStatusV1,
    transaction: Option<&TransactionSummary>,
    permissions: PermissionSummary,
    request: &DiagnosticExportRequest,
    assistant_version: &str,
    generated_at: &str,
) -> Result<DiagnosticBundle, String> {
    let support_id = selected_support_id(&request.support_id);
    let recent_log = recent_log_text();
    let diagnostic_status = DiagnosticStatus {
        schema_version: BUNDLE_SCHEMA_VERSION,
        support_id: &support_id,
        generated_at,
        assistant: AssistantStatus {
            version: assistant_version,
            platform: &status.platform,
            architecture: &status.architecture,
        },
        official_app: OfficialAppStatus {
            state: status.app.state.clone(),
            installed: status.app.installed,
            trusted: status.app.trusted,
            source: status.app.source.clone(),
            version: status.app.version.clone(),
            detail: redact_technical_detail(&status.app.detail),
        },
        config: ConfigStatus {
            state: status.config.state.clone(),
            effective_source: status.config.effective_source.clone(),
            backup_available: status.config.backup_available,
            last_transaction_id: status.config.last_transaction_id.clone(),
        },
        router: RouterStatus {
            state: status.router.state.clone(),
            reachable: status.router.reachable,
            endpoint: status.router.gateway.as_deref().and_then(safe_endpoint),
            result: redact_technical_detail(&status.router.detail),
            last_verified_at: status.router.last_verified_at.clone(),
        },
        environment: EnvironmentStatus {
            proxy_variables_present: present_variables(&[
                "HTTP_PROXY",
                "HTTPS_PROXY",
                "ALL_PROXY",
                "NO_PROXY",
            ]),
            ca_variables_present: present_variables(&[
                "SSL_CERT_FILE",
                "REQUESTS_CA_BUNDLE",
                "CURL_CA_BUNDLE",
            ]),
            permissions: PermissionSummaryJson {
                config_parent_exists: permissions.config_parent_exists,
                config_parent_read_only: permissions.config_parent_read_only,
                install_root_exists: permissions.install_root_exists,
                install_root_read_only: permissions.install_root_read_only,
            },
        },
        last_transaction: transaction.map(safe_transaction),
        last_error: safe_error(request),
    };

    let status_json = pretty_json(&diagnostic_status)?;
    ensure_no_suspected_secret("status.json", &status_json)?;
    ensure_no_suspected_secret("recent.log", &recent_log)?;

    let manifest = BundleManifest {
        schema_version: BUNDLE_SCHEMA_VERSION,
        support_id: &support_id,
        created_at: generated_at,
        assistant_version,
        platform: &status.platform,
        architecture: &status.architecture,
        files: vec![
            manifest_file("status.json", status_json.as_bytes()),
            manifest_file("recent.log", recent_log.as_bytes()),
        ],
    };
    let manifest_json = pretty_json(&manifest)?;
    ensure_no_suspected_secret("manifest.json", &manifest_json)?;

    let checksums = format!(
        "{}  manifest.json\n{}  status.json\n{}  recent.log\n",
        sha256_hex(manifest_json.as_bytes()),
        sha256_hex(status_json.as_bytes()),
        sha256_hex(recent_log.as_bytes())
    );
    let archive = write_archive(&manifest_json, &status_json, &recent_log, &checksums)?;
    let sha256 = sha256_hex(&archive);
    Ok(DiagnosticBundle {
        file_name: format!("diagnostics-{support_id}.zip"),
        content_base64: BASE64_STANDARD.encode(&archive),
        byte_length: archive.len(),
        sha256,
        support_id,
        saved_path: String::new(),
    })
}

pub fn save_bundle(bundle: &mut DiagnosticBundle, directory: &Path) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| format!("创建系统下载目录失败: {error}"))?;
    let bytes = BASE64_STANDARD
        .decode(&bundle.content_base64)
        .map_err(|error| format!("诊断包内部编码无效: {error}"))?;
    if bytes.len() != bundle.byte_length || sha256_hex(&bytes) != bundle.sha256 {
        return Err("诊断包写入前完整性复核失败".to_string());
    }
    let destination = directory.join(&bundle.file_name);
    atomic_write(&destination, &bytes)?;
    let stored =
        fs::read(&destination).map_err(|error| format!("读取已保存诊断包失败: {error}"))?;
    if stored.len() != bundle.byte_length || sha256_hex(&stored) != bundle.sha256 {
        return Err("诊断包写入后完整性复核失败".to_string());
    }
    bundle.saved_path = destination.to_string_lossy().to_string();
    Ok(())
}

fn selected_support_id(candidate: &str) -> String {
    let candidate = candidate.trim().to_ascii_uppercase();
    if SAFE_SUPPORT_ID.is_match(&candidate) {
        candidate
    } else {
        new_support_id()
    }
}

fn recent_log_text() -> String {
    let buffer = RECENT_LOGS
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if buffer.lines.is_empty() {
        return "No recent installer logs captured.\n".to_string();
    }
    let joined = buffer
        .lines
        .iter()
        .map(|line| redact_technical_detail(line))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{joined}\n")
}

fn safe_endpoint(value: &str) -> Option<SafeEndpoint> {
    let parsed = Url::parse(value).ok()?;
    let protocol = parsed.scheme();
    if protocol != "http" && protocol != "https" {
        return None;
    }
    Some(SafeEndpoint {
        protocol: protocol.to_string(),
        host: parsed.host_str()?.to_string(),
        port: parsed.port(),
    })
}

fn present_variables(names: &[&str]) -> Vec<String> {
    names
        .iter()
        .filter(|name| env::var_os(name).is_some())
        .map(|name| (*name).to_string())
        .collect()
}

fn safe_transaction(transaction: &TransactionSummary) -> SafeTransaction {
    SafeTransaction {
        transaction_id: transaction.transaction_id.clone(),
        operation: transaction.operation.clone(),
        status: transaction_status(&transaction.status).to_string(),
        completed_at: transaction.completed_at.clone(),
        failure: transaction.failure.as_deref().map(redact_technical_detail),
    }
}

fn transaction_status(status: &TransactionStatus) -> &'static str {
    match status {
        TransactionStatus::SnapshotCreated => "snapshot_created",
        TransactionStatus::Writing => "writing",
        TransactionStatus::Committed => "committed",
        TransactionStatus::RolledBack => "rolled_back",
        TransactionStatus::RollbackFailed => "rollback_failed",
    }
}

fn safe_error(request: &DiagnosticExportRequest) -> Option<SafeError> {
    let code = safe_identifier(&request.error_code);
    let stage = safe_identifier(&request.error_stage);
    let suggested_action = safe_identifier(&request.suggested_action);
    if code.is_empty() && stage.is_empty() && suggested_action.is_empty() {
        None
    } else {
        Some(SafeError {
            code,
            stage,
            suggested_action,
        })
    }
}

fn safe_identifier(value: &str) -> String {
    value
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
        .take(64)
        .collect()
}

fn pretty_json(value: &impl Serialize) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|error| format!("生成诊断 JSON 失败: {error}"))
}

fn manifest_file(name: &'static str, bytes: &[u8]) -> ManifestFile {
    ManifestFile {
        name,
        bytes: bytes.len(),
        sha256: sha256_hex(bytes),
    }
}

fn write_archive(
    manifest: &str,
    status: &str,
    recent_log: &str,
    checksums: &str,
) -> Result<Vec<u8>, String> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o600);
    for (name, content) in [
        ("manifest.json", manifest),
        ("status.json", status),
        ("recent.log", recent_log),
        ("checksums.txt", checksums),
    ] {
        writer
            .start_file(name, options)
            .map_err(|error| format!("创建诊断包条目失败: {error}"))?;
        writer
            .write_all(content.as_bytes())
            .map_err(|error| format!("写入诊断包条目失败: {error}"))?;
    }
    writer
        .finish()
        .map(|cursor| cursor.into_inner())
        .map_err(|error| format!("完成诊断包失败: {error}"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn ensure_no_suspected_secret(name: &str, value: &str) -> Result<(), String> {
    if SUSPECTED_SECRET.iter().any(|(pattern, permits_redacted)| {
        pattern.find_iter(value).any(|matched| {
            !permits_redacted || !matched.as_str().to_ascii_lowercase().contains("[redacted]")
        })
    }) {
        Err(format!(
            "{SECRET_DETECTED_MARKER}: 诊断包检测到疑似密钥或用户路径，已阻止导出 ({name})"
        ))
    } else {
        Ok(())
    }
}

fn truncate_utf8(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...[truncated]", &value[..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::SystemStatusInput;
    use std::io::Read;
    use zip::ZipArchive;

    fn status_with(gateway: &str) -> SystemStatusV1 {
        SystemStatusV1::from_input(SystemStatusInput {
            platform: "Windows".to_string(),
            architecture: "aarch64".to_string(),
            app_installed: true,
            app_state: "installed".to_string(),
            app_trusted: true,
            app_source: "appx".to_string(),
            app_name: "ChatGPT".to_string(),
            app_version: Some("1.2.3".to_string()),
            app_detail: r#"Installed for C:\Users\alice\AppData"#.to_string(),
            config_present: true,
            config_path: r#"C:\Users\alice\.codex\config.toml"#.to_string(),
            router_reachable: true,
            router_detail: "Responses verified with Bearer top-secret".to_string(),
            router_responses_verified: true,
            router_last_verified_at: Some("2026-07-28T00:00:00Z".to_string()),
            configured_gateway: Some(gateway.to_string()),
            configured_model: Some("model-a".to_string()),
            key_configured: true,
            backup_available: true,
            last_transaction_id: Some("tx-1".to_string()),
            transaction_recovery_failed: false,
        })
    }

    #[test]
    fn bundle_contains_only_contract_files_and_valid_checksums() {
        record_log(r#"request key=secret-value from C:\Users\alice\project"#);
        let bundle = build_bundle(
            &status_with("http://alice:secret@router.local:11434/v1?key=secret"),
            None,
            PermissionSummary {
                config_parent_exists: true,
                config_parent_read_only: false,
                install_root_exists: true,
                install_root_read_only: false,
            },
            &DiagnosticExportRequest::default(),
            "0.8.8",
            "2026-07-28T00:00:00Z",
        )
        .expect("bundle");
        let bytes = BASE64_STANDARD
            .decode(bundle.content_base64)
            .expect("base64");
        assert_eq!(bundle.byte_length, bytes.len());
        assert_eq!(bundle.sha256, sha256_hex(&bytes));
        assert!(bundle.saved_path.is_empty());

        let mut archive = ZipArchive::new(Cursor::new(bytes)).expect("zip");
        let mut names = (0..archive.len())
            .map(|index| archive.by_index(index).expect("entry").name().to_string())
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(
            names,
            [
                "checksums.txt",
                "manifest.json",
                "recent.log",
                "status.json"
            ]
        );

        let mut entries = std::collections::HashMap::new();
        for name in [
            "manifest.json",
            "status.json",
            "recent.log",
            "checksums.txt",
        ] {
            let mut content = String::new();
            archive
                .by_name(name)
                .expect("entry")
                .read_to_string(&mut content)
                .expect("text");
            assert!(!content.contains("alice"));
            assert!(!content.contains("secret"));
            entries.insert(name, content);
        }
        let checksums = &entries["checksums.txt"];
        for name in ["manifest.json", "status.json", "recent.log"] {
            assert!(
                checksums.contains(&format!("{}  {name}", sha256_hex(entries[name].as_bytes())))
            );
        }
    }

    #[test]
    fn scanner_blocks_unredacted_credentials_and_home_paths() {
        for value in [
            "Bearer live-credential",
            "api_key=live-credential",
            "access_key: live-credential",
            r#""token": "live-credential""#,
            "https://user:pass@example.com/v1",
            r#"C:\Users\alice\project"#,
            "/Users/alice/project",
            "sk-1234567890abcdef",
        ] {
            assert!(ensure_no_suspected_secret("test", value).is_err());
        }
    }

    #[test]
    fn request_contract_is_camel_case_and_identifiers_are_bounded() {
        let request: DiagnosticExportRequest = serde_json::from_value(serde_json::json!({
            "supportId": "CA-12345678-ABCDEF12",
            "errorCode": "ROUTER_AUTH_FAILED",
            "errorStage": "validate_router",
            "suggestedAction": "edit_key"
        }))
        .expect("request");
        assert_eq!(request.support_id, "CA-12345678-ABCDEF12");
        assert_eq!(
            safe_identifier("bad value/with secret"),
            "badvaluewithsecret"
        );
    }
}
