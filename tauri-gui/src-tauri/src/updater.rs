use crate::{
    contracts::{ErrorEnvelopeV1, SCHEMA_VERSION_V1},
    operation_gate,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_updater::{Error as UpdaterError, Update, UpdaterExt};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use url::Url;
use uuid::Uuid;

const UPDATE_EVENT: &str = "assistant-update-status";
const UPDATE_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_UPDATE_BYTES: usize = 256 * 1024 * 1024;
const MAX_RELEASE_NOTES_CHARS: usize = 4_000;
const DEFAULT_CHANNEL: &str = "internal-test";
const RECEIPT_SCHEMA_VERSION: u8 = 1;

const COMPILED_ENDPOINT: Option<&str> = option_env!("CODEX_ASSISTANT_UPDATE_ENDPOINT");
const COMPILED_PUBKEY: Option<&str> = option_env!("CODEX_ASSISTANT_UPDATE_PUBKEY");
const COMPILED_CHANNEL: Option<&str> = option_env!("CODEX_ASSISTANT_UPDATE_CHANNEL");

#[derive(Clone, Debug)]
struct UpdateConfiguration {
    endpoint: Url,
    pubkey: String,
    channel: String,
    allow_loopback_http: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePhaseV1 {
    NotConfigured,
    Idle,
    Checking,
    UpToDate,
    Available,
    Downloading,
    ReadyToInstall,
    Installing,
    RestartRequired,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpdateVerificationV1 {
    NotStarted,
    Verified,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReceiptSummaryV1 {
    pub from_version: String,
    pub to_version: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusV1 {
    pub schema_version: u8,
    pub phase: UpdatePhaseV1,
    pub current_version: String,
    pub channel: String,
    pub platform: String,
    pub architecture: String,
    pub configured: bool,
    pub available_version: Option<String>,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
    pub checked_at: Option<String>,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub progress_percent: Option<u8>,
    pub verification: UpdateVerificationV1,
    pub can_check: bool,
    pub can_download: bool,
    pub can_install: bool,
    pub requires_restart: bool,
    pub blocker_code: Option<String>,
    pub blocker_message: Option<String>,
    pub last_update: Option<UpdateReceiptSummaryV1>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateReceiptV1 {
    schema_version: u8,
    from_version: String,
    to_version: String,
    status: String,
    started_at: String,
    completed_at: Option<String>,
}

impl From<&UpdateReceiptV1> for UpdateReceiptSummaryV1 {
    fn from(receipt: &UpdateReceiptV1) -> Self {
        Self {
            from_version: receipt.from_version.clone(),
            to_version: receipt.to_version.clone(),
            status: receipt.status.clone(),
            started_at: receipt.started_at.clone(),
            completed_at: receipt.completed_at.clone(),
        }
    }
}

struct AssistantUpdaterInner {
    status: UpdateStatusV1,
    pending_update: Option<Update>,
    downloaded: Option<Vec<u8>>,
}

pub struct AssistantUpdaterState {
    configuration: Option<UpdateConfiguration>,
    configuration_error: Option<String>,
    receipt_path: Option<PathBuf>,
    inner: Mutex<AssistantUpdaterInner>,
}

impl AssistantUpdaterState {
    pub fn new(app: &AppHandle) -> Self {
        let configuration = update_configuration();
        let (configuration, configuration_error) = match configuration {
            Ok(configuration) => (configuration, None),
            Err(error) => (None, Some(error)),
        };
        let receipt_path = app
            .path()
            .app_data_dir()
            .ok()
            .map(|directory| directory.join("updates").join("update-state.json"));
        let last_update = receipt_path
            .as_deref()
            .and_then(|path| read_receipt(path).ok().flatten())
            .map(|receipt| UpdateReceiptSummaryV1::from(&receipt));
        let configured = configuration.is_some();
        let phase = if configured {
            UpdatePhaseV1::Idle
        } else {
            UpdatePhaseV1::NotConfigured
        };
        let blocker_message = configuration_error
            .clone()
            .or_else(|| (!configured).then(|| "当前构建未配置更新服务。".to_string()));
        let status = UpdateStatusV1 {
            schema_version: SCHEMA_VERSION_V1,
            phase,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            channel: configuration
                .as_ref()
                .map(|value| value.channel.clone())
                .unwrap_or_else(|| DEFAULT_CHANNEL.to_string()),
            platform: update_platform().to_string(),
            architecture: update_architecture().to_string(),
            configured,
            available_version: None,
            release_notes: None,
            published_at: None,
            checked_at: None,
            downloaded_bytes: 0,
            total_bytes: None,
            progress_percent: None,
            verification: UpdateVerificationV1::NotStarted,
            can_check: configured,
            can_download: false,
            can_install: false,
            requires_restart: false,
            blocker_code: (!configured).then(|| "UPDATE_NOT_CONFIGURED".to_string()),
            blocker_message,
            last_update,
        };

        Self {
            configuration,
            configuration_error,
            receipt_path,
            inner: Mutex::new(AssistantUpdaterInner {
                status,
                pending_update: None,
                downloaded: None,
            }),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.configuration.is_some()
    }

    fn snapshot(&self) -> UpdateStatusV1 {
        self.inner
            .lock()
            .map(|inner| inner.status.clone())
            .unwrap_or_else(|_| unavailable_status("更新状态暂时不可用。"))
    }

    fn set_failed(&self, code: &str, message: impl Into<String>) -> UpdateStatusV1 {
        let message = message.into();
        let Ok(mut inner) = self.inner.lock() else {
            return unavailable_status(&message);
        };
        inner.status.phase = UpdatePhaseV1::Failed;
        inner.status.can_check = inner.status.configured;
        inner.status.can_download = inner.pending_update.is_some();
        inner.status.can_install = inner.downloaded.is_some();
        inner.status.verification = if code == "UPDATE_SIGNATURE_INVALID" {
            UpdateVerificationV1::Failed
        } else {
            inner.status.verification.clone()
        };
        inner.status.blocker_code = Some(code.to_string());
        inner.status.blocker_message = Some(message);
        inner.status.clone()
    }

    fn reset_for_check(&self) -> Result<UpdateStatusV1, Box<ErrorEnvelopeV1>> {
        let mut inner = self.inner.lock().map_err(|_| {
            Box::new(update_error(
                "update_check",
                "UPDATE_STATE_UNAVAILABLE",
                "update state lock is unavailable",
            ))
        })?;
        if matches!(
            inner.status.phase,
            UpdatePhaseV1::Checking | UpdatePhaseV1::Downloading | UpdatePhaseV1::Installing
        ) {
            return Err(Box::new(update_error(
                "update_check",
                "UPDATE_BUSY",
                "another update operation is running",
            )));
        }
        inner.pending_update = None;
        inner.downloaded = None;
        inner.status.phase = UpdatePhaseV1::Checking;
        inner.status.available_version = None;
        inner.status.release_notes = None;
        inner.status.published_at = None;
        inner.status.downloaded_bytes = 0;
        inner.status.total_bytes = None;
        inner.status.progress_percent = None;
        inner.status.verification = UpdateVerificationV1::NotStarted;
        inner.status.can_check = false;
        inner.status.can_download = false;
        inner.status.can_install = false;
        inner.status.requires_restart = false;
        inner.status.blocker_code = None;
        inner.status.blocker_message = None;
        Ok(inner.status.clone())
    }

    fn set_check_result(&self, update: Option<Update>) -> UpdateStatusV1 {
        let Ok(mut inner) = self.inner.lock() else {
            return unavailable_status("更新状态暂时不可用。");
        };
        inner.status.checked_at = Some(now_rfc3339());
        inner.status.can_check = true;
        match update {
            Some(update) => {
                inner.status.phase = UpdatePhaseV1::Available;
                inner.status.available_version = Some(update.version.clone());
                inner.status.release_notes = update
                    .body
                    .as_deref()
                    .map(|notes| truncate_chars(notes, MAX_RELEASE_NOTES_CHARS));
                inner.status.published_at = update.date.and_then(|date| date.format(&Rfc3339).ok());
                inner.status.can_download = true;
                inner.pending_update = Some(update);
            }
            None => {
                inner.status.phase = UpdatePhaseV1::UpToDate;
                inner.pending_update = None;
            }
        }
        inner.status.clone()
    }

    fn begin_download(&self) -> Result<(Update, UpdateStatusV1), Box<ErrorEnvelopeV1>> {
        let mut inner = self.inner.lock().map_err(|_| {
            Box::new(update_error(
                "update_download",
                "UPDATE_STATE_UNAVAILABLE",
                "update state lock is unavailable",
            ))
        })?;
        if matches!(
            inner.status.phase,
            UpdatePhaseV1::Checking | UpdatePhaseV1::Downloading | UpdatePhaseV1::Installing
        ) {
            return Err(Box::new(update_error(
                "update_download",
                "UPDATE_BUSY",
                "another update operation is running",
            )));
        }
        let update = inner.pending_update.clone().ok_or_else(|| {
            Box::new(update_error(
                "update_download",
                "UPDATE_NOT_AVAILABLE",
                "there is no checked update to download",
            ))
        })?;
        inner.downloaded = None;
        inner.status.phase = UpdatePhaseV1::Downloading;
        inner.status.downloaded_bytes = 0;
        inner.status.total_bytes = None;
        inner.status.progress_percent = Some(0);
        inner.status.verification = UpdateVerificationV1::NotStarted;
        inner.status.can_check = false;
        inner.status.can_download = false;
        inner.status.can_install = false;
        inner.status.blocker_code = None;
        inner.status.blocker_message = None;
        Ok((update, inner.status.clone()))
    }

    fn record_progress(&self, chunk_length: usize, total: Option<u64>) -> Option<UpdateStatusV1> {
        let Ok(mut inner) = self.inner.lock() else {
            return None;
        };
        let old_percent = inner.status.progress_percent;
        inner.status.downloaded_bytes = inner
            .status
            .downloaded_bytes
            .saturating_add(chunk_length as u64);
        if total.is_some() {
            inner.status.total_bytes = total;
        }
        inner.status.progress_percent = inner.status.total_bytes.and_then(|length| {
            (length > 0).then(|| {
                ((inner.status.downloaded_bytes.saturating_mul(100) / length).min(100)) as u8
            })
        });
        (old_percent != inner.status.progress_percent).then(|| inner.status.clone())
    }

    fn finish_download(&self, bytes: Vec<u8>) -> UpdateStatusV1 {
        let Ok(mut inner) = self.inner.lock() else {
            return unavailable_status("更新状态暂时不可用。");
        };
        inner.status.phase = UpdatePhaseV1::ReadyToInstall;
        inner.status.downloaded_bytes = bytes.len() as u64;
        inner.status.total_bytes = Some(bytes.len() as u64);
        inner.status.progress_percent = Some(100);
        inner.status.verification = UpdateVerificationV1::Verified;
        inner.status.can_check = true;
        inner.status.can_download = true;
        inner.status.can_install = true;
        inner.status.blocker_code = None;
        inner.status.blocker_message = None;
        inner.downloaded = Some(bytes);
        inner.status.clone()
    }

    fn begin_install(&self) -> Result<(Update, Vec<u8>, UpdateStatusV1), Box<ErrorEnvelopeV1>> {
        let mut inner = self.inner.lock().map_err(|_| {
            Box::new(update_error(
                "update_install",
                "UPDATE_STATE_UNAVAILABLE",
                "update state lock is unavailable",
            ))
        })?;
        let update = inner.pending_update.take().ok_or_else(|| {
            Box::new(update_error(
                "update_install",
                "UPDATE_NOT_AVAILABLE",
                "there is no checked update to install",
            ))
        })?;
        let bytes = inner.downloaded.take().ok_or_else(|| {
            inner.pending_update = Some(update.clone());
            Box::new(update_error(
                "update_install",
                "UPDATE_NOT_DOWNLOADED",
                "the signed update has not been downloaded",
            ))
        })?;
        inner.status.phase = UpdatePhaseV1::Installing;
        inner.status.can_check = false;
        inner.status.can_download = false;
        inner.status.can_install = false;
        inner.status.blocker_code = None;
        inner.status.blocker_message = None;
        Ok((update, bytes, inner.status.clone()))
    }

    fn restore_after_install_failure(
        &self,
        update: Update,
        bytes: Vec<u8>,
        code: &str,
        message: &str,
    ) -> UpdateStatusV1 {
        let Ok(mut inner) = self.inner.lock() else {
            return unavailable_status(message);
        };
        inner.pending_update = Some(update);
        inner.downloaded = Some(bytes);
        inner.status.phase = UpdatePhaseV1::Failed;
        inner.status.can_check = true;
        inner.status.can_download = true;
        inner.status.can_install = true;
        inner.status.blocker_code = Some(code.to_string());
        inner.status.blocker_message = Some(message.to_string());
        inner.status.clone()
    }

    fn set_restart_required(&self) -> UpdateStatusV1 {
        let Ok(mut inner) = self.inner.lock() else {
            return unavailable_status("更新已安装，需要重新启动助手。");
        };
        inner.status.phase = UpdatePhaseV1::RestartRequired;
        inner.status.requires_restart = true;
        inner.status.can_check = false;
        inner.status.can_download = false;
        inner.status.can_install = false;
        inner.status.clone()
    }

    fn refresh_receipt(&self) -> Result<Option<UpdateReceiptV1>, String> {
        match self.receipt_path.as_deref() {
            Some(path) => read_receipt(path),
            None => Ok(None),
        }
    }

    fn update_last_receipt(&self, receipt: &UpdateReceiptV1) -> UpdateStatusV1 {
        let Ok(mut inner) = self.inner.lock() else {
            return unavailable_status("更新状态暂时不可用。");
        };
        inner.status.last_update = Some(UpdateReceiptSummaryV1::from(receipt));
        inner.status.clone()
    }
}

#[tauri::command]
pub async fn get_assistant_update_status(
    state: State<'_, AssistantUpdaterState>,
) -> Result<UpdateStatusV1, ErrorEnvelopeV1> {
    Ok(state.snapshot())
}

#[tauri::command]
pub async fn check_for_assistant_update(
    app: AppHandle,
    state: State<'_, AssistantUpdaterState>,
) -> Result<UpdateStatusV1, ErrorEnvelopeV1> {
    let configuration = state.configuration.clone().ok_or_else(|| {
        update_error(
            "update_check",
            "UPDATE_NOT_CONFIGURED",
            state
                .configuration_error
                .as_deref()
                .unwrap_or("update endpoint and public key are not configured"),
        )
    })?;
    publish_status(&app, &state.reset_for_check().map_err(|error| *error)?);

    let updater = app
        .updater_builder()
        .endpoints(vec![configuration.endpoint])
        .map_err(|error| {
            let status = state.set_failed("UPDATE_CHECK_FAILED", error.to_string());
            publish_status(&app, &status);
            update_error("update_check", "UPDATE_CHECK_FAILED", error.to_string())
        })?
        .pubkey(configuration.pubkey)
        .header("X-Update-Channel", configuration.channel)
        .map_err(|error| {
            let status = state.set_failed("UPDATE_CHECK_FAILED", error.to_string());
            publish_status(&app, &status);
            update_error("update_check", "UPDATE_CHECK_FAILED", error.to_string())
        })?
        .timeout(UPDATE_TIMEOUT)
        .build()
        .map_err(|error| {
            let status = state.set_failed("UPDATE_CHECK_FAILED", error.to_string());
            publish_status(&app, &status);
            update_error("update_check", "UPDATE_CHECK_FAILED", error.to_string())
        })?;

    match updater.check().await {
        Ok(Some(update)) => {
            if let Err(detail) =
                validate_update_url(&update.download_url, configuration.allow_loopback_http)
            {
                let status = state.set_failed("UPDATE_CHECK_FAILED", &detail);
                publish_status(&app, &status);
                return Err(update_error("update_check", "UPDATE_CHECK_FAILED", detail));
            }
            let status = state.set_check_result(Some(update));
            publish_status(&app, &status);
            Ok(status)
        }
        Ok(None) => {
            let status = state.set_check_result(None);
            publish_status(&app, &status);
            Ok(status)
        }
        Err(error) => {
            let status = state.set_failed("UPDATE_CHECK_FAILED", error.to_string());
            publish_status(&app, &status);
            Err(update_error(
                "update_check",
                "UPDATE_CHECK_FAILED",
                error.to_string(),
            ))
        }
    }
}

#[tauri::command]
pub async fn download_assistant_update(
    app: AppHandle,
    state: State<'_, AssistantUpdaterState>,
) -> Result<UpdateStatusV1, ErrorEnvelopeV1> {
    let (update, status) = state.begin_download().map_err(|error| *error)?;
    publish_status(&app, &status);
    let progress_app = app.clone();
    let result = update
        .download(
            move |chunk_length, content_length| {
                let progress_state = progress_app.state::<AssistantUpdaterState>();
                if let Some(status) = progress_state.record_progress(chunk_length, content_length) {
                    publish_status(&progress_app, &status);
                }
            },
            || {},
        )
        .await;

    match result {
        Ok(bytes) if bytes.len() <= MAX_UPDATE_BYTES => {
            let status = state.finish_download(bytes);
            publish_status(&app, &status);
            Ok(status)
        }
        Ok(bytes) => {
            let detail = format!(
                "update package exceeds the {} byte limit: {}",
                MAX_UPDATE_BYTES,
                bytes.len()
            );
            let status = state.set_failed("UPDATE_DOWNLOAD_FAILED", &detail);
            publish_status(&app, &status);
            Err(update_error(
                "update_download",
                "UPDATE_DOWNLOAD_FAILED",
                detail,
            ))
        }
        Err(error) => {
            let detail = error.to_string();
            let code = update_download_error_code(&error);
            let status = state.set_failed(code, &detail);
            publish_status(&app, &status);
            Err(update_error("update_download", code, detail))
        }
    }
}

#[tauri::command]
pub async fn install_assistant_update(
    app: AppHandle,
    state: State<'_, AssistantUpdaterState>,
) -> Result<UpdateStatusV1, ErrorEnvelopeV1> {
    let operation = operation_gate::try_begin("assistant_update_install")
        .map_err(|error| update_error("update_install", "UPDATE_BUSY", error))?;
    let (update, bytes, status) = state.begin_install().map_err(|error| *error)?;
    let receipt = UpdateReceiptV1 {
        schema_version: RECEIPT_SCHEMA_VERSION,
        from_version: env!("CARGO_PKG_VERSION").to_string(),
        to_version: update.version.clone(),
        status: "pending_install".to_string(),
        started_at: now_rfc3339(),
        completed_at: None,
    };
    let receipt_path = match state.receipt_path.as_deref() {
        Some(path) => path,
        None => {
            let detail = "update receipt path is unavailable";
            let status =
                state.restore_after_install_failure(update, bytes, "UPDATE_RECEIPT_FAILED", detail);
            publish_status(&app, &status);
            return Err(update_error(
                "update_install",
                "UPDATE_RECEIPT_FAILED",
                detail,
            ));
        }
    };
    if let Err(error) = write_receipt(receipt_path, &receipt) {
        let status =
            state.restore_after_install_failure(update, bytes, "UPDATE_RECEIPT_FAILED", &error);
        publish_status(&app, &status);
        return Err(update_error(
            "update_install",
            "UPDATE_RECEIPT_FAILED",
            error,
        ));
    }
    state.update_last_receipt(&receipt);
    publish_status(&app, &status);

    if let Err(error) = update.install(&bytes) {
        let detail = error.to_string();
        let mut failed_receipt = receipt;
        failed_receipt.status = "install_failed".to_string();
        failed_receipt.completed_at = Some(now_rfc3339());
        let _ = write_receipt(receipt_path, &failed_receipt);
        state.update_last_receipt(&failed_receipt);
        let status =
            state.restore_after_install_failure(update, bytes, "UPDATE_INSTALL_FAILED", &detail);
        publish_status(&app, &status);
        return Err(update_error(
            "update_install",
            "UPDATE_INSTALL_FAILED",
            detail,
        ));
    }

    drop(operation);
    let status = state.set_restart_required();
    publish_status(&app, &status);
    app.restart();
}

#[tauri::command]
pub async fn confirm_assistant_update_health(
    app: AppHandle,
    state: State<'_, AssistantUpdaterState>,
) -> Result<UpdateStatusV1, ErrorEnvelopeV1> {
    let mut receipt = match state.refresh_receipt() {
        Ok(Some(receipt)) => receipt,
        Ok(None) => return Ok(state.snapshot()),
        Err(detail) => {
            let status = state.set_failed("UPDATE_RECEIPT_FAILED", &detail);
            publish_status(&app, &status);
            return Err(update_error(
                "update_health",
                "UPDATE_RECEIPT_FAILED",
                detail,
            ));
        }
    };
    if receipt.status != "pending_install" || receipt.to_version != env!("CARGO_PKG_VERSION") {
        return Ok(state.snapshot());
    }
    receipt.status = "healthy".to_string();
    receipt.completed_at = Some(now_rfc3339());
    if let Some(path) = state.receipt_path.as_deref() {
        write_receipt(path, &receipt)
            .map_err(|error| update_error("update_health", "UPDATE_RECEIPT_FAILED", error))?;
    }
    let status = state.update_last_receipt(&receipt);
    publish_status(&app, &status);
    Ok(status)
}

fn publish_status(app: &AppHandle, status: &UpdateStatusV1) {
    let _ = app.emit(UPDATE_EVENT, status.clone());
}

fn update_error(stage: &str, code: &str, detail: impl Into<String>) -> ErrorEnvelopeV1 {
    ErrorEnvelopeV1::from_code(stage, code, detail)
}

fn update_download_error_code(error: &UpdaterError) -> &'static str {
    if matches!(
        error,
        UpdaterError::Minisign(_) | UpdaterError::Base64(_) | UpdaterError::SignatureUtf8(_)
    ) {
        "UPDATE_SIGNATURE_INVALID"
    } else {
        "UPDATE_DOWNLOAD_FAILED"
    }
}

fn update_configuration() -> Result<Option<UpdateConfiguration>, String> {
    let endpoint = configured_value(COMPILED_ENDPOINT, "CODEX_ASSISTANT_UPDATE_ENDPOINT");
    let pubkey = configured_value(COMPILED_PUBKEY, "CODEX_ASSISTANT_UPDATE_PUBKEY");
    let channel = configured_value(COMPILED_CHANNEL, "CODEX_ASSISTANT_UPDATE_CHANNEL")
        .unwrap_or_else(|| DEFAULT_CHANNEL.to_string());

    match (endpoint, pubkey) {
        (None, None) => Ok(None),
        (Some(endpoint), Some(pubkey)) => validate_configuration(
            &endpoint,
            &pubkey,
            &channel,
            cfg!(debug_assertions) || cfg!(feature = "updater-mock"),
        )
        .map(Some),
        _ => Err("更新 endpoint 与 public key 必须同时配置。".to_string()),
    }
}

fn configured_value(compiled: Option<&str>, runtime_name: &str) -> Option<String> {
    if let Some(value) = compiled.map(str::trim).filter(|value| !value.is_empty()) {
        return Some(value.to_string());
    }
    #[cfg(debug_assertions)]
    {
        std::env::var(runtime_name)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = runtime_name;
        None
    }
}

fn validate_configuration(
    endpoint: &str,
    pubkey: &str,
    channel: &str,
    allow_loopback_http: bool,
) -> Result<UpdateConfiguration, String> {
    if endpoint.len() > 2_048 {
        return Err("更新 endpoint 过长。".to_string());
    }
    let endpoint = Url::parse(endpoint).map_err(|_| "更新 endpoint 格式无效。".to_string())?;
    validate_update_url(&endpoint, allow_loopback_http)
        .map_err(|_| "正式更新 endpoint 必须使用 HTTPS。".to_string())?;
    if pubkey.trim().is_empty() || pubkey.len() > 4_096 {
        return Err("更新 public key 无效。".to_string());
    }
    if !matches!(channel, "internal-test" | "beta" | "stable") {
        return Err("更新通道必须是 internal-test、beta 或 stable。".to_string());
    }
    Ok(UpdateConfiguration {
        endpoint,
        pubkey: pubkey.trim().to_string(),
        channel: channel.to_string(),
        allow_loopback_http,
    })
}

fn validate_update_url(url: &Url, allow_loopback_http: bool) -> Result<(), String> {
    if url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
        return Err("更新清单包含无效的下载地址。".to_string());
    }
    let secure = url.scheme() == "https";
    let loopback_http = url.scheme() == "http"
        && url
            .host_str()
            .is_some_and(|host| matches!(host, "127.0.0.1" | "localhost" | "::1"));
    if secure || allow_loopback_http && loopback_http {
        Ok(())
    } else {
        Err("正式更新包下载地址必须使用 HTTPS。".to_string())
    }
}

fn update_platform() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        value => value,
    }
}

fn update_architecture() -> &'static str {
    match std::env::consts::ARCH {
        "x86" => "i686",
        "arm" => "armv7",
        value => value,
    }
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut output: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        output.push('…');
    }
    output
}

fn unavailable_status(message: &str) -> UpdateStatusV1 {
    UpdateStatusV1 {
        schema_version: SCHEMA_VERSION_V1,
        phase: UpdatePhaseV1::Failed,
        current_version: env!("CARGO_PKG_VERSION").to_string(),
        channel: DEFAULT_CHANNEL.to_string(),
        platform: update_platform().to_string(),
        architecture: update_architecture().to_string(),
        configured: false,
        available_version: None,
        release_notes: None,
        published_at: None,
        checked_at: None,
        downloaded_bytes: 0,
        total_bytes: None,
        progress_percent: None,
        verification: UpdateVerificationV1::NotStarted,
        can_check: false,
        can_download: false,
        can_install: false,
        requires_restart: false,
        blocker_code: Some("UPDATE_STATE_UNAVAILABLE".to_string()),
        blocker_message: Some(message.to_string()),
        last_update: None,
    }
}

fn read_receipt(path: &Path) -> Result<Option<UpdateReceiptV1>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(path).map_err(|error| format!("读取更新收据失败: {error}"))?;
    let receipt: UpdateReceiptV1 =
        serde_json::from_slice(&bytes).map_err(|error| format!("更新收据格式无效: {error}"))?;
    if receipt.schema_version != RECEIPT_SCHEMA_VERSION {
        return Err("更新收据版本不受支持。".to_string());
    }
    Ok(Some(receipt))
}

fn write_receipt(path: &Path, receipt: &UpdateReceiptV1) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "更新收据路径无效。".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("创建更新状态目录失败: {error}"))?;
    let temporary = parent.join(format!(".update-state-{}.tmp", Uuid::new_v4()));
    let bytes =
        serde_json::to_vec_pretty(receipt).map_err(|error| format!("生成更新收据失败: {error}"))?;
    let mut file =
        fs::File::create(&temporary).map_err(|error| format!("创建更新收据失败: {error}"))?;
    let write_result = file
        .write_all(&bytes)
        .and_then(|_| file.write_all(b"\n"))
        .and_then(|_| file.sync_all());
    drop(file);
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temporary);
        return Err(format!("写入更新收据失败: {error}"));
    }
    replace_file(&temporary, path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("提交更新收据失败: {error}")
    })
}

#[cfg(target_os = "windows")]
fn replace_file(source: &Path, target: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };
    let source: Vec<u16> = source
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let target: Vec<u16> = target
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let result = unsafe {
        MoveFileExW(
            source.as_ptr(),
            target.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn replace_file(source: &Path, target: &Path) -> Result<(), String> {
    fs::rename(source, target).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_configuration_requires_https() {
        let error = validate_configuration(
            "http://updates.example.test/check",
            "public-key",
            "stable",
            false,
        )
        .expect_err("plain HTTP should be rejected");
        assert!(error.contains("HTTPS"));
    }

    #[test]
    fn mock_configuration_only_allows_loopback_http() {
        let local = validate_configuration(
            "http://127.0.0.1:4317/{{target}}/{{arch}}/{{current_version}}",
            "public-key",
            "internal-test",
            true,
        )
        .expect("loopback mock endpoint should be allowed");
        assert_eq!(local.channel, "internal-test");

        let remote = validate_configuration(
            "http://192.168.50.130:4317/check",
            "public-key",
            "internal-test",
            true,
        )
        .expect_err("LAN HTTP endpoint should remain blocked");
        assert!(remote.contains("HTTPS"));
    }

    #[test]
    fn production_download_url_requires_https_without_credentials() {
        validate_update_url(
            &Url::parse("https://downloads.example.test/codex-assistant.exe").unwrap(),
            false,
        )
        .expect("HTTPS download URL should be allowed");
        let plain_http = Url::parse("http://downloads.example.test/codex-assistant.exe").unwrap();
        assert!(validate_update_url(&plain_http, false).is_err());
        let credentials =
            Url::parse("https://token@downloads.example.test/codex-assistant.exe").unwrap();
        assert!(validate_update_url(&credentials, false).is_err());
    }

    #[test]
    fn mock_download_url_only_allows_loopback_http() {
        validate_update_url(
            &Url::parse("http://localhost:4317/codex-assistant.exe").unwrap(),
            true,
        )
        .expect("loopback mock download URL should be allowed");
        let lan = Url::parse("http://192.168.50.130:4317/codex-assistant.exe").unwrap();
        assert!(validate_update_url(&lan, true).is_err());
    }

    #[test]
    fn configuration_requires_known_channel() {
        let error = validate_configuration(
            "https://updates.example.test/check",
            "public-key",
            "customer",
            false,
        )
        .expect_err("unknown channel should be rejected");
        assert!(error.contains("internal-test"));
    }

    #[test]
    fn receipt_round_trip_is_versioned() {
        let root = std::env::temp_dir().join(format!("codex-updater-{}", Uuid::new_v4()));
        let path = root.join("update-state.json");
        let receipt = UpdateReceiptV1 {
            schema_version: RECEIPT_SCHEMA_VERSION,
            from_version: "0.9.0".to_string(),
            to_version: "0.9.1".to_string(),
            status: "pending_install".to_string(),
            started_at: "2026-07-30T00:00:00Z".to_string(),
            completed_at: None,
        };
        write_receipt(&path, &receipt).expect("receipt should be written");
        let restored = read_receipt(&path)
            .expect("receipt should be readable")
            .expect("receipt should exist");
        assert_eq!(restored.from_version, "0.9.0");
        assert_eq!(restored.to_version, "0.9.1");
        assert_eq!(restored.status, "pending_install");
        fs::remove_dir_all(root).expect("temporary receipt directory should be removed");
    }

    #[test]
    fn release_notes_are_bounded() {
        let notes = "a".repeat(MAX_RELEASE_NOTES_CHARS + 20);
        let truncated = truncate_chars(&notes, MAX_RELEASE_NOTES_CHARS);
        assert_eq!(truncated.chars().count(), MAX_RELEASE_NOTES_CHARS + 1);
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn malformed_signature_has_a_stable_error_code() {
        let error = UpdaterError::SignatureUtf8("invalid".to_string());
        assert_eq!(
            update_download_error_code(&error),
            "UPDATE_SIGNATURE_INVALID"
        );
    }
}
