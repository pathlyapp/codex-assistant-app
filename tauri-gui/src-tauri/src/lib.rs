use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    env, fs,
    io::{BufRead, BufReader},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};
use toml_edit::{value, Array, DocumentMut, Item, Table};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

use std::io::{Cursor, Read};

mod token_support;

const VERSION: &str = "0.8.8";
const CONFIG_START: &str = "# >>> CodexAssistant Managed Config";
const CONFIG_END: &str = "# <<< CodexAssistant Managed Config";
const LEGACY_CONFIG_START: &str = "# >>> CompanyCodex Gateway PoC";
const LEGACY_CONFIG_END: &str = "# <<< CompanyCodex Gateway PoC";
const DEFAULT_GATEWAY: &str = "http://127.0.0.1:11434/v1";
const PROVIDER_ID: &str = "codex_assistant_router";
const PROVIDER_NAME: &str = "Codex Assistant Router";
const WINDOWS_STORE_ID: &str = "9PLM9XGG6VKS";
const APPEARANCE_PORT: u16 = 9335;
const MAX_THEME_IMAGE_BYTES: usize = 8 * 1024 * 1024;
const MAX_THEME_IMAGE_DIMENSION: u32 = 16_384;
const MAX_THEME_IMAGE_PIXELS: u64 = 50_000_000;
const GALLERY_API_BASE: &str = "https://api.dreamskin.cc";
const GALLERY_LIST_LIMIT: usize = 24;
const MAX_GALLERY_PACKAGE_BYTES: u64 = 48 * 1024 * 1024;
const MAX_GALLERY_ZIP_ENTRIES: usize = 64;
const MAX_GALLERY_STORED: usize = 12;
const WINDOWS_SETUP_PROBE: &str = r#"(() => {
  const text = document.body?.innerText || '';
  return {
    pending: /Finish Windows setup/i.test(text),
    readyState: document.readyState,
    textLength: text.length
  };
})()"#;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetupOptions {
    gateway: String,
    #[serde(default)]
    model: String,
    #[serde(default)]
    key: String,
    #[serde(default)]
    no_auth: bool,
    #[serde(default = "default_true")]
    install_chatgpt: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GatewayProbeRequest {
    gateway: String,
    #[serde(default)]
    key: String,
    #[serde(default)]
    use_saved_key: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelDiscovery {
    gateway: String,
    models: Vec<String>,
    used_saved_key: bool,
    message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemStatus {
    platform: String,
    architecture: String,
    app_installed: bool,
    app_name: String,
    app_version: Option<String>,
    app_detail: String,
    config_present: bool,
    config_path: String,
    router_reachable: bool,
    router_detail: String,
    configured_gateway: Option<String>,
    configured_model: Option<String>,
    key_configured: bool,
    backup_available: bool,
    ready: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RestoreResult {
    restored_from: String,
    message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResetStepResult {
    id: String,
    label: String,
    status: String,
    message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FactoryResetResult {
    success: bool,
    summary: String,
    steps: Vec<ResetStepResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfigurationSnapshot {
    version: String,
    timestamp: String,
    codex_config_existed: bool,
    state_existed: bool,
    models_existed: bool,
    key_existed: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppearanceStatus {
    supported: bool,
    selected_theme: String,
    active: bool,
    requires_restart: bool,
    custom_theme_ready: bool,
    custom_theme_name: Option<String>,
    message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppearanceState {
    version: String,
    selected_theme: String,
    port: u16,
    applied_at: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThemeImageRequest {
    file_name: String,
    mime_type: String,
    data_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThemeImageInfo {
    file_name: String,
    mime_type: String,
    stored_file: String,
    width: u32,
    height: u32,
    imported_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct ArtColors {
    background: String,
    panel: String,
    panel_alt: String,
    accent: String,
    highlight: String,
    text: String,
    muted: String,
    line: String,
}

impl Default for ArtColors {
    fn default() -> Self {
        Self {
            background: "#0b1118".to_string(),
            panel: "#151c25".to_string(),
            panel_alt: "#202a35".to_string(),
            accent: "#8095a5".to_string(),
            highlight: "#a4b4c0".to_string(),
            text: "#f4f7fb".to_string(),
            muted: "#9aa5ad".to_string(),
            line: "#3f3f3f".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct ArtThemeMeta {
    name: String,
    author: String,
    license: String,
    appearance: String,
    focus_x: f64,
    focus_y: f64,
    colors: ArtColors,
    stored_file: String,
    mime: String,
}

impl Default for ArtThemeMeta {
    fn default() -> Self {
        Self {
            name: String::new(),
            author: String::new(),
            license: String::new(),
            appearance: "dark".to_string(),
            focus_x: 0.5,
            focus_y: 0.5,
            colors: ArtColors::default(),
            stored_file: String::new(),
            mime: "image/jpeg".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PresetThemeInfo {
    id: String,
    name: String,
    author: String,
    license: String,
    appearance: String,
    colors: ArtColors,
    preview_data_url: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GalleryThemeInfo {
    version_id: String,
    theme_id: String,
    name: String,
    author: String,
    license: String,
    downloads: u64,
    package_bytes: u64,
    appearance: Option<String>,
    colors: Option<ArtColors>,
    downloaded: bool,
}

struct PreparedTheme {
    attr: String,
    css: String,
    image_data_url: Option<String>,
}

struct ResolvedArt {
    attr: String,
    meta: ArtThemeMeta,
    bytes: Vec<u8>,
}

struct ArtPreset {
    slug: &'static str,
    name: &'static str,
    author: &'static str,
    license: &'static str,
    appearance: &'static str,
    focus_x: f64,
    focus_y: f64,
    mime: &'static str,
    bytes: &'static [u8],
    colors: ArtPalette,
}

#[derive(Clone, Copy)]
struct ArtPalette {
    background: &'static str,
    panel: &'static str,
    panel_alt: &'static str,
    accent: &'static str,
    highlight: &'static str,
    text: &'static str,
    muted: &'static str,
    line: &'static str,
}

impl From<ArtPalette> for ArtColors {
    fn from(palette: ArtPalette) -> Self {
        Self {
            background: palette.background.to_string(),
            panel: palette.panel.to_string(),
            panel_alt: palette.panel_alt.to_string(),
            accent: palette.accent.to_string(),
            highlight: palette.highlight.to_string(),
            text: palette.text.to_string(),
            muted: palette.muted.to_string(),
            line: palette.line.to_string(),
        }
    }
}

static ART_PRESETS: &[ArtPreset] = &[
    ArtPreset {
        slug: "wukong",
        name: "悟空 WUKONG",
        author: "JamesOpsLab",
        license: "MIT",
        appearance: "dark",
        focus_x: 0.0,
        focus_y: 0.5,
        mime: "image/webp",
        bytes: include_bytes!("../presets/wukong.webp"),
        colors: ArtPalette {
            background: "#131313",
            panel: "#1d1e1d",
            panel_alt: "#2a2a2a",
            accent: "#f6c696",
            highlight: "#f8d4b0",
            text: "#f0f0f0",
            muted: "#939393",
            line: "#3f3f3f",
        },
    },
    ArtPreset {
        slug: "firefly",
        name: "firefly",
        author: "1xifengdeyouxi",
        license: "MIT",
        appearance: "light",
        focus_x: 0.37,
        focus_y: 0.5,
        mime: "image/jpeg",
        bytes: include_bytes!("../presets/firefly.jpg"),
        colors: ArtPalette {
            background: "#2d507f",
            panel: "#c4d8da",
            panel_alt: "#e9eaea",
            accent: "#9b850c",
            highlight: "#746409",
            text: "#1c1c1d",
            muted: "#696969",
            line: "#d3d3d3",
        },
    },
    ArtPreset {
        slug: "vault-office",
        name: "保险柜 办公室 卡通",
        author: "陆健辉",
        license: "CC BY 4.0",
        appearance: "dark",
        focus_x: 0.28,
        focus_y: 0.5,
        mime: "image/jpeg",
        bytes: include_bytes!("../presets/vault-office.jpg"),
        colors: ArtPalette {
            background: "#131313",
            panel: "#1e1e1d",
            panel_alt: "#2b2b2a",
            accent: "#d04f37",
            highlight: "#dc7b69",
            text: "#f0f0ef",
            muted: "#939393",
            line: "#3f3f3f",
        },
    },
];

#[derive(Clone, Debug, Serialize)]
struct InstallerFinished {
    success: bool,
    code: Option<i32>,
    summary: String,
    stages: Vec<StageEvent>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StageEvent {
    stage: String,
    label: String,
    status: String,
    message: String,
    current: usize,
    total: usize,
    recoverable: bool,
    details: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallState {
    version: String,
    provider_id: String,
    provider_display_name: String,
    model: String,
    gateway_base_url: String,
    token_mode: String,
    wire_api: String,
    #[serde(default)]
    available_models: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    key_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    secret_storage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    model_catalog_path: Option<String>,
    installed_at: String,
}

#[derive(Clone, Debug)]
struct InstallerPaths {
    install_root: PathBuf,
    codex_config_path: PathBuf,
}

#[derive(Clone, Debug)]
struct DesktopAppInfo {
    installed: bool,
    name: String,
    version: Option<String>,
    detail: String,
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    package_family_name: Option<String>,
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    app_id: Option<String>,
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    executable_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CdpTarget {
    #[serde(rename = "type")]
    target_type: String,
    url: String,
    #[serde(default)]
    web_socket_debugger_url: String,
}

#[derive(Clone, Debug)]
struct TokenPrep {
    token_mode: String,
    key_path: Option<PathBuf>,
    secret_storage: Option<String>,
}

#[derive(Clone, Debug)]
struct TokenHelperCommand {
    command: String,
    args: Vec<String>,
}

#[derive(Clone, Debug)]
struct InstallContext {
    options: SetupOptions,
    models: Vec<String>,
}

#[derive(Clone, Debug)]
struct StageOutcome {
    status: &'static str,
    message: String,
    details: serde_json::Value,
}

impl StageOutcome {
    fn complete(message: impl Into<String>) -> Self {
        Self {
            status: "complete",
            message: message.into(),
            details: json!({}),
        }
    }

    fn skipped(message: impl Into<String>) -> Self {
        Self {
            status: "skipped",
            message: message.into(),
            details: json!({}),
        }
    }

    fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }
}

type StageRunner = fn(&AppHandle, &mut InstallContext) -> Result<StageOutcome, String>;

fn default_true() -> bool {
    true
}

#[tauri::command]
async fn get_system_status() -> Result<SystemStatus, String> {
    tauri::async_runtime::spawn_blocking(collect_system_status)
        .await
        .map_err(|error| format!("读取状态失败: {error}"))?
}

#[tauri::command]
async fn discover_models(request: GatewayProbeRequest) -> Result<ModelDiscovery, String> {
    tauri::async_runtime::spawn_blocking(move || discover_models_inner(request))
        .await
        .map_err(|error| format!("模型检测任务失败: {error}"))?
}

#[tauri::command]
async fn start_setup(app: AppHandle, options: SetupOptions) -> Result<InstallerFinished, String> {
    let options = resolve_options(options)?;
    tauri::async_runtime::spawn_blocking(move || run_setup(app, options))
        .await
        .map_err(|error| format!("配置任务失败: {error}"))
}

#[tauri::command]
async fn install_chatgpt_app(app: AppHandle) -> Result<SystemStatus, String> {
    tauri::async_runtime::spawn_blocking(move || install_chatgpt_app_inner(&app))
        .await
        .map_err(|error| format!("安装任务失败: {error}"))?
}

#[tauri::command]
async fn launch_chatgpt() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(launch_chatgpt_preferred)
        .await
        .map_err(|error| format!("启动任务失败: {error}"))?
}

#[tauri::command]
async fn restart_chatgpt() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(restart_chatgpt_inner)
        .await
        .map_err(|error| format!("重启任务失败: {error}"))?
}

#[tauri::command]
async fn restore_codex_config() -> Result<RestoreResult, String> {
    tauri::async_runtime::spawn_blocking(restore_codex_config_inner)
        .await
        .map_err(|error| format!("恢复任务失败: {error}"))?
}

#[tauri::command]
async fn factory_reset() -> Result<FactoryResetResult, String> {
    tauri::async_runtime::spawn_blocking(factory_reset_inner)
        .await
        .map_err(|error| format!("还原任务失败: {error}"))?
}

#[tauri::command]
async fn get_appearance_status() -> Result<AppearanceStatus, String> {
    tauri::async_runtime::spawn_blocking(collect_appearance_status)
        .await
        .map_err(|error| format!("读取外观状态失败: {error}"))?
}

#[tauri::command]
async fn apply_appearance(theme: String) -> Result<AppearanceStatus, String> {
    tauri::async_runtime::spawn_blocking(move || apply_appearance_inner(&theme))
        .await
        .map_err(|error| format!("应用外观任务失败: {error}"))?
}

#[tauri::command]
async fn import_theme_image(request: ThemeImageRequest) -> Result<AppearanceStatus, String> {
    tauri::async_runtime::spawn_blocking(move || import_theme_image_inner(&request))
        .await
        .map_err(|error| format!("导入主题图片任务失败: {error}"))?
}

#[tauri::command]
async fn list_preset_themes() -> Result<Vec<PresetThemeInfo>, String> {
    tauri::async_runtime::spawn_blocking(list_preset_themes_inner)
        .await
        .map_err(|error| format!("读取内置主题任务失败: {error}"))?
}

#[tauri::command]
async fn list_gallery_themes() -> Result<Vec<GalleryThemeInfo>, String> {
    tauri::async_runtime::spawn_blocking(list_gallery_themes_inner)
        .await
        .map_err(|error| format!("读取在线主题库任务失败: {error}"))?
}

fn collect_system_status() -> Result<SystemStatus, String> {
    let paths = resolve_paths()?;
    let app = detect_chatgpt_app()?;
    let state = read_state(&paths).ok();
    let config_present = state
        .as_ref()
        .map(|saved| codex_config_matches(&paths.codex_config_path, saved))
        .unwrap_or(false);

    let (router_reachable, router_detail) = match &state {
        Some(saved) => match gateway_bearer_from_state(saved)
            .and_then(|bearer| fetch_models(&saved.gateway_base_url, bearer.as_deref()))
        {
            Ok(models) => (true, format!("服务可用，发现 {} 个模型", models.len())),
            Err(error) => (false, friendly_error(&error)),
        },
        None => (false, "尚未配置 Router".to_string()),
    };

    let backup_available = latest_configuration_snapshot(&paths)?.is_some();
    let ready = app.installed && config_present && router_reachable;
    Ok(SystemStatus {
        platform: platform_name().to_string(),
        architecture: std::env::consts::ARCH.to_string(),
        app_installed: app.installed,
        app_name: app.name,
        app_version: app.version,
        app_detail: app.detail,
        config_present,
        config_path: paths.codex_config_path.to_string_lossy().to_string(),
        router_reachable,
        router_detail,
        configured_gateway: state.as_ref().map(|saved| saved.gateway_base_url.clone()),
        configured_model: state.as_ref().map(|saved| saved.model.clone()),
        key_configured: state
            .as_ref()
            .map(|saved| saved.token_mode != "none")
            .unwrap_or(false),
        backup_available,
        ready,
    })
}

fn discover_models_inner(request: GatewayProbeRequest) -> Result<ModelDiscovery, String> {
    let gateway = normalize_gateway(&request.gateway)?;
    let paths = resolve_paths()?;
    let saved = read_state(&paths).ok();
    let mut used_saved_key = false;
    let bearer = if !request.key.trim().is_empty() {
        Some(request.key.trim().to_string())
    } else if request.use_saved_key {
        match saved {
            Some(state) if state.gateway_base_url == gateway => {
                used_saved_key = state.token_mode != "none";
                gateway_bearer_from_state(&state)?
            }
            _ => None,
        }
    } else {
        None
    };
    let models = fetch_models(&gateway, bearer.as_deref())?;
    Ok(ModelDiscovery {
        gateway,
        message: format!("连接成功，发现 {} 个模型", models.len()),
        models,
        used_saved_key,
    })
}

fn run_setup(app: AppHandle, options: SetupOptions) -> InstallerFinished {
    let stages: [(&str, &str, StageRunner); 5] = [
        ("preflight", "检查本机环境", preflight_setup),
        ("install_chatgpt", "准备 ChatGPT", install_chatgpt),
        ("validate_router", "验证 Router", validate_router),
        ("configure_codex", "写入 Codex 配置", configure_provider),
        ("verify", "复核配置", verify_setup),
    ];
    let mut ctx = InstallContext {
        options,
        models: Vec::new(),
    };
    let mut results = Vec::new();
    let mut success = true;
    let mut summary = "ChatGPT 与 Codex Router 已配置完成".to_string();

    for (index, (stage, label, runner)) in stages.iter().enumerate() {
        emit_stage(
            &app,
            stage,
            label,
            "running",
            format!("正在{label}"),
            index + 1,
            stages.len(),
            false,
            json!({}),
        );
        emit_log(&app, format!("[{}/{}] {label}\n", index + 1, stages.len()));
        match runner(&app, &mut ctx) {
            Ok(outcome) => {
                let event = stage_event(
                    *stage,
                    *label,
                    outcome.status,
                    outcome.message,
                    index + 1,
                    stages.len(),
                    false,
                    outcome.details,
                );
                emit_stage_event(&app, &event);
                results.push(event);
            }
            Err(error) => {
                emit_log(&app, format!("[FAIL] {}\n", redact_error(&error)));
                let event = stage_event(
                    *stage,
                    *label,
                    "failed",
                    friendly_error(&error),
                    index + 1,
                    stages.len(),
                    true,
                    json!({ "error": friendly_error(&error) }),
                );
                emit_stage_event(&app, &event);
                results.push(event);
                success = false;
                summary = format!("{label}失败");
                break;
            }
        }
    }

    let finished = InstallerFinished {
        success,
        code: Some(if success { 0 } else { 1 }),
        summary,
        stages: results,
    };
    let _ = app.emit("installer-finished", finished.clone());
    finished
}

fn resolve_options(mut options: SetupOptions) -> Result<SetupOptions, String> {
    options.gateway = normalize_gateway(&options.gateway)?;
    options.model = options.model.trim().to_string();
    options.key = options.key.trim().to_string();
    if options.key.len() > 16 * 1024 {
        return Err("Access Key 长度异常".to_string());
    }
    Ok(options)
}

fn preflight_setup(app: &AppHandle, ctx: &mut InstallContext) -> Result<StageOutcome, String> {
    let paths = resolve_paths()?;
    fs::create_dir_all(&paths.install_root)
        .map_err(|error| format!("创建助手数据目录失败: {error}"))?;
    if let Some(parent) = paths.codex_config_path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("创建 Codex 配置目录失败: {error}"))?;
    }
    emit_log(
        app,
        format!("[OK] Codex config: {}\n", paths.codex_config_path.display()),
    );

    let detected = detect_chatgpt_app()?;
    if detected.installed {
        emit_log(app, format!("[OK] {}\n", detected.detail));
    } else if ctx.options.install_chatgpt && cfg!(target_os = "windows") {
        let winget = resolve_winget()?;
        emit_log(
            app,
            format!("[OK] Official installer available: {winget}\n"),
        );
    } else if ctx.options.install_chatgpt {
        return Err("未检测到 ChatGPT。当前平台请先通过 OpenAI 官方渠道安装 ChatGPT".to_string());
    } else {
        return Err("未检测到 ChatGPT，且未允许安装官方应用".to_string());
    }
    Ok(StageOutcome::complete("环境检查通过"))
}

fn install_chatgpt_app_inner(app: &AppHandle) -> Result<SystemStatus, String> {
    if detect_chatgpt_app()?.installed {
        return collect_system_status();
    }
    if !cfg!(target_os = "windows") {
        return Err("当前平台请先通过 OpenAI 官方渠道安装 ChatGPT，再返回助手刷新状态".to_string());
    }
    install_chatgpt_with_winget(app)?;
    if !detect_chatgpt_app()?.installed {
        return Err("官方安装命令已结束，但系统仍未检测到 ChatGPT".to_string());
    }
    collect_system_status()
}

fn install_chatgpt(app: &AppHandle, ctx: &mut InstallContext) -> Result<StageOutcome, String> {
    let detected = detect_chatgpt_app()?;
    if detected.installed {
        return Ok(StageOutcome::skipped("已安装官方 ChatGPT，无需重复安装")
            .with_details(json!({ "version": detected.version, "detail": detected.detail })));
    }
    if !ctx.options.install_chatgpt {
        return Err("ChatGPT 尚未安装".to_string());
    }
    if !cfg!(target_os = "windows") {
        return Err("当前版本仅在 Windows 上支持自动安装 ChatGPT".to_string());
    }

    install_chatgpt_with_winget(app)?;
    let installed = detect_chatgpt_app()?;
    if !installed.installed {
        return Err("官方安装命令已结束，但系统仍未检测到 ChatGPT".to_string());
    }
    Ok(
        StageOutcome::complete("官方 ChatGPT 已安装").with_details(json!({
            "version": installed.version,
            "detail": installed.detail,
        })),
    )
}

fn validate_router(app: &AppHandle, ctx: &mut InstallContext) -> Result<StageOutcome, String> {
    let paths = resolve_paths()?;
    let saved = read_state(&paths).ok();
    let bearer = if ctx.options.no_auth {
        None
    } else if !ctx.options.key.is_empty() {
        Some(ctx.options.key.clone())
    } else {
        match saved {
            Some(state) if state.gateway_base_url == ctx.options.gateway => {
                gateway_bearer_from_state(&state)?
            }
            _ => return Err("请输入 Access Key，或选择“此 Router 无需 Key”".to_string()),
        }
    };

    let models = fetch_models(&ctx.options.gateway, bearer.as_deref())?;
    if ctx.options.model.is_empty() {
        ctx.options.model = models[0].clone();
    } else if !models.iter().any(|model| model == &ctx.options.model) {
        return Err(format!(
            "Router 未返回模型“{}”，请重新检测并选择可用模型",
            ctx.options.model
        ));
    }
    emit_log(app, format!("[OK] Router: {}\n", ctx.options.gateway));
    emit_log(app, format!("[OK] Model: {}\n", ctx.options.model));
    emit_log(app, format!("[OK] Available models: {}\n", models.len()));
    ctx.models = models;
    Ok(StageOutcome::complete(format!("Router 可用，已选择 {}", ctx.options.model)).with_details(
        json!({ "gateway": ctx.options.gateway, "model": ctx.options.model, "modelCount": ctx.models.len() }),
    ))
}

fn configure_provider(app: &AppHandle, ctx: &mut InstallContext) -> Result<StageOutcome, String> {
    let paths = resolve_paths()?;
    create_configuration_snapshot(&paths)?;
    let existing = read_state(&paths).ok();
    let token = prepare_token(&paths, &ctx.options, existing.as_ref())?;
    let mut state = InstallState {
        version: VERSION.to_string(),
        provider_id: PROVIDER_ID.to_string(),
        provider_display_name: PROVIDER_NAME.to_string(),
        model: ctx.options.model.clone(),
        gateway_base_url: ctx.options.gateway.clone(),
        token_mode: token.token_mode,
        wire_api: "responses".to_string(),
        available_models: ctx.models.clone(),
        key_path: token
            .key_path
            .map(|path| path.to_string_lossy().to_string()),
        secret_storage: token.secret_storage,
        model_catalog_path: None,
        installed_at: unix_timestamp(),
    };
    let catalog = write_model_catalog(&paths, &state)?;
    state.model_catalog_path = Some(catalog.to_string_lossy().to_string());
    write_state(&paths, &state)?;
    let helper = if state.token_mode == "none" {
        None
    } else {
        Some(token_helper_command(&paths)?)
    };
    write_codex_config(&paths.codex_config_path, &state, helper.as_ref())?;
    emit_log(app, "[OK] Codex provider configuration written\n");
    emit_log(app, format!("[OK] Provider: {}\n", state.provider_id));
    emit_log(app, format!("[OK] Wire API: {}\n", state.wire_api));
    Ok(
        StageOutcome::complete("Codex Router 配置已安全写入").with_details(json!({
            "configPath": paths.codex_config_path,
            "provider": state.provider_id,
            "model": state.model,
            "keyProtected": state.token_mode != "none",
        })),
    )
}

fn verify_setup(app: &AppHandle, _ctx: &mut InstallContext) -> Result<StageOutcome, String> {
    let paths = resolve_paths()?;
    let state = read_state(&paths).map_err(|error| format!("读取助手状态失败: {error}"))?;
    if !codex_config_matches(&paths.codex_config_path, &state) {
        return Err("Codex 配置复核失败，写入内容不完整".to_string());
    }
    let bearer = gateway_bearer_from_state(&state)?;
    let models = fetch_models(&state.gateway_base_url, bearer.as_deref())?;
    if !models.iter().any(|model| model == &state.model) {
        return Err("配置的模型已不在 Router 模型列表中".to_string());
    }
    let app_info = detect_chatgpt_app()?;
    if !app_info.installed {
        return Err("配置复核时未检测到 ChatGPT".to_string());
    }
    emit_log(app, "[OK] ChatGPT package verified\n");
    emit_log(app, "[OK] Codex config verified\n");
    emit_log(app, "[OK] Router connection verified\n");
    Ok(
        StageOutcome::complete("全部检查通过，可以打开 ChatGPT").with_details(json!({
            "app": app_info.name,
            "gateway": state.gateway_base_url,
            "model": state.model,
        })),
    )
}

fn install_chatgpt_with_winget(app: &AppHandle) -> Result<(), String> {
    let winget = resolve_winget()?;
    let args = [
        "install",
        "--id",
        WINDOWS_STORE_ID,
        "-e",
        "-s",
        "msstore",
        "--accept-source-agreements",
        "--accept-package-agreements",
    ];
    emit_log(
        app,
        "正在调用 Microsoft Store 官方安装渠道，此过程可能出现系统确认窗口。\n",
    );
    run_command_stream(
        app,
        "winget install ChatGPT",
        &winget,
        &args,
        Duration::from_secs(12 * 60),
        Duration::from_secs(15),
    )?;
    if wait_for_chatgpt(Duration::from_secs(90)) {
        Ok(())
    } else {
        Err("winget 已结束，但 90 秒内未检测到 ChatGPT 安装包".to_string())
    }
}

fn wait_for_chatgpt(timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if detect_chatgpt_app()
            .map(|app| app.installed)
            .unwrap_or(false)
        {
            return true;
        }
        thread::sleep(Duration::from_secs(2));
    }
    false
}

fn detect_chatgpt_app() -> Result<DesktopAppInfo, String> {
    #[cfg(target_os = "windows")]
    {
        detect_chatgpt_windows()
    }
    #[cfg(target_os = "macos")]
    {
        for path in [
            PathBuf::from("/Applications/ChatGPT.app"),
            user_home_dir()?.join("Applications").join("ChatGPT.app"),
            PathBuf::from("/Applications/Codex.app"),
        ] {
            if path.is_dir() {
                let executable_name = path
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("ChatGPT");
                return Ok(DesktopAppInfo {
                    installed: true,
                    name: "ChatGPT".to_string(),
                    version: None,
                    detail: format!("已安装：{}", path.display()),
                    package_family_name: None,
                    app_id: None,
                    executable_path: Some(
                        path.join("Contents")
                            .join("MacOS")
                            .join(executable_name)
                            .to_string_lossy()
                            .to_string(),
                    ),
                });
            }
        }
        Ok(DesktopAppInfo {
            installed: false,
            name: "ChatGPT".to_string(),
            version: None,
            detail: "未检测到官方 ChatGPT 应用".to_string(),
            package_family_name: None,
            app_id: None,
            executable_path: None,
        })
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    Ok(DesktopAppInfo {
        installed: false,
        name: "ChatGPT".to_string(),
        version: None,
        detail: "当前平台不支持桌面应用检测".to_string(),
        package_family_name: None,
        app_id: None,
        executable_path: None,
    })
}

#[cfg(target_os = "windows")]
fn detect_chatgpt_windows() -> Result<DesktopAppInfo, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
$pkg = Get-AppxPackage -Name 'OpenAI.Codex' | Sort-Object Version -Descending | Select-Object -First 1
if (-not $pkg) {
  $pkg = Get-AppxPackage | Where-Object { $_.PackageFamilyName -like 'OpenAI.Codex_*' } | Sort-Object Version -Descending | Select-Object -First 1
}
if ($pkg) {
  $appId = ''
  $exe = ''
  $manifestPath = Join-Path $pkg.InstallLocation 'AppxManifest.xml'
  if (Test-Path $manifestPath) {
    [xml]$manifest = Get-Content -LiteralPath $manifestPath
    $app = $manifest.Package.Applications.Application | Where-Object { $_.Executable -match '(ChatGPT|Codex)\.exe$' } | Select-Object -First 1
    if (-not $app) { $app = $manifest.Package.Applications.Application | Select-Object -First 1 }
    if ($app) {
      $appId = '' + $app.Id
      $exe = Join-Path $pkg.InstallLocation ('' + $app.Executable)
    }
  }
  @($pkg.Name, $pkg.PackageFamilyName, $pkg.Version, $pkg.InstallLocation, $appId, $exe) -join "`t"
}
"#;
    let output = run_command_capture(
        "powershell.exe",
        &["-NoProfile", "-NonInteractive", "-Command", script],
        Duration::from_secs(10),
    )?;
    let line = output.lines().find(|line| !line.trim().is_empty());
    let Some(line) = line else {
        return Ok(DesktopAppInfo {
            installed: false,
            name: "ChatGPT".to_string(),
            version: None,
            detail: "未检测到 Microsoft Store 官方 ChatGPT".to_string(),
            package_family_name: None,
            app_id: None,
            executable_path: None,
        });
    };
    let fields = line.split('\t').collect::<Vec<_>>();
    let version = fields
        .get(2)
        .map(|value| value.to_string())
        .filter(|value| !value.is_empty());
    let package_family_name = fields
        .get(1)
        .map(|value| value.to_string())
        .filter(|value| !value.is_empty());
    let app_id = fields
        .get(4)
        .map(|value| value.to_string())
        .filter(|value| !value.is_empty());
    let executable_ok = fields
        .get(5)
        .map(|path| Path::new(path).is_file())
        .unwrap_or(false);
    Ok(DesktopAppInfo {
        installed: package_family_name.is_some() && executable_ok,
        name: "ChatGPT".to_string(),
        version: version.clone(),
        detail: if executable_ok {
            format!(
                "Microsoft Store 官方应用{}",
                version.map(|v| format!(" · {v}")).unwrap_or_default()
            )
        } else {
            "检测到软件包，但 ChatGPT 程序文件不完整".to_string()
        },
        package_family_name,
        app_id,
        executable_path: fields
            .get(5)
            .map(|value| value.to_string())
            .filter(|value| !value.is_empty()),
    })
}

fn launch_chatgpt_inner() -> Result<(), String> {
    let app = detect_chatgpt_app()?;
    if !app.installed {
        return Err("未检测到 ChatGPT".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        let pfn = app
            .package_family_name
            .ok_or_else(|| "缺少 ChatGPT PackageFamilyName".to_string())?;
        let app_id = app.app_id.ok_or_else(|| "缺少 ChatGPT AppId".to_string())?;
        Command::new("explorer.exe")
            .arg(format!("shell:AppsFolder\\{pfn}!{app_id}"))
            .spawn()
            .map_err(|error| format!("启动 ChatGPT 失败: {error}"))?;
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-a", "ChatGPT"])
            .spawn()
            .map_err(|error| format!("启动 ChatGPT 失败: {error}"))?;
        return Ok(());
    }
    #[allow(unreachable_code)]
    Err("当前平台不支持启动 ChatGPT".to_string())
}

fn launch_chatgpt_preferred() -> Result<(), String> {
    let paths = resolve_paths()?;
    let appearance = read_appearance_state(&paths).ok();
    if let Some(state) = appearance.as_ref() {
        let theme = state.selected_theme.as_str();
        if matches!(theme, "focus" | "custom") || is_art_theme(theme) {
            return apply_appearance_with_download(theme, false).map(|_| ());
        }
    }
    launch_chatgpt_inner()
}

fn restart_chatgpt_inner() -> Result<(), String> {
    let app = detect_chatgpt_app()?;
    if !app.installed {
        return Err("未检测到 ChatGPT".to_string());
    }
    stop_chatgpt(&app)?;
    thread::sleep(Duration::from_millis(900));
    launch_chatgpt_preferred()
}

fn snapshot_manifest_path(paths: &InstallerPaths, timestamp: &str) -> PathBuf {
    paths
        .install_root
        .join(format!("snapshot.{timestamp}.json"))
}

fn snapshot_backup_path(path: &Path, timestamp: &str) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("无法确定备份目录: {}", path.display()))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("备份文件名无效: {}", path.display()))?;
    Ok(parent.join(format!("{file_name}.bak.{timestamp}")))
}

fn next_snapshot_timestamp(paths: &InstallerPaths) -> String {
    let mut timestamp = unix_timestamp().parse::<u64>().unwrap_or(0);
    loop {
        let candidate = timestamp.to_string();
        if !snapshot_manifest_path(paths, &candidate).exists() {
            return candidate;
        }
        timestamp += 1;
    }
}

fn backup_snapshot_file(path: &Path, timestamp: &str) -> Result<bool, String> {
    if !path.is_file() {
        return Ok(false);
    }
    let backup = snapshot_backup_path(path, timestamp)?;
    fs::copy(path, &backup).map_err(|error| format!("备份 {} 失败: {error}", path.display()))?;
    Ok(true)
}

fn create_configuration_snapshot(paths: &InstallerPaths) -> Result<PathBuf, String> {
    fs::create_dir_all(&paths.install_root)
        .map_err(|error| format!("创建配置快照目录失败: {error}"))?;
    if let Some(parent) = paths.codex_config_path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("创建 Codex 配置目录失败: {error}"))?;
    }
    let timestamp = next_snapshot_timestamp(paths);
    let state_path = paths.install_root.join("config.json");
    let models_path = paths.install_root.join("models.json");
    let key_path = paths.install_root.join("router-key.secret");
    let snapshot = ConfigurationSnapshot {
        version: VERSION.to_string(),
        timestamp: timestamp.clone(),
        codex_config_existed: backup_snapshot_file(&paths.codex_config_path, &timestamp)?,
        state_existed: backup_snapshot_file(&state_path, &timestamp)?,
        models_existed: backup_snapshot_file(&models_path, &timestamp)?,
        key_existed: backup_snapshot_file(&key_path, &timestamp)?,
    };
    let manifest_path = snapshot_manifest_path(paths, &timestamp);
    let data = serde_json::to_string_pretty(&snapshot)
        .map_err(|error| format!("生成配置快照失败: {error}"))?;
    fs::write(&manifest_path, format!("{data}\n"))
        .map_err(|error| format!("写入配置快照失败: {error}"))?;
    Ok(manifest_path)
}

fn latest_configuration_snapshot(
    paths: &InstallerPaths,
) -> Result<Option<(u64, PathBuf, ConfigurationSnapshot)>, String> {
    if !paths.install_root.is_dir() {
        return Ok(None);
    }
    let mut candidates = fs::read_dir(&paths.install_root)
        .map_err(|error| format!("读取配置快照目录失败: {error}"))?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_str()?;
            let timestamp = name
                .strip_prefix("snapshot.")?
                .strip_suffix(".json")?
                .parse::<u64>()
                .ok()?;
            let data = fs::read_to_string(entry.path()).ok()?;
            let snapshot = serde_json::from_str::<ConfigurationSnapshot>(&data).ok()?;
            (snapshot.timestamp == timestamp.to_string()).then_some((
                timestamp,
                entry.path(),
                snapshot,
            ))
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(timestamp, _, _)| *timestamp);
    Ok(candidates.pop())
}

fn restore_snapshot_file(path: &Path, timestamp: &str, existed: bool) -> Result<(), String> {
    if existed {
        let backup = snapshot_backup_path(path, timestamp)?;
        if !backup.is_file() {
            return Err(format!("配置快照不完整，缺少 {}", backup.display()));
        }
        fs::copy(&backup, path)
            .map_err(|error| format!("恢复 {} 失败: {error}", path.display()))?;
    } else if path.exists() {
        fs::remove_file(path).map_err(|error| format!("清理 {} 失败: {error}", path.display()))?;
    }
    Ok(())
}

fn restore_configuration_snapshot(paths: &InstallerPaths) -> Result<RestoreResult, String> {
    let (_, manifest_path, snapshot) = latest_configuration_snapshot(paths)?
        .ok_or_else(|| "没有可恢复的完整配置快照".to_string())?;
    create_configuration_snapshot(paths)?;
    let state_path = paths.install_root.join("config.json");
    let models_path = paths.install_root.join("models.json");
    let key_path = paths.install_root.join("router-key.secret");
    restore_snapshot_file(
        &paths.codex_config_path,
        &snapshot.timestamp,
        snapshot.codex_config_existed,
    )?;
    restore_snapshot_file(&state_path, &snapshot.timestamp, snapshot.state_existed)?;
    restore_snapshot_file(&models_path, &snapshot.timestamp, snapshot.models_existed)?;
    restore_snapshot_file(&key_path, &snapshot.timestamp, snapshot.key_existed)?;
    Ok(RestoreResult {
        restored_from: manifest_path.to_string_lossy().to_string(),
        message: "已恢复最近一次完整配置；重新打开 ChatGPT 后生效".to_string(),
    })
}

fn restore_codex_config_inner() -> Result<RestoreResult, String> {
    restore_configuration_snapshot(&resolve_paths()?)
}

fn reset_step(id: &str, label: &str, status: &str, message: impl Into<String>) -> ResetStepResult {
    ResetStepResult {
        id: id.to_string(),
        label: label.to_string(),
        status: status.to_string(),
        message: message.into(),
    }
}

fn factory_reset_inner() -> Result<FactoryResetResult, String> {
    let paths = resolve_paths()?;
    let detected = detect_chatgpt_app()?;
    let mut steps = Vec::new();

    if detected.installed {
        match stop_chatgpt(&detected) {
            Ok(()) => steps.push(reset_step(
                "stop",
                "停止 ChatGPT",
                "complete",
                "已停止正在运行的 ChatGPT",
            )),
            Err(error) => steps.push(reset_step(
                "stop",
                "停止 ChatGPT",
                "failed",
                friendly_error(&error),
            )),
        }
        if cfg!(target_os = "windows") {
            match uninstall_chatgpt() {
                Ok(()) => steps.push(reset_step(
                    "uninstall",
                    "卸载 ChatGPT",
                    "complete",
                    "已通过系统官方渠道卸载",
                )),
                Err(error) => steps.push(reset_step(
                    "uninstall",
                    "卸载 ChatGPT",
                    "failed",
                    friendly_error(&error),
                )),
            }
        } else {
            steps.push(reset_step(
                "uninstall",
                "卸载 ChatGPT",
                "skipped",
                "当前平台请手动卸载 ChatGPT",
            ));
        }
    } else {
        steps.push(reset_step(
            "stop",
            "停止 ChatGPT",
            "skipped",
            "未检测到运行中的 ChatGPT",
        ));
        steps.push(reset_step(
            "uninstall",
            "卸载 ChatGPT",
            "skipped",
            "ChatGPT 未安装，无需卸载",
        ));
    }

    match clean_codex_config(&paths.codex_config_path) {
        Ok(true) => steps.push(reset_step(
            "config",
            "移除 Codex 配置",
            "complete",
            "已从 config.toml 移除助手写入的配置",
        )),
        Ok(false) => steps.push(reset_step(
            "config",
            "移除 Codex 配置",
            "skipped",
            "没有需要移除的助手配置",
        )),
        Err(error) => steps.push(reset_step(
            "config",
            "移除 Codex 配置",
            "failed",
            friendly_error(&error),
        )),
    }

    let data_root = assistant_data_root(&paths);
    match remove_assistant_data(&data_root) {
        Ok(true) => steps.push(reset_step(
            "data",
            "清除助手数据",
            "complete",
            "已删除本地状态、备份、主题和保存的 Key",
        )),
        Ok(false) => steps.push(reset_step(
            "data",
            "清除助手数据",
            "skipped",
            "没有可清除的助手数据",
        )),
        Err(error) => steps.push(reset_step(
            "data",
            "清除助手数据",
            "failed",
            friendly_error(&error),
        )),
    }

    let uninstall_expected = detected.installed && cfg!(target_os = "windows");
    let app_gone = !detect_chatgpt_app()?.installed;
    let config_clean = !config_mentions_assistant(&paths.codex_config_path);
    let data_gone = !data_root.exists();
    let verified = config_clean && data_gone && (!uninstall_expected || app_gone);
    steps.push(reset_step(
        "verify",
        "复核",
        if verified { "complete" } else { "failed" },
        if verified {
            "已恢复初始状态".to_string()
        } else {
            "仍有项目未清理干净，请重试一键还原".to_string()
        },
    ));

    let success = steps.iter().all(|step| step.status != "failed");
    Ok(FactoryResetResult {
        success,
        summary: if success {
            "已还原到初始状态".to_string()
        } else {
            "还原未完成，请查看失败步骤".to_string()
        },
        steps,
    })
}

fn assistant_data_root(paths: &InstallerPaths) -> PathBuf {
    match paths.install_root.parent() {
        Some(parent) if parent.file_name().and_then(|name| name.to_str()) == Some("CodexAssistant") => {
            parent.to_path_buf()
        }
        _ => paths.install_root.clone(),
    }
}

fn remove_assistant_data(data_root: &Path) -> Result<bool, String> {
    if !data_root.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(data_root).map_err(|error| format!("删除助手数据目录失败: {error}"))?;
    Ok(true)
}

fn clean_codex_config(path: &Path) -> Result<bool, String> {
    let Ok(existing) = fs::read_to_string(path) else {
        return Ok(false);
    };
    let cleaned = remove_managed_blocks(&existing);
    if cleaned.trim() == existing.trim() {
        return Ok(false);
    }
    if cleaned.trim().is_empty() {
        fs::remove_file(path).map_err(|error| format!("删除 Codex 配置失败: {error}"))?;
        if let Some(parent) = path.parent() {
            // 仅在 .codex 目录已空时移除，保留用户的其他 Codex 文件
            let _ = fs::remove_dir(parent);
        }
    } else {
        fs::write(path, format!("{}\n", cleaned.trim_end()))
            .map_err(|error| format!("重写 Codex 配置失败: {error}"))?;
    }
    Ok(true)
}

fn config_mentions_assistant(path: &Path) -> bool {
    fs::read_to_string(path)
        .map(|content| {
            content.contains(PROVIDER_ID)
                || content.contains("model_catalog_json")
                || content.contains(CONFIG_START)
                || content.contains(LEGACY_CONFIG_START)
        })
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn uninstall_chatgpt() -> Result<(), String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
$packages = @(Get-AppxPackage -Name 'OpenAI.Codex' -ErrorAction SilentlyContinue)
if ($packages.Count -eq 0) {
  $packages = @(Get-AppxPackage | Where-Object { $_.PackageFamilyName -like 'OpenAI.Codex_*' })
}
if ($packages.Count -eq 0) { throw '未找到 ChatGPT 安装包' }
foreach ($pkg in $packages) {
  Remove-AppxPackage -Package $pkg.PackageFullName -ErrorAction Stop
}
'OK'
"#;
    run_command_capture(
        "powershell.exe",
        &["-NoProfile", "-NonInteractive", "-Command", script],
        Duration::from_secs(180),
    )?;
    if wait_for_chatgpt_removed(Duration::from_secs(60)) {
        Ok(())
    } else {
        Err("卸载命令已结束，但 60 秒内仍能检测到 ChatGPT".to_string())
    }
}

#[cfg(not(target_os = "windows"))]
fn uninstall_chatgpt() -> Result<(), String> {
    Err("当前平台请手动卸载 ChatGPT".to_string())
}

fn wait_for_chatgpt_removed(timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if detect_chatgpt_app().map(|app| !app.installed).unwrap_or(true) {
            return true;
        }
        thread::sleep(Duration::from_secs(2));
    }
    detect_chatgpt_app().map(|app| !app.installed).unwrap_or(true)
}

fn collect_appearance_status() -> Result<AppearanceStatus, String> {
    let supported = cfg!(any(target_os = "windows", target_os = "macos"));
    let paths = resolve_paths()?;
    let state = read_appearance_state(&paths).unwrap_or(AppearanceState {
        version: VERSION.to_string(),
        selected_theme: "official".to_string(),
        port: APPEARANCE_PORT,
        applied_at: String::new(),
    });
    let custom_theme = read_custom_theme(&paths).ok();
    let custom_theme_ready = custom_theme.is_some();
    let art_ready = !is_art_theme(&state.selected_theme)
        || resolve_art_theme(&paths, &state.selected_theme, false).is_ok();
    let active = state.selected_theme != "official"
        && fetch_cdp_targets(state.port)
            .map(|targets| !targets.is_empty())
            .unwrap_or(false);
    let message = if !supported {
        "当前平台不支持 ChatGPT 个性化".to_string()
    } else if state.selected_theme == "official" {
        "当前使用 ChatGPT 官方外观".to_string()
    } else if state.selected_theme == "custom" && !custom_theme_ready {
        "自定义背景文件缺失，请重新选择图片".to_string()
    } else if !art_ready {
        "主题文件缺失，请在外观页重新应用该主题".to_string()
    } else if active {
        "主题已在本次 ChatGPT 会话中生效".to_string()
    } else {
        "主题已保存，下次从助手打开 ChatGPT 时生效".to_string()
    };
    let requires_restart = state.selected_theme != "official" && !active;
    Ok(AppearanceStatus {
        supported,
        selected_theme: state.selected_theme,
        active,
        requires_restart,
        custom_theme_ready,
        custom_theme_name: custom_theme.map(|(info, _)| info.file_name),
        message,
    })
}

fn apply_appearance_inner(theme: &str) -> Result<AppearanceStatus, String> {
    apply_appearance_with_download(theme, true)
}

fn apply_appearance_with_download(theme: &str, allow_download: bool) -> Result<AppearanceStatus, String> {
    if !matches!(theme, "official" | "focus" | "custom") && !is_art_theme(theme) {
        return Err("不支持的主题".to_string());
    }
    if !cfg!(any(target_os = "windows", target_os = "macos")) {
        return Err("当前平台不支持 ChatGPT 个性化".to_string());
    }
    let app = detect_chatgpt_app()?;
    if !app.installed {
        return Err("请先安装官方 ChatGPT".to_string());
    }
    let paths = resolve_paths()?;
    fs::create_dir_all(&paths.install_root)
        .map_err(|error| format!("创建外观状态目录失败: {error}"))?;
    let prepared = if theme == "official" {
        None
    } else {
        Some(prepare_theme(&paths, theme, allow_download)?)
    };

    if theme == "official" {
        stop_chatgpt(&app)?;
        thread::sleep(Duration::from_millis(700));
        let state = AppearanceState {
            version: VERSION.to_string(),
            selected_theme: "official".to_string(),
            port: APPEARANCE_PORT,
            applied_at: unix_timestamp(),
        };
        write_appearance_state(&paths, &state)?;
        launch_chatgpt_inner()?;
        return collect_appearance_status();
    }

    let prepared = prepared.expect("non-official theme must be prepared");
    stop_chatgpt(&app)?;
    thread::sleep(Duration::from_millis(700));
    let port = select_appearance_port()?;
    if let Err(error) = launch_chatgpt_with_cdp(&app, port)
        .and_then(|_| wait_for_cdp_targets(port, Duration::from_secs(45)))
        .and_then(|targets| {
            validate_cdp_owner(port, &app)?;
            inject_theme_into_targets(&prepared, port, &targets)
        })
    {
        let _ = stop_chatgpt(&app);
        thread::sleep(Duration::from_millis(500));
        let fallback_state = AppearanceState {
            version: VERSION.to_string(),
            selected_theme: "official".to_string(),
            port: APPEARANCE_PORT,
            applied_at: unix_timestamp(),
        };
        let _ = write_appearance_state(&paths, &fallback_state);
        let _ = launch_chatgpt_inner();
        return Err(format!(
            "主题启动失败，已恢复官方启动方式：{}",
            friendly_error(&error)
        ));
    }
    let state = AppearanceState {
        version: VERSION.to_string(),
        selected_theme: theme.to_string(),
        port,
        applied_at: unix_timestamp(),
    };
    write_appearance_state(&paths, &state)?;
    collect_appearance_status()
}

fn appearance_state_path(paths: &InstallerPaths) -> PathBuf {
    paths.install_root.join("appearance.json")
}

fn read_appearance_state(paths: &InstallerPaths) -> Result<AppearanceState, String> {
    let data =
        fs::read_to_string(appearance_state_path(paths)).map_err(|error| error.to_string())?;
    serde_json::from_str(&data).map_err(|error| error.to_string())
}

fn write_appearance_state(paths: &InstallerPaths, state: &AppearanceState) -> Result<(), String> {
    let data = serde_json::to_string_pretty(state)
        .map_err(|error| format!("生成外观状态失败: {error}"))?;
    fs::write(appearance_state_path(paths), format!("{data}\n"))
        .map_err(|error| format!("保存外观状态失败: {error}"))
}

fn theme_directory(paths: &InstallerPaths) -> PathBuf {
    paths.install_root.join("themes")
}

fn theme_image_info_path(paths: &InstallerPaths) -> PathBuf {
    theme_directory(paths).join("custom-theme.json")
}

fn import_theme_image_inner(request: &ThemeImageRequest) -> Result<AppearanceStatus, String> {
    let comma = request
        .data_url
        .find(',')
        .ok_or_else(|| "主题图片数据格式不正确".to_string())?;
    let metadata = &request.data_url[..comma];
    let mime_type = metadata
        .strip_prefix("data:")
        .and_then(|value| value.strip_suffix(";base64"))
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "主题图片必须使用 base64 data URL".to_string())?;
    if !request.mime_type.trim().is_empty()
        && !request.mime_type.trim().eq_ignore_ascii_case(&mime_type)
    {
        return Err("主题图片 MIME 类型不一致".to_string());
    }
    let bytes = BASE64_STANDARD
        .decode(&request.data_url[comma + 1..])
        .map_err(|_| "主题图片 base64 内容无效".to_string())?;
    if bytes.len() > MAX_THEME_IMAGE_BYTES {
        return Err("主题图片不能超过 8 MB".to_string());
    }
    let (extension, width, height) = validate_theme_image(&mime_type, &bytes)?;
    let file_name = safe_theme_file_name(&request.file_name);
    let paths = resolve_paths()?;
    let directory = theme_directory(&paths);
    fs::create_dir_all(&directory).map_err(|error| format!("创建主题目录失败: {error}"))?;
    let stored_file = format!(
        "custom-background-{}-{}.{}",
        unix_timestamp(),
        std::process::id(),
        extension
    );
    let image_path = directory.join(&stored_file);
    fs::write(&image_path, &bytes).map_err(|error| format!("保存主题图片失败: {error}"))?;
    let info = ThemeImageInfo {
        file_name,
        mime_type,
        stored_file: stored_file.clone(),
        width,
        height,
        imported_at: unix_timestamp(),
    };
    let info_data = serde_json::to_string_pretty(&info)
        .map_err(|error| format!("生成主题图片信息失败: {error}"))?;
    let info_path = theme_image_info_path(&paths);
    let temporary_info = directory.join(".custom-theme.json.tmp");
    fs::write(&temporary_info, format!("{info_data}\n"))
        .map_err(|error| format!("保存主题图片信息失败: {error}"))?;
    if info_path.exists() {
        fs::remove_file(&info_path).map_err(|error| format!("替换主题图片信息失败: {error}"))?;
    }
    fs::rename(&temporary_info, &info_path)
        .map_err(|error| format!("提交主题图片信息失败: {error}"))?;
    cleanup_old_theme_images(&directory, &stored_file);
    collect_appearance_status()
}

fn safe_theme_file_name(input: &str) -> String {
    let name = Path::new(input)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("自定义背景");
    let cleaned = name
        .chars()
        .filter(|character| !character.is_control())
        .take(120)
        .collect::<String>();
    if cleaned.trim().is_empty() {
        "自定义背景".to_string()
    } else {
        cleaned
    }
}

fn cleanup_old_theme_images(directory: &Path, active_file: &str) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("custom-background-") && name != active_file {
            let _ = fs::remove_file(entry.path());
        }
    }
}

fn read_custom_theme(paths: &InstallerPaths) -> Result<(ThemeImageInfo, Vec<u8>), String> {
    let info_data = fs::read_to_string(theme_image_info_path(paths))
        .map_err(|_| "尚未导入自定义背景".to_string())?;
    let info: ThemeImageInfo =
        serde_json::from_str(&info_data).map_err(|_| "自定义背景信息已损坏".to_string())?;
    if !info.stored_file.starts_with("custom-background-")
        || !is_safe_package_token(&info.stored_file, 160)
    {
        return Err("自定义背景文件名不安全".to_string());
    }
    let image_path = theme_directory(paths).join(&info.stored_file);
    let bytes = fs::read(&image_path).map_err(|_| "自定义背景文件缺失".to_string())?;
    if bytes.len() > MAX_THEME_IMAGE_BYTES {
        return Err("自定义背景文件超过 8 MB".to_string());
    }
    let (_, width, height) = validate_theme_image(&info.mime_type, &bytes)?;
    if width != info.width || height != info.height {
        return Err("自定义背景尺寸与导入记录不一致".to_string());
    }
    Ok((info, bytes))
}

fn custom_theme_data_url(paths: &InstallerPaths) -> Result<String, String> {
    let (info, bytes) = read_custom_theme(paths)?;
    Ok(format!(
        "data:{};base64,{}",
        info.mime_type,
        BASE64_STANDARD.encode(bytes)
    ))
}

fn is_art_theme(theme: &str) -> bool {
    theme.starts_with("preset:") || theme.starts_with("gallery:")
}

fn prepare_theme(paths: &InstallerPaths, theme: &str, allow_download: bool) -> Result<PreparedTheme, String> {
    match theme {
        "focus" => Ok(PreparedTheme {
            attr: "focus".to_string(),
            css: FOCUS_THEME_CSS.to_string(),
            image_data_url: None,
        }),
        "custom" => Ok(PreparedTheme {
            attr: "custom".to_string(),
            css: CUSTOM_THEME_CSS.to_string(),
            image_data_url: Some(custom_theme_data_url(paths)?),
        }),
        _ if is_art_theme(theme) => {
            let art = resolve_art_theme(paths, theme, allow_download)?;
            let css = render_art_css(&art.attr, &art.meta);
            let data_url = format!(
                "data:{};base64,{}",
                art.meta.mime,
                BASE64_STANDARD.encode(&art.bytes)
            );
            Ok(PreparedTheme {
                attr: art.attr,
                css,
                image_data_url: Some(data_url),
            })
        }
        _ => Err("不支持的主题".to_string()),
    }
}

fn resolve_art_theme(
    paths: &InstallerPaths,
    theme: &str,
    allow_download: bool,
) -> Result<ResolvedArt, String> {
    if let Some(slug) = theme.strip_prefix("preset:") {
        let preset = ART_PRESETS
            .iter()
            .find(|preset| preset.slug == slug)
            .ok_or_else(|| "未知的内置主题".to_string())?;
        return Ok(ResolvedArt {
            attr: format!("art-{}", preset.slug),
            meta: ArtThemeMeta {
                name: preset.name.to_string(),
                author: preset.author.to_string(),
                license: preset.license.to_string(),
                appearance: preset.appearance.to_string(),
                focus_x: preset.focus_x,
                focus_y: preset.focus_y,
                colors: preset.colors.into(),
                stored_file: String::new(),
                mime: preset.mime.to_string(),
            },
            bytes: preset.bytes.to_vec(),
        });
    }
    if let Some(version_id) = theme.strip_prefix("gallery:") {
        validate_gallery_version_id(version_id)?;
        let directory = gallery_theme_directory(paths, version_id);
        if !directory.join("meta.json").exists() {
            if !allow_download {
                return Err("主题文件缺失，请在外观页重新应用该主题".to_string());
            }
            download_gallery_theme(paths, version_id)?;
        }
        let meta_data = fs::read_to_string(directory.join("meta.json"))
            .map_err(|_| "主题信息缺失，请重新下载该主题".to_string())?;
        let meta: ArtThemeMeta =
            serde_json::from_str(&meta_data).map_err(|_| "主题信息已损坏，请重新下载".to_string())?;
        if !meta.stored_file.starts_with("background.") || !is_safe_package_token(&meta.stored_file, 64)
        {
            return Err("主题背景文件名不安全".to_string());
        }
        let bytes = fs::read(directory.join(&meta.stored_file))
            .map_err(|_| "主题背景文件缺失，请重新下载".to_string())?;
        if bytes.len() > MAX_THEME_IMAGE_BYTES {
            return Err("主题背景文件超过 8 MB".to_string());
        }
        validate_theme_image(&meta.mime, &bytes)?;
        return Ok(ResolvedArt {
            attr: format!("art-g-{}", &version_id[4..]),
            meta,
            bytes,
        });
    }
    Err("不支持的主题".to_string())
}

fn validate_gallery_version_id(version_id: &str) -> Result<(), String> {
    let valid = version_id.len() >= 12
        && version_id.len() <= 68
        && version_id.starts_with("ver_")
        && version_id[4..]
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        Err("主题版本标识不合法".to_string())
    }
}

fn gallery_theme_directory(paths: &InstallerPaths, version_id: &str) -> PathBuf {
    theme_directory(paths).join("gallery").join(version_id)
}

fn list_preset_themes_inner() -> Result<Vec<PresetThemeInfo>, String> {
    Ok(ART_PRESETS
        .iter()
        .map(|preset| PresetThemeInfo {
            id: format!("preset:{}", preset.slug),
            name: preset.name.to_string(),
            author: preset.author.to_string(),
            license: preset.license.to_string(),
            appearance: preset.appearance.to_string(),
            colors: preset.colors.into(),
            preview_data_url: format!(
                "data:{};base64,{}",
                preset.mime,
                BASE64_STANDARD.encode(preset.bytes)
            ),
        })
        .collect())
}

fn gallery_http_agent() -> ureq::Agent {
    let mut builder = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(120))
        .user_agent(concat!("CodexAssistant/", env!("CARGO_PKG_VERSION")));
    for variable in ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy"] {
        let Ok(value) = env::var(variable) else {
            continue;
        };
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        if let Ok(proxy) = ureq::Proxy::new(value) {
            builder = builder.proxy(proxy);
            break;
        }
    }
    builder.build()
}

fn list_gallery_themes_inner() -> Result<Vec<GalleryThemeInfo>, String> {
    let paths = resolve_paths()?;
    let agent = gallery_http_agent();
    let url = format!(
        "{GALLERY_API_BASE}/v1/themes?sort=popular&limit={GALLERY_LIST_LIMIT}"
    );
    let response = agent
        .get(&url)
        .call()
        .map_err(|error| format!("连接在线主题库失败（如网络受限可先配置代理）: {error}"))?;
    let payload: serde_json::Value = response
        .into_json()
        .map_err(|error| format!("解析在线主题库响应失败: {error}"))?;
    let items = payload
        .get("items")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "在线主题库返回数据格式不正确".to_string())?;
    let mut themes = Vec::new();
    for item in items.iter().take(GALLERY_LIST_LIMIT) {
        let Some(version_id) = item.get("id").and_then(|value| value.as_str()) else {
            continue;
        };
        if validate_gallery_version_id(version_id).is_err() {
            continue;
        }
        let name = item
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("未命名主题")
            .trim()
            .to_string();
        let colors = item
            .pointer("/displayMeta/colors")
            .and_then(|value| serde_json::from_value::<ArtColors>(value.clone()).ok());
        let appearance = item
            .pointer("/displayMeta/appearance")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string());
        themes.push(GalleryThemeInfo {
            version_id: version_id.to_string(),
            theme_id: item
                .get("themeId")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string(),
            name,
            author: item
                .get("authorDisplayName")
                .and_then(|value| value.as_str())
                .unwrap_or("未知作者")
                .to_string(),
            license: item
                .get("license")
                .and_then(|value| value.as_str())
                .unwrap_or("未声明")
                .to_string(),
            downloads: item
                .get("downloadCount")
                .and_then(|value| value.as_u64())
                .unwrap_or(0),
            package_bytes: item
                .get("packageBytes")
                .and_then(|value| value.as_u64())
                .unwrap_or(0),
            appearance,
            colors,
            downloaded: gallery_theme_directory(&paths, version_id)
                .join("meta.json")
                .exists(),
        });
    }
    if themes.is_empty() {
        return Err("在线主题库暂时没有可用主题".to_string());
    }
    Ok(themes)
}

fn download_gallery_theme(paths: &InstallerPaths, version_id: &str) -> Result<(), String> {
    validate_gallery_version_id(version_id)?;
    let agent = gallery_http_agent();
    let detail_url = format!("{GALLERY_API_BASE}/v1/themes/{version_id}");
    let detail: serde_json::Value = agent
        .get(&detail_url)
        .call()
        .and_then(|response| response.into_json().map_err(ureq::Error::from))
        .map_err(|error| format!("获取主题信息失败: {error}"))?;
    let expected_sha = detail
        .get("packageSha256")
        .and_then(|value| value.as_str())
        .map(|value| value.to_ascii_lowercase());
    let expected_bytes = detail.get("packageBytes").and_then(|value| value.as_u64());
    if expected_bytes.unwrap_or(0) > MAX_GALLERY_PACKAGE_BYTES {
        return Err("主题包超过 48 MB 限制".to_string());
    }

    let download_url = format!("{GALLERY_API_BASE}/v1/themes/{version_id}/download");
    let response = agent
        .get(&download_url)
        .call()
        .map_err(|error| format!("下载主题包失败（如网络受限可先配置代理）: {error}"))?;
    let mut package = Vec::new();
    response
        .into_reader()
        .take(MAX_GALLERY_PACKAGE_BYTES + 1)
        .read_to_end(&mut package)
        .map_err(|error| format!("读取主题包失败: {error}"))?;
    if package.len() as u64 > MAX_GALLERY_PACKAGE_BYTES {
        return Err("主题包超过 48 MB 限制".to_string());
    }
    if let Some(expected) = expected_bytes {
        if expected != package.len() as u64 {
            return Err("主题包大小与清单不一致，已中止".to_string());
        }
    }
    if let Some(expected) = expected_sha {
        let digest = Sha256::digest(&package);
        let actual = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        if actual != expected {
            return Err("主题包校验和不一致，已中止".to_string());
        }
    }

    let mut archive = zip::ZipArchive::new(Cursor::new(package))
        .map_err(|_| "主题包不是有效的 ZIP 文件".to_string())?;
    if archive.len() > MAX_GALLERY_ZIP_ENTRIES {
        return Err("主题包文件数量过多".to_string());
    }
    let mut theme_json: Option<serde_json::Value> = None;
    let mut image: Option<(String, Vec<u8>)> = None;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|error| format!("读取主题包条目失败: {error}"))?;
        let enclosed = entry
            .enclosed_name()
            .map(PathBuf::from)
            .ok_or_else(|| "主题包包含不安全路径".to_string())?;
        let depth = enclosed.components().count();
        if depth > 2 {
            continue;
        }
        let file_name = enclosed
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if file_name == "theme.json" {
            let mut buffer = Vec::new();
            entry
                .take(1024 * 1024)
                .read_to_end(&mut buffer)
                .map_err(|error| format!("读取 theme.json 失败: {error}"))?;
            theme_json = serde_json::from_slice(&buffer).ok().or(theme_json);
        } else if matches!(
            file_name.as_str(),
            name if name.starts_with("background.")
                && matches!(name.rsplit('.').next(), Some("jpg" | "jpeg" | "png" | "webp"))
        ) {
            let mut buffer = Vec::new();
            entry
                .take(MAX_THEME_IMAGE_BYTES as u64 + 1)
                .read_to_end(&mut buffer)
                .map_err(|error| format!("读取主题背景失败: {error}"))?;
            if buffer.len() > MAX_THEME_IMAGE_BYTES {
                return Err("主题背景图片超过 8 MB".to_string());
            }
            image = Some((file_name, buffer));
        }
    }
    let theme_json = theme_json.ok_or_else(|| "主题包缺少 theme.json".to_string())?;
    let declared_image = theme_json
        .get("image")
        .and_then(|value| value.as_str())
        .map(|value| value.to_ascii_lowercase());
    let (image_name, image_bytes) = match (declared_image, image) {
        (Some(declared), Some((found, bytes))) if found == declared => Some((found, bytes)),
        (Some(_), Some(_)) => None,
        (None, Some(found)) => Some(found),
        _ => None,
    }
    .ok_or_else(|| "主题包缺少 theme.json 声明的背景图片".to_string())?;
    let extension = image_name.rsplit('.').next().unwrap_or("jpg");
    let mime = match extension {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        _ => return Err("主题背景仅支持 JPEG、PNG 或 WebP".to_string()),
    };
    validate_theme_image(mime, &image_bytes)?;

    let mut meta = ArtThemeMeta {
        name: theme_json
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("画廊主题")
            .trim()
            .to_string(),
        appearance: theme_json
            .get("appearance")
            .and_then(|value| value.as_str())
            .unwrap_or("dark")
            .to_string(),
        focus_x: theme_json
            .pointer("/art/focusX")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.5)
            .clamp(0.0, 1.0),
        focus_y: theme_json
            .pointer("/art/focusY")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.5)
            .clamp(0.0, 1.0),
        colors: theme_json
            .get("colors")
            .and_then(|value| serde_json::from_value::<ArtColors>(value.clone()).ok())
            .unwrap_or_default(),
        stored_file: format!("background.{extension}"),
        mime: mime.to_string(),
        ..ArtThemeMeta::default()
    };
    meta.author = detail
        .get("authorDisplayName")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    meta.license = detail
        .get("license")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    if !matches!(meta.appearance.as_str(), "dark" | "light" | "auto") {
        meta.appearance = "dark".to_string();
    }

    let directory = gallery_theme_directory(paths, version_id);
    let staging = theme_directory(paths)
        .join("gallery")
        .join(format!(".staging-{version_id}"));
    if staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    fs::create_dir_all(&staging).map_err(|error| format!("创建主题目录失败: {error}"))?;
    let write_result = (|| -> Result<(), String> {
        fs::write(staging.join(&meta.stored_file), &image_bytes)
            .map_err(|error| format!("保存主题背景失败: {error}"))?;
        let meta_data = serde_json::to_string_pretty(&meta)
            .map_err(|error| format!("生成主题信息失败: {error}"))?;
        fs::write(staging.join("meta.json"), format!("{meta_data}\n"))
            .map_err(|error| format!("保存主题信息失败: {error}"))?;
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    if directory.exists() {
        let _ = fs::remove_dir_all(&directory);
    }
    fs::rename(&staging, &directory).map_err(|error| format!("提交主题文件失败: {error}"))?;
    prune_gallery_themes(paths);
    Ok(())
}

fn prune_gallery_themes(paths: &InstallerPaths) {
    let gallery_root = theme_directory(paths).join("gallery");
    let Ok(entries) = fs::read_dir(&gallery_root) else {
        return;
    };
    let mut directories: Vec<(PathBuf, std::time::SystemTime)> = entries
        .flatten()
        .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|name| validate_gallery_version_id(name).is_ok())
                .unwrap_or(false)
        })
        .map(|entry| {
            let modified = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            (entry.path(), modified)
        })
        .collect();
    directories.sort_by_key(|(_, modified)| *modified);
    while directories.len() > MAX_GALLERY_STORED {
        if let Some((path, _)) = directories.first() {
            let _ = fs::remove_dir_all(path);
        }
        directories.remove(0);
    }
}

fn parse_hex_rgb(color: &str) -> Option<(u8, u8, u8)> {
    let hex = color.trim().strip_prefix('#')?;
    let expand = |pair: &str| u8::from_str_radix(pair, 16).ok();
    match hex.len() {
        3 => {
            let mut channels = [0u8; 3];
            for (index, channel) in hex.chars().enumerate() {
                let value = channel.to_digit(16)? as u8;
                channels[index] = value * 16 + value;
            }
            Some((channels[0], channels[1], channels[2]))
        }
        6 | 8 => Some((expand(&hex[0..2])?, expand(&hex[2..4])?, expand(&hex[4..6])?)),
        _ => None,
    }
}

fn is_safe_css_color(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= 64
        && (value.starts_with('#') || value.starts_with("rgb") || value.starts_with("hsl"))
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(character, '#' | ',' | '.' | '(' | ')' | '%' | ' ' | '-')
        })
}

fn css_color(value: &str, fallback: &str) -> String {
    let value = value.trim();
    if is_safe_css_color(value) {
        value.to_string()
    } else {
        fallback.to_string()
    }
}

fn color_with_alpha(value: &str, alpha: f64, fallback: &str) -> String {
    if let Some((red, green, blue)) = parse_hex_rgb(value) {
        format!("rgba({red}, {green}, {blue}, {alpha})")
    } else {
        css_color(value, fallback)
    }
}

fn color_luminance(value: &str) -> Option<f64> {
    let (red, green, blue) = parse_hex_rgb(value)?;
    Some(0.299 * f64::from(red) + 0.587 * f64::from(green) + 0.114 * f64::from(blue))
}

fn art_theme_is_dark(meta: &ArtThemeMeta) -> bool {
    match meta.appearance.as_str() {
        "light" => false,
        "dark" => true,
        _ => color_luminance(&meta.colors.background)
            .map(|luminance| luminance < 128.0)
            .unwrap_or(true),
    }
}

fn render_art_css(attr: &str, meta: &ArtThemeMeta) -> String {
    let dark = art_theme_is_dark(meta);
    let scheme = if dark { "dark" } else { "light" };
    let colors = &meta.colors;
    let background = css_color(&colors.background, "#0b1118");
    let text = css_color(&colors.text, "#f4f7fb");
    let text_soft = color_with_alpha(&colors.text, 0.9, "rgba(244, 247, 251, .90)");
    let panel_a1 = color_with_alpha(&colors.panel, if dark { 0.76 } else { 0.88 }, "rgba(8, 14, 22, .76)");
    let panel_a2 = color_with_alpha(&colors.panel, if dark { 0.46 } else { 0.62 }, "rgba(8, 14, 22, .46)");
    let panel_bar = color_with_alpha(&colors.panel, 0.78, "rgba(15, 23, 33, .78)");
    let composer_bg = color_with_alpha(&colors.panel_alt, 0.84, "rgba(15, 23, 33, .84)");
    let line_soft = color_with_alpha(&colors.line, 0.5, "rgba(255, 255, 255, .14)");
    let line_softer = color_with_alpha(&colors.line, 0.3, "rgba(255, 255, 255, .08)");
    let veil_base: &str = if dark { "#070c13" } else { "#ffffff" };
    let veil_a1 = color_with_alpha(veil_base, if dark { 0.22 } else { 0.3 }, veil_base);
    let veil_a2 = color_with_alpha(veil_base, if dark { 0.06 } else { 0.1 }, veil_base);
    let veil_a3 = color_with_alpha(veil_base, if dark { 0.16 } else { 0.18 }, veil_base);
    let accent = css_color(&colors.accent, "#4f8f78");
    let selection_text = if color_luminance(&colors.accent).unwrap_or(0.0) >= 150.0 {
        "#201a18"
    } else {
        "#ffffff"
    };
    let focus_x = (meta.focus_x.clamp(0.0, 1.0) * 100.0).round() as u32;
    let focus_y = (meta.focus_y.clamp(0.0, 1.0) * 100.0).round() as u32;
    let heading_shadow = if dark {
        "text-shadow: 0 1px 3px rgba(0, 0, 0, .56) !important;"
    } else {
        "text-shadow: 0 1px 2px rgba(255, 255, 255, .35) !important;"
    };
    let selector = format!("html[data-codex-assistant-theme=\"{attr}\"]");
    format!(
        r#"
{selector} {{ color-scheme: {scheme} !important; background-color: {background} !important; }}
{selector} body {{
  min-height: 100vh !important;
  background-color: {background} !important;
  background-image: var(--codex-assistant-art) !important;
  background-position: {focus_x}% {focus_y}% !important;
  background-size: cover !important;
  background-repeat: no-repeat !important;
  background-attachment: fixed !important;
  color: {text} !important;
}}
{selector} aside.app-shell-left-panel {{
  color: {text} !important;
  background: linear-gradient(90deg, {panel_a1}, {panel_a2}) !important;
  border-color: {line_soft} !important;
  box-shadow: inset -1px 0 {line_softer} !important;
  backdrop-filter: blur(10px) saturate(1.06) !important;
}}
{selector} aside.app-shell-left-panel button,
{selector} aside.app-shell-left-panel a,
{selector} aside.app-shell-left-panel [class*="text-token"],
{selector} aside.app-shell-left-panel svg {{
  color: {text_soft} !important;
}}
{selector} main.main-surface {{
  color: {text} !important;
  background: linear-gradient(90deg, {veil_a1}, {veil_a2} 52%, {veil_a3}) !important;
  border: 0 !important;
  box-shadow: none !important;
}}
{selector} main.main-surface > header.app-header-tint,
{selector} main.main-surface [role="main"],
{selector} main.main-surface .app-shell-main-content-frame,
{selector} main.main-surface .app-shell-main-content-top-fade,
{selector} main.main-surface .thread-scroll-container .bg-gradient-to-t.from-token-main-surface-primary {{
  background: transparent !important;
  box-shadow: none !important;
}}
{selector} main.main-surface .app-shell-main-content-top-fade {{ display: none !important; }}
{selector} main.main-surface [class~="bg-token-main-surface-primary"][class~="h-full"][class~="w-full"] {{
  background: rgba(246, 248, 251, .90) !important;
  box-shadow: 0 10px 28px rgba(0, 0, 0, .18), inset 0 0 0 1px rgba(255, 255, 255, .22) !important;
  backdrop-filter: blur(8px) saturate(1.04) !important;
}}
{selector} main.main-surface [class*="_homeUtilityBar_"] {{
  color: {text} !important;
  background: {panel_bar} !important;
  border-color: {line_soft} !important;
  box-shadow: none !important;
  backdrop-filter: blur(10px) saturate(1.04) !important;
}}
{selector} main.main-surface [class*="_homeUtilityBar_"] * {{ color: inherit !important; }}
{selector} .composer-surface-chrome {{
  color: {text} !important;
  background: {composer_bg} !important;
  border-color: {line_soft} !important;
  box-shadow: 0 12px 34px rgba(0, 0, 0, .28), inset 0 0 0 1px {line_softer} !important;
  backdrop-filter: blur(12px) saturate(1.04) !important;
}}
{selector} .composer-surface-chrome button:not([class~="bg-token-foreground"]),
{selector} .composer-surface-chrome input,
{selector} .composer-surface-chrome textarea,
{selector} .composer-surface-chrome [contenteditable="true"],
{selector} .composer-surface-chrome [class*="text-token"],
{selector} .composer-surface-chrome svg {{
  color: {text_soft} !important;
}}
{selector} [class~="group/application-menu-top-bar"],
{selector} [class~="group/application-menu-top-bar"] button,
{selector} [class~="group/application-menu-top-bar"] svg {{
  color: {text_soft} !important;
}}
{selector} main.main-surface h1,
{selector} main.main-surface h2,
{selector} main.main-surface h3,
{selector} main.main-surface .heading-xl,
{selector} main.main-surface .heading-xl * {{
  color: {text} !important;
  {heading_shadow}
}}
{selector} ::selection {{ background-color: {accent} !important; color: {selection_text} !important; }}
"#
    )
}

fn validate_theme_image(mime_type: &str, bytes: &[u8]) -> Result<(&'static str, u32, u32), String> {
    let (extension, dimensions) = match mime_type {
        "image/png" => ("png", png_dimensions(bytes)),
        "image/jpeg" => ("jpg", jpeg_dimensions(bytes)),
        "image/webp" => ("webp", webp_dimensions(bytes)),
        _ => return Err("只支持 PNG、JPEG 或 WebP 背景图片".to_string()),
    };
    let (width, height) = dimensions.ok_or_else(|| "无法识别主题图片内容或尺寸".to_string())?;
    if width == 0
        || height == 0
        || width > MAX_THEME_IMAGE_DIMENSION
        || height > MAX_THEME_IMAGE_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_THEME_IMAGE_PIXELS
    {
        return Err("主题图片尺寸过大；最大边长 16384，总像素不超过 5000 万".to_string());
    }
    Ok((extension, width, height))
}

fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 || bytes.get(..8)? != b"\x89PNG\r\n\x1a\n" || bytes.get(12..16)? != b"IHDR"
    {
        return None;
    }
    Some((
        u32::from_be_bytes(bytes.get(16..20)?.try_into().ok()?),
        u32::from_be_bytes(bytes.get(20..24)?.try_into().ok()?),
    ))
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || bytes.get(..2)? != b"\xff\xd8" {
        return None;
    }
    let mut cursor = 2usize;
    while cursor + 4 <= bytes.len() {
        while cursor < bytes.len() && bytes[cursor] == 0xff {
            cursor += 1;
        }
        let marker = *bytes.get(cursor)?;
        cursor += 1;
        if marker == 0x01 || (0xd0..=0xd9).contains(&marker) {
            continue;
        }
        let segment_length =
            u16::from_be_bytes(bytes.get(cursor..cursor + 2)?.try_into().ok()?) as usize;
        if segment_length < 2 || cursor + segment_length > bytes.len() {
            return None;
        }
        if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) {
            if segment_length < 7 {
                return None;
            }
            let height =
                u16::from_be_bytes(bytes.get(cursor + 3..cursor + 5)?.try_into().ok()?) as u32;
            let width =
                u16::from_be_bytes(bytes.get(cursor + 5..cursor + 7)?.try_into().ok()?) as u32;
            return Some((width, height));
        }
        cursor += segment_length;
    }
    None
}

fn webp_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 30 || bytes.get(..4)? != b"RIFF" || bytes.get(8..12)? != b"WEBP" {
        return None;
    }
    match bytes.get(12..16)? {
        b"VP8X" => {
            let width = 1 + u32::from_le_bytes([bytes[24], bytes[25], bytes[26], 0]);
            let height = 1 + u32::from_le_bytes([bytes[27], bytes[28], bytes[29], 0]);
            Some((width, height))
        }
        b"VP8 " if bytes.get(23..26)? == b"\x9d\x01\x2a" => {
            let width = u16::from_le_bytes(bytes.get(26..28)?.try_into().ok()?) & 0x3fff;
            let height = u16::from_le_bytes(bytes.get(28..30)?.try_into().ok()?) & 0x3fff;
            Some((u32::from(width), u32::from(height)))
        }
        b"VP8L" if bytes.get(20) == Some(&0x2f) => {
            let bits = u32::from_le_bytes(bytes.get(21..25)?.try_into().ok()?);
            Some(((bits & 0x3fff) + 1, ((bits >> 14) & 0x3fff) + 1))
        }
        _ => None,
    }
}

fn select_appearance_port() -> Result<u16, String> {
    for port in APPEARANCE_PORT..=APPEARANCE_PORT + 10 {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Ok(port);
        }
    }
    Err("本机没有可用的主题调试端口".to_string())
}

fn stop_chatgpt(app: &DesktopAppInfo) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let expected = app
            .executable_path
            .as_deref()
            .ok_or_else(|| "缺少 ChatGPT 可执行程序路径".to_string())?;
        let expected = powershell_single_quote(expected);
        let script = format!(
            "$expected = '{expected}'; Get-CimInstance Win32_Process -Filter \"Name = 'ChatGPT.exe'\" -ErrorAction SilentlyContinue | Where-Object {{ $_.ExecutablePath -and ([IO.Path]::GetFullPath($_.ExecutablePath) -ieq [IO.Path]::GetFullPath($expected)) }} | ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force -ErrorAction Stop }}"
        );
        let _ = run_command_capture(
            "powershell.exe",
            &["-NoProfile", "-NonInteractive", "-Command", &script],
            Duration::from_secs(10),
        )?;
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("pkill").args(["-x", "ChatGPT"]).status();
        let _ = app;
        return Ok(());
    }
    #[allow(unreachable_code)]
    Err("当前平台不支持关闭 ChatGPT".to_string())
}

fn launch_chatgpt_with_cdp(app: &DesktopAppInfo, port: u16) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        return launch_chatgpt_package_with_arguments(app, port);
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args([
                "-na",
                "ChatGPT",
                "--args",
                "--remote-debugging-address=127.0.0.1",
                &format!("--remote-debugging-port={port}"),
            ])
            .spawn()
            .map_err(|error| format!("以主题模式启动 ChatGPT 失败: {error}"))?;
        let _ = app;
        return Ok(());
    }
    #[allow(unreachable_code)]
    Err("当前平台不支持主题模式".to_string())
}

#[cfg(target_os = "windows")]
fn launch_chatgpt_package_with_arguments(app: &DesktopAppInfo, port: u16) -> Result<(), String> {
    let pfn = app
        .package_family_name
        .as_deref()
        .ok_or_else(|| "缺少 ChatGPT PackageFamilyName".to_string())?;
    let app_id = app
        .app_id
        .as_deref()
        .ok_or_else(|| "缺少 ChatGPT AppId".to_string())?;
    if !is_safe_package_token(pfn, 128) || !is_safe_package_token(app_id, 64) {
        return Err("ChatGPT 软件包标识不符合安全要求".to_string());
    }
    let aumid = format!("{pfn}!{app_id}");
    let args = format!("--remote-debugging-address=127.0.0.1 --remote-debugging-port={port}");
    let script = r#"
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
[ComImport, Guid("2e941141-7f97-4756-ba1d-9decde894a3d"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IApplicationActivationManager {
  [PreserveSig] int ActivateApplication([MarshalAs(UnmanagedType.LPWStr)] string appUserModelId, [MarshalAs(UnmanagedType.LPWStr)] string arguments, uint options, out uint processId);
}
[ComImport, Guid("45ba127d-10a8-46ea-8ab7-56ea9078943c")]
class ApplicationActivationManager {}
public static class CodexAssistantLauncher {
  public static uint Launch(string appUserModelId, string arguments) {
    var manager = (IApplicationActivationManager)new ApplicationActivationManager();
    try { uint pid; int hr = manager.ActivateApplication(appUserModelId, arguments, 0, out pid); Marshal.ThrowExceptionForHR(hr); return pid; }
    finally { if (Marshal.IsComObject(manager)) Marshal.FinalReleaseComObject(manager); }
  }
}
'@
[CodexAssistantLauncher]::Launch('__AUMID__', '__ARGS__')
"#
    .replace("__AUMID__", &aumid)
    .replace("__ARGS__", &args);
    let output = run_command_capture(
        "powershell.exe",
        &["-NoProfile", "-NonInteractive", "-Command", &script],
        Duration::from_secs(20),
    )?;
    if output.trim().parse::<u32>().unwrap_or(0) == 0 {
        return Err("Windows 未返回 ChatGPT 进程 ID".to_string());
    }
    Ok(())
}

fn is_safe_package_token(value: &str, max_len: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_len
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

#[cfg(target_os = "windows")]
fn powershell_single_quote(value: &str) -> String {
    value.replace('\'', "''")
}

fn wait_for_cdp_targets(port: u16, timeout: Duration) -> Result<Vec<CdpTarget>, String> {
    let deadline = Instant::now() + timeout;
    let mut last_error = "ChatGPT 尚未开放主题调试端口".to_string();
    while Instant::now() < deadline {
        match fetch_cdp_targets(port) {
            Ok(targets) if !targets.is_empty() => return Ok(targets),
            Ok(_) => last_error = "主题调试端口已启动，但还没有 ChatGPT 页面".to_string(),
            Err(error) => last_error = error,
        }
        thread::sleep(Duration::from_millis(400));
    }
    Err(format!("45 秒内未能连接 ChatGPT：{last_error}"))
}

fn fetch_cdp_targets(port: u16) -> Result<Vec<CdpTarget>, String> {
    let endpoint = format!("http://127.0.0.1:{port}/json/list");
    let response = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(1))
        .timeout_read(Duration::from_secs(2))
        .build()
        .get(&endpoint)
        .call()
        .map_err(|error| format!("主题调试端口不可用：{error}"))?;
    let targets: Vec<CdpTarget> = response
        .into_json()
        .map_err(|error| format!("读取主题调试目标失败: {error}"))?;
    Ok(targets
        .into_iter()
        .filter(|target| {
            target.target_type == "page"
                && target.url.starts_with("app://")
                && valid_cdp_websocket_url(&target.web_socket_debugger_url, port)
        })
        .collect())
}

fn valid_cdp_websocket_url(value: &str, port: u16) -> bool {
    let Ok(url) = Url::parse(value) else {
        return false;
    };
    let host_ok = matches!(
        url.host_str(),
        Some("127.0.0.1") | Some("localhost") | Some("::1")
    );
    let Some(id) = url.path().strip_prefix("/devtools/page/") else {
        return false;
    };
    url.scheme() == "ws"
        && host_ok
        && url.port_or_known_default() == Some(port)
        && url.username().is_empty()
        && url.password().is_none()
        && url.query().is_none()
        && url.fragment().is_none()
        && is_safe_package_token(id, 200)
}

fn validate_cdp_owner(port: u16, app: &DesktopAppInfo) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let expected = app
            .executable_path
            .as_deref()
            .ok_or_else(|| "缺少 ChatGPT 可执行程序路径".to_string())?;
        let expected = powershell_single_quote(expected);
        let script = format!(
            "$expected = '{expected}'; $owners = @(Get-NetTCPConnection -LocalAddress 127.0.0.1 -LocalPort {port} -State Listen -ErrorAction Stop | Select-Object -ExpandProperty OwningProcess -Unique); if ($owners.Count -ne 1) {{ throw 'Unexpected listener count' }}; $p = Get-CimInstance Win32_Process -Filter \"ProcessId = $($owners[0])\" -ErrorAction Stop; if (-not $p.ExecutablePath -or ([IO.Path]::GetFullPath($p.ExecutablePath) -ine [IO.Path]::GetFullPath($expected))) {{ throw 'Listener owner is not the verified ChatGPT executable' }}; 'OK'"
        );
        let output = run_command_capture(
            "powershell.exe",
            &["-NoProfile", "-NonInteractive", "-Command", &script],
            Duration::from_secs(10),
        )?;
        if output.trim() != "OK" {
            return Err("无法验证主题调试端口所有者".to_string());
        }
    }
    #[cfg(not(target_os = "windows"))]
    let _ = (port, app);
    Ok(())
}

fn inject_theme_into_targets(
    prepared: &PreparedTheme,
    port: u16,
    targets: &[CdpTarget],
) -> Result<(), String> {
    let source = theme_injection_source(prepared)?;
    for target in targets {
        if target_requires_windows_setup(&target.web_socket_debugger_url, port)? {
            return Err(
                "请先在 ChatGPT 中完成一次 Finish Windows setup，再返回助手应用主题".to_string(),
            );
        }
    }
    let mut applied = 0usize;
    let mut last_error = None;
    for target in targets {
        match inject_theme_target(&target.web_socket_debugger_url, port, &source) {
            Ok(()) => applied += 1,
            Err(error) => last_error = Some(error),
        }
    }
    if applied == 0 {
        return Err(last_error.unwrap_or_else(|| "没有可注入的 ChatGPT 页面".to_string()));
    }
    Ok(())
}

fn target_requires_windows_setup(websocket_url: &str, port: u16) -> Result<bool, String> {
    if !valid_cdp_websocket_url(websocket_url, port) {
        return Err("拒绝连接非回环地址的主题调试目标".to_string());
    }
    let (mut socket, _) =
        connect(websocket_url).map_err(|error| format!("连接 ChatGPT 页面失败: {error}"))?;
    set_websocket_timeout(&mut socket, Duration::from_secs(8));

    for id in 10..20 {
        let result = send_cdp_command(
            &mut socket,
            id,
            "Runtime.evaluate",
            json!({
                "expression": WINDOWS_SETUP_PROBE,
                "awaitPromise": true,
                "returnByValue": true
            }),
        )?;
        let value = result.pointer("/result/result/value");
        if value
            .and_then(|value| value.get("pending"))
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            let _ = socket.close(None);
            return Ok(true);
        }
        let ready = value
            .and_then(|value| value.get("readyState"))
            .and_then(|value| value.as_str())
            == Some("complete");
        let has_content = value
            .and_then(|value| value.get("textLength"))
            .and_then(|value| value.as_u64())
            .unwrap_or(0)
            > 20;
        if ready && has_content {
            let _ = socket.close(None);
            return Ok(false);
        }
        thread::sleep(Duration::from_millis(300));
    }

    let _ = socket.close(None);
    Ok(false)
}

const FOCUS_THEME_CSS: &str = r#"
html[data-codex-assistant-theme="focus"] { color-scheme: dark !important; }
html[data-codex-assistant-theme="focus"] body,
html[data-codex-assistant-theme="focus"] main.main-surface { background-color: #11151b !important; color: #e6e9ee !important; }
html[data-codex-assistant-theme="focus"] aside.app-shell-left-panel { background-color: #191f28 !important; border-color: #303846 !important; color: #e6e9ee !important; }
html[data-codex-assistant-theme="focus"] .composer-surface-chrome { background-color: #202732 !important; border-color: #3a4554 !important; box-shadow: 0 10px 32px rgba(0,0,0,.24) !important; }
html[data-codex-assistant-theme="focus"] main.main-surface h1,
html[data-codex-assistant-theme="focus"] main.main-surface h2,
html[data-codex-assistant-theme="focus"] main.main-surface h3,
html[data-codex-assistant-theme="focus"] main.main-surface p { color: #e6e9ee !important; }
html[data-codex-assistant-theme="focus"] ::selection { background-color: #477a67 !important; color: #ffffff !important; }
"#;

const CUSTOM_THEME_CSS: &str = r#"
html[data-codex-assistant-theme="custom"] { color-scheme: dark !important; background-color: #0b1118 !important; }
html[data-codex-assistant-theme="custom"] body {
  min-height: 100vh !important;
  background-color: #0b1118 !important;
  background-image: var(--codex-assistant-art) !important;
  background-position: center center !important;
  background-size: cover !important;
  background-repeat: no-repeat !important;
  background-attachment: fixed !important;
  color: #f4f7fb !important;
}
html[data-codex-assistant-theme="custom"] aside.app-shell-left-panel {
  color: #f4f7fb !important;
  background: linear-gradient(90deg, rgba(8, 14, 22, .76), rgba(8, 14, 22, .46)) !important;
  border-color: rgba(255, 255, 255, .14) !important;
  box-shadow: inset -1px 0 rgba(255, 255, 255, .08) !important;
  backdrop-filter: blur(10px) saturate(1.06) !important;
}
html[data-codex-assistant-theme="custom"] aside.app-shell-left-panel button,
html[data-codex-assistant-theme="custom"] aside.app-shell-left-panel a,
html[data-codex-assistant-theme="custom"] aside.app-shell-left-panel [class*="text-token"],
html[data-codex-assistant-theme="custom"] aside.app-shell-left-panel svg {
  color: rgba(244, 247, 251, .90) !important;
}
html[data-codex-assistant-theme="custom"] main.main-surface {
  color: #f4f7fb !important;
  background: linear-gradient(90deg, rgba(7, 12, 19, .22), rgba(7, 12, 19, .06) 52%, rgba(7, 12, 19, .16)) !important;
  border: 0 !important;
  box-shadow: none !important;
}
html[data-codex-assistant-theme="custom"] main.main-surface > header.app-header-tint,
html[data-codex-assistant-theme="custom"] main.main-surface [role="main"],
html[data-codex-assistant-theme="custom"] main.main-surface .app-shell-main-content-frame,
html[data-codex-assistant-theme="custom"] main.main-surface .app-shell-main-content-top-fade,
html[data-codex-assistant-theme="custom"] main.main-surface .thread-scroll-container .bg-gradient-to-t.from-token-main-surface-primary {
  background: transparent !important;
  box-shadow: none !important;
}
html[data-codex-assistant-theme="custom"] main.main-surface .app-shell-main-content-top-fade { display: none !important; }
html[data-codex-assistant-theme="custom"] main.main-surface [class~="bg-token-main-surface-primary"][class~="h-full"][class~="w-full"] {
  background: rgba(246, 248, 251, .90) !important;
  box-shadow: 0 10px 28px rgba(0, 0, 0, .18), inset 0 0 0 1px rgba(255, 255, 255, .22) !important;
  backdrop-filter: blur(8px) saturate(1.04) !important;
}
html[data-codex-assistant-theme="custom"] main.main-surface [class*="_homeUtilityBar_"] {
  color: #eef3f8 !important;
  background: rgba(15, 23, 33, .78) !important;
  border-color: rgba(255, 255, 255, .15) !important;
  box-shadow: none !important;
  backdrop-filter: blur(10px) saturate(1.04) !important;
}
html[data-codex-assistant-theme="custom"] main.main-surface [class*="_homeUtilityBar_"] * { color: inherit !important; }
html[data-codex-assistant-theme="custom"] .composer-surface-chrome {
  color: #eef3f8 !important;
  background: rgba(15, 23, 33, .84) !important;
  border-color: rgba(255, 255, 255, .18) !important;
  box-shadow: 0 12px 34px rgba(0, 0, 0, .28), inset 0 0 0 1px rgba(255, 255, 255, .07) !important;
  backdrop-filter: blur(12px) saturate(1.04) !important;
}
html[data-codex-assistant-theme="custom"] .composer-surface-chrome button:not([class~="bg-token-foreground"]),
html[data-codex-assistant-theme="custom"] .composer-surface-chrome input,
html[data-codex-assistant-theme="custom"] .composer-surface-chrome textarea,
html[data-codex-assistant-theme="custom"] .composer-surface-chrome [contenteditable="true"],
html[data-codex-assistant-theme="custom"] .composer-surface-chrome [class*="text-token"],
html[data-codex-assistant-theme="custom"] .composer-surface-chrome svg {
  color: rgba(238, 243, 248, .88) !important;
}
html[data-codex-assistant-theme="custom"] [class~="group/application-menu-top-bar"],
html[data-codex-assistant-theme="custom"] [class~="group/application-menu-top-bar"] button,
html[data-codex-assistant-theme="custom"] [class~="group/application-menu-top-bar"] svg {
  color: rgba(244, 247, 251, .88) !important;
}
html[data-codex-assistant-theme="custom"] main.main-surface h1,
html[data-codex-assistant-theme="custom"] main.main-surface h2,
html[data-codex-assistant-theme="custom"] main.main-surface h3,
html[data-codex-assistant-theme="custom"] main.main-surface .heading-xl,
html[data-codex-assistant-theme="custom"] main.main-surface .heading-xl * {
  color: #f4f7fb !important;
  text-shadow: 0 1px 3px rgba(0, 0, 0, .56) !important;
}
html[data-codex-assistant-theme="custom"] ::selection { background-color: #4f8f78 !important; color: #ffffff !important; }
"#;

fn theme_injection_source(prepared: &PreparedTheme) -> Result<String, String> {
    let css = &prepared.css;
    let custom_image_json = match prepared.image_data_url.as_deref() {
        Some(image) => Some(serde_json::to_string(image).map_err(|error| error.to_string())?),
        None => None,
    };
    let image_setup = if let Some(image_json) = custom_image_json {
        r#"
  const imageDataUrl = __CUSTOM_IMAGE__;
  const comma = imageDataUrl.indexOf(',');
  const metadata = imageDataUrl.slice(5, comma);
  const mimeType = metadata.split(';', 1)[0] || 'application/octet-stream';
  const binary = atob(imageDataUrl.slice(comma + 1));
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index);
  const previousBlobUrl = window.__codexAssistantThemeBlobUrl;
  if (previousBlobUrl) URL.revokeObjectURL(previousBlobUrl);
  const blobUrl = URL.createObjectURL(new Blob([bytes], { type: mimeType }));
  window.__codexAssistantThemeBlobUrl = blobUrl;
  document.documentElement.style.setProperty('--codex-assistant-art', `url("${blobUrl}")`);
"#
        .replace("__CUSTOM_IMAGE__", &image_json)
    } else {
        r#"
  const previousBlobUrl = window.__codexAssistantThemeBlobUrl;
  if (previousBlobUrl) URL.revokeObjectURL(previousBlobUrl);
  delete window.__codexAssistantThemeBlobUrl;
  document.documentElement.style.removeProperty('--codex-assistant-art');
"#
        .to_string()
    };
    let theme_json = serde_json::to_string(&prepared.attr).map_err(|error| error.to_string())?;
    let css_json = serde_json::to_string(css).map_err(|error| error.to_string())?;
    Ok(format!(
        r#"(() => {{
  const id = 'codex-assistant-theme-style';
{image_setup}
  let style = document.getElementById(id);
  if (!style) {{ style = document.createElement('style'); style.id = id; document.documentElement.appendChild(style); }}
  style.textContent = {css_json};
  document.documentElement.dataset.codexAssistantTheme = {theme_json};
  return style.isConnected && document.documentElement.dataset.codexAssistantTheme === {theme_json};
}})()"#
    ))
}

fn inject_theme_target(websocket_url: &str, port: u16, source: &str) -> Result<(), String> {
    if !valid_cdp_websocket_url(websocket_url, port) {
        return Err("拒绝连接非回环地址的主题调试目标".to_string());
    }
    let (mut socket, _) =
        connect(websocket_url).map_err(|error| format!("连接 ChatGPT 页面失败: {error}"))?;
    set_websocket_timeout(&mut socket, Duration::from_secs(8));
    send_cdp_command(
        &mut socket,
        1,
        "Page.addScriptToEvaluateOnNewDocument",
        json!({ "source": source }),
    )?;
    let result = send_cdp_command(
        &mut socket,
        2,
        "Runtime.evaluate",
        json!({ "expression": source, "awaitPromise": true, "returnByValue": true }),
    )?;
    let applied = result
        .pointer("/result/result/value")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let _ = socket.close(None);
    if applied {
        Ok(())
    } else {
        Err("ChatGPT 页面未确认主题已生效".to_string())
    }
}

fn set_websocket_timeout(
    socket: &mut WebSocket<MaybeTlsStream<std::net::TcpStream>>,
    timeout: Duration,
) {
    if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));
    }
}

fn send_cdp_command(
    socket: &mut WebSocket<MaybeTlsStream<std::net::TcpStream>>,
    id: u64,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let command = json!({ "id": id, "method": method, "params": params });
    socket
        .send(Message::Text(command.to_string().into()))
        .map_err(|error| format!("发送主题命令失败: {error}"))?;
    loop {
        let message = socket
            .read()
            .map_err(|error| format!("读取主题命令结果失败: {error}"))?;
        let Message::Text(text) = message else {
            continue;
        };
        let payload: serde_json::Value = serde_json::from_str(&text)
            .map_err(|error| format!("解析主题命令结果失败: {error}"))?;
        if payload.get("id").and_then(|value| value.as_u64()) != Some(id) {
            continue;
        }
        if let Some(error) = payload.get("error") {
            return Err(format!("ChatGPT 拒绝主题命令: {error}"));
        }
        return Ok(payload);
    }
}

fn resolve_paths() -> Result<InstallerPaths, String> {
    let home = user_home_dir()?;
    let data_root = if cfg!(target_os = "macos") {
        home.join("Library").join("Application Support")
    } else {
        env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join("AppData").join("Local"))
    };
    Ok(InstallerPaths {
        install_root: data_root.join("CodexAssistant").join("runtime"),
        codex_config_path: home.join(".codex").join("config.toml"),
    })
}

fn user_home_dir() -> Result<PathBuf, String> {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| "无法确定当前用户目录".to_string())
}

fn prepare_token(
    paths: &InstallerPaths,
    options: &SetupOptions,
    existing: Option<&InstallState>,
) -> Result<TokenPrep, String> {
    let key_path = paths.install_root.join("router-key.secret");
    if options.no_auth {
        let _ = fs::remove_file(&key_path);
        return Ok(TokenPrep {
            token_mode: "none".to_string(),
            key_path: None,
            secret_storage: None,
        });
    }
    if !options.key.is_empty() {
        let (protected, storage) = token_support::protect_gateway_secret(&options.key)?;
        fs::write(&key_path, protected)
            .map_err(|error| format!("安全保存 Access Key 失败: {error}"))?;
        return Ok(TokenPrep {
            token_mode: "static".to_string(),
            key_path: Some(key_path),
            secret_storage: Some(storage.to_string()),
        });
    }
    if let Some(saved) = existing {
        if saved.gateway_base_url == options.gateway && saved.token_mode == "static" {
            let saved_path = saved.key_path.as_ref().map(PathBuf::from);
            if saved_path
                .as_ref()
                .map(|path| path.is_file())
                .unwrap_or(false)
            {
                return Ok(TokenPrep {
                    token_mode: "static".to_string(),
                    key_path: saved_path,
                    secret_storage: saved.secret_storage.clone(),
                });
            }
        }
    }
    Err("请输入 Access Key，或选择“此 Router 无需 Key”".to_string())
}

fn token_helper_command(paths: &InstallerPaths) -> Result<TokenHelperCommand, String> {
    let exe = env::current_exe().map_err(|error| format!("定位 Codex 助手失败: {error}"))?;
    Ok(TokenHelperCommand {
        command: exe.to_string_lossy().to_string(),
        args: vec![
            "--codex-assistant-token-helper".to_string(),
            paths
                .install_root
                .join("config.json")
                .to_string_lossy()
                .to_string(),
        ],
    })
}

fn write_model_catalog(paths: &InstallerPaths, state: &InstallState) -> Result<PathBuf, String> {
    let path = paths.install_root.join("models.json");
    let models = state
        .available_models
        .iter()
        .map(|model| {
            json!({
                "slug": model,
                "display_name": format!("{} · {}", model, state.provider_display_name),
                "description": format!("{} via {}", model, state.gateway_base_url),
                "default_reasoning_level": "medium",
                "supported_reasoning_levels": [
                    {"effort": "minimal", "description": "Minimal reasoning"},
                    {"effort": "low", "description": "Light reasoning"},
                    {"effort": "medium", "description": "Balanced reasoning"},
                    {"effort": "high", "description": "More reasoning"},
                    {"effort": "xhigh", "description": "Extra reasoning"}
                ],
                "shell_type": "shell_command",
                "visibility": "list",
                "supported_in_api": true,
                "priority": if model == &state.model { 0 } else { 10 },
                "base_instructions": "You are Codex, a coding agent. Follow the user's instructions and use the available tools carefully.",
                "supports_reasoning_summaries": false,
                "default_reasoning_summary": "none",
                "support_verbosity": false,
                "default_verbosity": "low",
                "truncation_policy": {"mode": "tokens", "limit": 10000},
                "supports_parallel_tool_calls": true,
                "experimental_supported_tools": [],
                "input_modalities": ["text"]
            })
        })
        .collect::<Vec<_>>();
    let data = serde_json::to_string_pretty(&json!({ "models": models }))
        .map_err(|error| format!("生成模型目录失败: {error}"))?;
    fs::write(&path, format!("{data}\n")).map_err(|error| format!("写入模型目录失败: {error}"))?;
    Ok(path)
}

fn write_state(paths: &InstallerPaths, state: &InstallState) -> Result<(), String> {
    fs::create_dir_all(&paths.install_root)
        .map_err(|error| format!("创建数据目录失败: {error}"))?;
    let data = serde_json::to_string_pretty(state)
        .map_err(|error| format!("生成助手状态失败: {error}"))?;
    fs::write(paths.install_root.join("config.json"), format!("{data}\n"))
        .map_err(|error| format!("写入助手状态失败: {error}"))
}

fn read_state(paths: &InstallerPaths) -> Result<InstallState, String> {
    let data = fs::read_to_string(paths.install_root.join("config.json"))
        .map_err(|error| error.to_string())?;
    serde_json::from_str(&data).map_err(|error| error.to_string())
}

fn write_codex_config(
    path: &Path,
    state: &InstallState,
    token_helper: Option<&TokenHelperCommand>,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("创建 Codex 配置目录失败: {error}"))?;
    }
    let existing = fs::read_to_string(path).unwrap_or_default();
    let cleaned = remove_managed_blocks(&existing);
    let mut document = if cleaned.trim().is_empty() {
        DocumentMut::new()
    } else {
        cleaned
            .parse::<DocumentMut>()
            .map_err(|error| format!("现有 Codex 配置不是有效 TOML: {error}"))?
    };

    document["model"] = value(state.model.as_str());
    document["model_provider"] = value(state.provider_id.as_str());
    if let Some(catalog) = &state.model_catalog_path {
        document["model_catalog_json"] = value(catalog.as_str());
    } else {
        document.as_table_mut().remove("model_catalog_json");
    }

    if document.get("model_providers").is_none() {
        document["model_providers"] = Item::Table(Table::new());
    }
    let providers = document["model_providers"]
        .as_table_mut()
        .ok_or_else(|| "Codex model_providers 必须是 TOML 表".to_string())?;
    for id in [
        PROVIDER_ID,
        "local_lmstudio",
        "local_ollama",
        "company_gateway",
        "custom_noauth",
        "company",
    ] {
        providers.remove(id);
    }

    let mut provider = Table::new();
    provider["name"] = value(state.provider_display_name.as_str());
    provider["base_url"] = value(state.gateway_base_url.as_str());
    provider["wire_api"] = value("responses");
    if state.token_mode != "none" {
        let helper = token_helper.ok_or_else(|| "缺少 Token Helper 配置".to_string())?;
        let mut args = Array::new();
        for arg in &helper.args {
            args.push(arg.as_str());
        }
        let mut auth = Table::new();
        auth["command"] = value(helper.command.as_str());
        auth["args"] = value(args);
        auth["timeout_ms"] = value(5000_i64);
        auth["refresh_interval_ms"] = value(300000_i64);
        provider["auth"] = Item::Table(auth);
    }
    providers.insert(PROVIDER_ID, Item::Table(provider));

    let mut output = document.to_string();
    if !output.ends_with('\n') {
        output.push('\n');
    }
    fs::write(path, output).map_err(|error| format!("写入 Codex 配置失败: {error}"))
}

fn remove_managed_blocks(content: &str) -> String {
    let provider_ids = [
        PROVIDER_ID,
        "local_lmstudio",
        "local_ollama",
        "company_gateway",
        "custom_noauth",
        "company",
    ];
    let mut kept = Vec::new();
    let mut skip_provider = false;
    let mut seen_table = false;
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    for line in normalized.lines() {
        let trim = line.trim();
        if matches!(
            trim,
            CONFIG_START | CONFIG_END | LEGACY_CONFIG_START | LEGACY_CONFIG_END
        ) {
            continue;
        }
        if is_provider_table(trim, &provider_ids) {
            skip_provider = true;
            seen_table = true;
            continue;
        }
        if skip_provider && is_table(trim) {
            skip_provider = false;
        }
        if skip_provider {
            continue;
        }
        if is_table(trim) {
            seen_table = true;
        }
        if !seen_table && is_managed_root_key(trim) {
            continue;
        }
        kept.push(line);
    }
    kept.join("\n").trim().to_string()
}

fn is_provider_table(line: &str, provider_ids: &[&str]) -> bool {
    provider_ids.iter().any(|id| {
        line == format!("[model_providers.{id}]") || line == format!("[model_providers.{id}.auth]")
    })
}

fn is_table(line: &str) -> bool {
    line.starts_with('[') && line.ends_with(']')
}

fn is_managed_root_key(line: &str) -> bool {
    line.starts_with("model =")
        || line.starts_with("model_provider =")
        || line.starts_with("model_catalog_json =")
}

fn codex_config_matches(path: &Path, state: &InstallState) -> bool {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| content.parse::<DocumentMut>().ok())
        .map(|document| {
            let provider = document
                .get("model_providers")
                .and_then(Item::as_table)
                .and_then(|providers| providers.get(PROVIDER_ID))
                .and_then(Item::as_table);
            document.get("model_provider").and_then(Item::as_str)
                == Some(state.provider_id.as_str())
                && document.get("model").and_then(Item::as_str) == Some(state.model.as_str())
                && provider
                    .and_then(|table| table.get("base_url"))
                    .and_then(Item::as_str)
                    == Some(state.gateway_base_url.as_str())
                && provider
                    .and_then(|table| table.get("wire_api"))
                    .and_then(Item::as_str)
                    == Some("responses")
        })
        .unwrap_or(false)
}

fn gateway_bearer_from_state(state: &InstallState) -> Result<Option<String>, String> {
    token_support::gateway_bearer_from_fields(
        &state.token_mode,
        state.key_path.as_deref(),
        state.secret_storage.as_deref(),
    )
}

fn fetch_models(gateway: &str, bearer: Option<&str>) -> Result<Vec<String>, String> {
    let endpoint = format!("{}/models", gateway.trim_end_matches('/'));
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(4))
        .timeout_read(Duration::from_secs(8))
        .timeout_write(Duration::from_secs(8))
        .build();
    let mut request = agent.get(&endpoint).set("Accept", "application/json");
    if let Some(token) = bearer {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }
    let response = request
        .call()
        .map_err(|error| http_error("GET /models", gateway, error))?;
    let body = response
        .into_string()
        .map_err(|error| format!("读取 /models 响应失败: {error}"))?;
    let payload: serde_json::Value =
        serde_json::from_str(&body).map_err(|error| format!("/models 未返回有效 JSON: {error}"))?;
    let mut models = payload
        .get("data")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(|value| value.as_str()))
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    models.sort();
    models.dedup();
    if models.is_empty() {
        if is_local_ollama_gateway(gateway) {
            return Err(
                "Ollama 已启动，但尚未返回模型。请先运行 ollama pull <模型名> 下载至少一个模型"
                    .to_string(),
            );
        }
        return Err("Router /models 没有返回可用模型".to_string());
    }
    Ok(models)
}

fn is_local_ollama_gateway(gateway: &str) -> bool {
    Url::parse(gateway)
        .ok()
        .map(|url| {
            matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"))
                && url.port_or_known_default() == Some(11434)
        })
        .unwrap_or(false)
}

fn is_ollama_gateway(gateway: &str) -> bool {
    Url::parse(gateway)
        .ok()
        .and_then(|url| url.port_or_known_default())
        == Some(11434)
}

fn local_ollama_connection_error() -> String {
    if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        return "未检测到 Windows 本机 Ollama 服务。当前是 Windows ARM64；请确认 Ollama 已安装并启动，或填写 macOS 宿主机可访问地址。127.0.0.1 只指向此 Windows VM".to_string();
    }
    "未检测到本机 Ollama 服务。请先安装并启动 Ollama；如果 Ollama 在虚拟机宿主机上，请填写宿主机可访问地址，不能使用 127.0.0.1".to_string()
}

fn remote_ollama_connection_error(gateway: &str) -> String {
    if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        return format!(
            "无法连接 Ollama：{gateway}。该地址的 11434 端口没有服务监听。若 Ollama 运行在 Parallels 的 macOS 宿主机，请先启动宿主机桥接，并使用 http://10.211.55.2:11434/v1；macOS Wi-Fi 地址不能替代未开放的 Ollama 监听地址"
        );
    }
    format!("无法连接 Ollama：{gateway}。请确认 Ollama 已启动，并监听当前设备可访问的网络接口")
}

fn http_error(operation: &str, gateway: &str, error: ureq::Error) -> String {
    match error {
        ureq::Error::Status(code, _) if code == 401 || code == 403 => {
            format!("{operation} 鉴权失败（HTTP {code}），请检查 Access Key")
        }
        ureq::Error::Status(code, _) => format!("{operation} 返回 HTTP {code}"),
        ureq::Error::Transport(_) if is_local_ollama_gateway(gateway) => {
            local_ollama_connection_error()
        }
        ureq::Error::Transport(_) if is_ollama_gateway(gateway) => {
            remote_ollama_connection_error(gateway)
        }
        ureq::Error::Transport(error) => format!("无法连接 Router：{error}"),
    }
}

fn normalize_gateway(input: &str) -> Result<String, String> {
    let trimmed = input.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Ok(DEFAULT_GATEWAY.to_string());
    }
    let mut parsed = Url::parse(trimmed).map_err(|_| "Router URL 格式不正确".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("Router URL 必须使用 http:// 或 https://".to_string());
    }
    if parsed.host_str().is_none() {
        return Err("Router URL 缺少主机地址".to_string());
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err("Router URL 不能包含查询参数或片段".to_string());
    }
    let current_path = parsed.path().trim_end_matches('/');
    let normalized_path = if current_path.is_empty() {
        "/v1".to_string()
    } else if current_path.ends_with("/v1") {
        current_path.to_string()
    } else {
        format!("{current_path}/v1")
    };
    parsed.set_path(&normalized_path);
    Ok(parsed.to_string().trim_end_matches('/').to_string())
}

fn resolve_winget() -> Result<String, String> {
    for name in ["winget.exe", "winget"] {
        if command_exists(name) {
            return Ok(name.to_string());
        }
    }
    if let Ok(local) = env::var("LOCALAPPDATA") {
        let path = Path::new(&local)
            .join("Microsoft")
            .join("WindowsApps")
            .join("winget.exe");
        if path.is_file() {
            return Ok(path.to_string_lossy().to_string());
        }
    }
    Err(
        "系统缺少 winget（Windows App Installer），无法调用 Microsoft Store 官方安装渠道"
            .to_string(),
    )
}

fn command_exists(name: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).any(|path| path.join(name).is_file()))
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn run_command_capture(program: &str, args: &[&str], timeout: Duration) -> Result<String, String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_console_window(&mut command);
    let mut child = command
        .spawn()
        .map_err(|error| format!("启动 {program} 失败: {error}"))?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut output = String::new();
                if let Some(mut stdout) = child.stdout.take() {
                    let _ = stdout.read_to_string(&mut output);
                }
                if let Some(mut stderr) = child.stderr.take() {
                    let _ = stderr.read_to_string(&mut output);
                }
                return if status.success() {
                    Ok(output)
                } else {
                    Err(format!("{program} 执行失败：{}", output.trim()))
                };
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                return Err(format!("{program} 超时（{} 秒）", timeout.as_secs()));
            }
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(error) => return Err(format!("等待 {program} 失败: {error}")),
        }
    }
}

fn run_command_stream(
    app: &AppHandle,
    label: &str,
    program: &str,
    args: &[&str],
    timeout: Duration,
    heartbeat: Duration,
) -> Result<(), String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_console_window(&mut command);
    let mut child = command
        .spawn()
        .map_err(|error| format!("启动 {label} 失败: {error}"))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_app = app.clone();
    let stdout_thread = stdout.map(|stream| {
        thread::spawn(move || {
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                emit_log(&stdout_app, format!("{}\n", redact_error(&line)));
            }
        })
    });
    let stderr_app = app.clone();
    let stderr_thread = stderr.map(|stream| {
        thread::spawn(move || {
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                emit_log(&stderr_app, format!("{}\n", redact_error(&line)));
            }
        })
    });
    let started = Instant::now();
    let mut last_heartbeat = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                return Err(format!("{label} 超时（{} 秒）", timeout.as_secs()));
            }
            Ok(None) => {
                if last_heartbeat.elapsed() >= heartbeat {
                    emit_log(
                        app,
                        format!(
                            "{label} 仍在进行，已等待 {} 秒\n",
                            started.elapsed().as_secs()
                        ),
                    );
                    last_heartbeat = Instant::now();
                }
                thread::sleep(Duration::from_millis(250));
            }
            Err(error) => return Err(format!("等待 {label} 失败: {error}")),
        }
    };
    if let Some(handle) = stdout_thread {
        let _ = handle.join();
    }
    if let Some(handle) = stderr_thread {
        let _ = handle.join();
    }
    if status.success() {
        Ok(())
    } else {
        Err(format!("{label} 未成功完成（{status}）"))
    }
}

fn hide_console_window(command: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(target_os = "windows"))]
    let _ = command;
}

fn platform_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else {
        "Desktop"
    }
}

fn unix_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn friendly_error(error: &str) -> String {
    redact_error(error).replace("Transport(Transport", "连接错误(")
}

fn redact_error(value: &str) -> String {
    let mut output = value.to_string();
    for marker in ["Bearer ", "token=", "key="] {
        while let Some(index) = output
            .to_ascii_lowercase()
            .find(&marker.to_ascii_lowercase())
        {
            let start = index + marker.len();
            let end = output[start..]
                .find(|ch: char| ch.is_whitespace() || matches!(ch, '&' | ',' | ';' | '"' | '\''))
                .map(|offset| start + offset)
                .unwrap_or(output.len());
            output.replace_range(start..end, "[redacted]");
            if end == output.len() {
                break;
            }
        }
    }
    output
}

fn emit_log(app: &AppHandle, line: impl Into<String>) {
    let _ = app.emit("installer-log", line.into());
}

#[allow(clippy::too_many_arguments)]
fn emit_stage(
    app: &AppHandle,
    stage: &str,
    label: &str,
    status: &str,
    message: impl Into<String>,
    current: usize,
    total: usize,
    recoverable: bool,
    details: serde_json::Value,
) {
    emit_stage_event(
        app,
        &stage_event(
            stage,
            label,
            status,
            message,
            current,
            total,
            recoverable,
            details,
        ),
    );
}

#[allow(clippy::too_many_arguments)]
fn stage_event(
    stage: impl Into<String>,
    label: impl Into<String>,
    status: impl Into<String>,
    message: impl Into<String>,
    current: usize,
    total: usize,
    recoverable: bool,
    details: serde_json::Value,
) -> StageEvent {
    StageEvent {
        stage: stage.into(),
        label: label.into(),
        status: status.into(),
        message: message.into(),
        current,
        total,
        recoverable,
        details,
    }
}

fn emit_stage_event(app: &AppHandle, event: &StageEvent) {
    let _ = app.emit("installer-stage", event.clone());
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            get_system_status,
            discover_models,
            start_setup,
            install_chatgpt_app,
            launch_chatgpt,
            restart_chatgpt,
            restore_codex_config,
            factory_reset,
            get_appearance_status,
            apply_appearance,
            import_theme_image,
            list_preset_themes,
            list_gallery_themes
        ])
        .run(tauri::generate_context!())
        .expect("error while running Codex Assistant app");
}

pub fn try_run_token_helper_from_args() -> Option<i32> {
    let mut args = env::args_os();
    let _ = args.next();
    let mode = args.next()?;
    if mode != "--codex-assistant-token-helper" && mode != "--company-codex-token-helper" {
        return None;
    }
    let Some(config_path) = args.next() else {
        eprintln!("Missing Codex Assistant state path");
        return Some(2);
    };
    match token_support::gateway_bearer_from_config(Path::new(&config_path)) {
        Ok(Some(token)) => {
            println!("{token}");
            Some(0)
        }
        Ok(None) => Some(0),
        Err(error) => {
            eprintln!("{error}");
            Some(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn art_theme_id_detection() {
        assert!(is_art_theme("preset:wukong"));
        assert!(is_art_theme("gallery:ver_018ad695fbb33b12e141"));
        assert!(!is_art_theme("focus"));
        assert!(!is_art_theme("custom"));
        assert!(!is_art_theme("presetx:wukong"));
    }

    #[test]
    fn gallery_version_id_validation() {
        assert!(validate_gallery_version_id("ver_018ad695fbb33b12e141").is_ok());
        assert!(validate_gallery_version_id("ver_8e695413").is_ok());
        assert!(validate_gallery_version_id("ver_short").is_err());
        assert!(validate_gallery_version_id("bad_018ad695fbb33b12e141").is_err());
        assert!(validate_gallery_version_id("ver_018AD695FBB33B12E141").is_err());
        assert!(validate_gallery_version_id("ver_../../etc/passwd00").is_err());
    }

    #[test]
    fn hex_color_parsing() {
        assert_eq!(parse_hex_rgb("#f6c696"), Some((246, 198, 150)));
        assert_eq!(parse_hex_rgb("#abc"), Some((170, 187, 204)));
        assert_eq!(parse_hex_rgb("#131313ff"), Some((19, 19, 19)));
        assert_eq!(parse_hex_rgb("rgba(1,2,3,.5)"), None);
        assert_eq!(parse_hex_rgb("#12"), None);
    }

    #[test]
    fn css_color_sanitization_blocks_injection() {
        assert_eq!(css_color("#f6c696", "#000"), "#f6c696");
        assert_eq!(
            css_color("rgba(102, 119, 111, 0.28)", "#000"),
            "rgba(102, 119, 111, 0.28)"
        );
        assert_eq!(css_color("red; } body {", "#000"), "#000");
        assert_eq!(css_color("url(https://evil)", "#000"), "#000");
        assert_eq!(css_color("", "#000"), "#000");
        assert_eq!(
            color_with_alpha("#f6c696", 0.5, "#000"),
            "rgba(246, 198, 150, 0.5)"
        );
        assert_eq!(
            color_with_alpha("rgba(9,9,9,.4)", 0.5, "#000"),
            "rgba(9,9,9,.4)"
        );
    }

    #[test]
    fn art_css_uses_theme_tokens() {
        let preset = ART_PRESETS
            .iter()
            .find(|preset| preset.slug == "wukong")
            .expect("wukong preset");
        let meta = ArtThemeMeta {
            name: preset.name.to_string(),
            appearance: preset.appearance.to_string(),
            focus_x: preset.focus_x,
            focus_y: preset.focus_y,
            colors: preset.colors.into(),
            ..ArtThemeMeta::default()
        };
        let css = render_art_css("art-wukong", &meta);
        assert!(css.contains("html[data-codex-assistant-theme=\"art-wukong\"]"));
        assert!(css.contains("color-scheme: dark"));
        assert!(css.contains("background-position: 0% 50%"));
        assert!(css.contains("#f0f0f0"));
        assert!(css.contains("#f6c696"));
        assert!(css.contains("rgba(29, 30, 29"));
    }

    #[test]
    fn art_css_light_theme_uses_light_scheme() {
        let mut meta = ArtThemeMeta {
            appearance: "light".to_string(),
            focus_x: 0.37,
            focus_y: 0.5,
            ..ArtThemeMeta::default()
        };
        meta.colors.text = "#1c1c1d".to_string();
        let css = render_art_css("art-g-test", &meta);
        assert!(css.contains("color-scheme: light"));
        assert!(css.contains("background-position: 37% 50%"));
        assert!(css.contains("#1c1c1d"));
    }

    #[test]
    fn injection_source_embeds_prepared_theme() {
        let prepared = PreparedTheme {
            attr: "art-wukong".to_string(),
            css: "html {}".to_string(),
            image_data_url: None,
        };
        let source = theme_injection_source(&prepared).expect("injection source");
        assert!(source.contains("codex-assistant-theme-style"));
        assert!(source.contains("\"art-wukong\""));
        assert!(!source.contains("imageDataUrl"));
        let with_image = PreparedTheme {
            attr: "custom".to_string(),
            css: "html {}".to_string(),
            image_data_url: Some("data:image/png;base64,AAAA".to_string()),
        };
        let source = theme_injection_source(&with_image).expect("injection source");
        assert!(source.contains("imageDataUrl"));
    }

    #[test]
    fn gateway_normalization_adds_v1() {
        assert_eq!(
            normalize_gateway("http://127.0.0.1:11434").unwrap(),
            DEFAULT_GATEWAY
        );
        assert_eq!(
            normalize_gateway("https://router.example.com/api/").unwrap(),
            "https://router.example.com/api/v1"
        );
    }

    #[test]
    fn configuration_snapshot_restores_all_managed_files() {
        let root = env::temp_dir().join(format!(
            "codex-assistant-backup-test-{}-{}",
            std::process::id(),
            unix_timestamp()
        ));
        let paths = InstallerPaths {
            install_root: root.join("runtime"),
            codex_config_path: root.join(".codex").join("config.toml"),
        };
        fs::create_dir_all(&paths.install_root).expect("create runtime directory");
        fs::create_dir_all(paths.codex_config_path.parent().expect("config parent"))
            .expect("create config directory");
        let state_path = paths.install_root.join("config.json");
        let models_path = paths.install_root.join("models.json");
        let key_path = paths.install_root.join("router-key.secret");
        fs::write(&paths.codex_config_path, "model = 'old'\n").expect("write old config");
        fs::write(&state_path, "old state\n").expect("write old state");
        fs::write(&models_path, "old models\n").expect("write old models");
        fs::write(&key_path, b"old key").expect("write old key");
        create_configuration_snapshot(&paths).expect("create snapshot");

        fs::write(&paths.codex_config_path, "model = 'new'\n").expect("write new config");
        fs::write(&state_path, "new state\n").expect("write new state");
        fs::write(&models_path, "new models\n").expect("write new models");
        fs::write(&key_path, b"new key").expect("write new key");
        restore_configuration_snapshot(&paths).expect("restore snapshot");

        assert_eq!(
            fs::read_to_string(&paths.codex_config_path).expect("read config"),
            "model = 'old'\n"
        );
        assert_eq!(
            fs::read_to_string(&state_path).expect("read state"),
            "old state\n"
        );
        assert_eq!(
            fs::read_to_string(&models_path).expect("read models"),
            "old models\n"
        );
        assert_eq!(fs::read(&key_path).expect("read key"), b"old key");

        fs::remove_dir_all(root).expect("cleanup test directory");
    }

    #[test]
    fn gateway_normalization_rejects_unsafe_schemes() {
        assert!(normalize_gateway("file:///tmp/router").is_err());
        assert!(normalize_gateway("router.example.com").is_err());
    }

    #[test]
    fn local_ollama_detection_is_loopback_and_port_specific() {
        assert!(is_local_ollama_gateway("http://127.0.0.1:11434/v1"));
        assert!(is_local_ollama_gateway("http://localhost:11434/v1"));
        assert!(!is_local_ollama_gateway("http://10.211.55.2:11434/v1"));
        assert!(!is_local_ollama_gateway("http://127.0.0.1:1234/v1"));
    }

    #[test]
    fn ollama_detection_accepts_remote_port() {
        assert!(is_ollama_gateway("http://10.211.55.2:11434/v1"));
        assert!(is_ollama_gateway("http://192.168.50.130:11434/v1"));
        assert!(!is_ollama_gateway("http://192.168.50.130:1234/v1"));
    }

    #[test]
    fn managed_cleanup_removes_legacy_provider() {
        let existing = r#"
approval_policy = "on-request"
model = "old-model"
model_provider = "local_ollama"

[model_providers.local_ollama]
name = "Local Ollama"
base_url = "http://127.0.0.1:11434/v1"

[profiles.keep]
model = "gpt-5"
"#;
        let cleaned = remove_managed_blocks(existing);
        assert!(cleaned.contains("approval_policy = \"on-request\""));
        assert!(cleaned.contains("[profiles.keep]"));
        assert!(!cleaned.contains("[model_providers.local_ollama]"));
        assert!(!cleaned.contains("127.0.0.1:11434"));
    }

    #[test]
    fn managed_cleanup_preserves_chatgpt_settings_inside_old_markers() {
        let existing = format!(
            r#"{CONFIG_START}
model = "old-model"
model_provider = "local_ollama"
notify = ["codex-computer-use.exe", "turn-ended"]

[model_providers.local_ollama]
base_url = "http://127.0.0.1:11434/v1"

[desktop]
followUpQueueMode = "queue"
{CONFIG_END}
"#
        );
        let cleaned = remove_managed_blocks(&existing);
        assert!(cleaned.contains("notify ="));
        assert!(cleaned.contains("[desktop]"));
        assert!(cleaned.contains("followUpQueueMode = \"queue\""));
        assert!(!cleaned.contains("local_ollama"));
    }

    #[test]
    fn clean_codex_config_removes_only_managed_settings() {
        let root = env::temp_dir().join(format!(
            "codex-assistant-clean-test-{}-{}",
            std::process::id(),
            unix_timestamp()
        ));
        fs::create_dir_all(&root).expect("create test directory");
        let config = root.join("config.toml");
        fs::write(
            &config,
            format!(
                r#"{CONFIG_START}
model = "llama3.1"
model_provider = "{PROVIDER_ID}"
model_catalog_json = "C:/models.json"

[model_providers.{PROVIDER_ID}]
name = "Codex Assistant Router"
base_url = "http://127.0.0.1:11434/v1"
wire_api = "responses"
{CONFIG_END}

[profiles.keep]
model = "gpt-5"
"#
            ),
        )
        .expect("write config");

        let changed = clean_codex_config(&config).expect("clean config");
        assert!(changed);
        let after = fs::read_to_string(&config).expect("read cleaned config");
        assert!(after.contains("[profiles.keep]"));
        assert!(after.contains("model = \"gpt-5\""));
        assert!(!after.contains(PROVIDER_ID));
        assert!(!after.contains("model_catalog_json"));
        assert!(!config_mentions_assistant(&config));

        let changed_again = clean_codex_config(&config).expect("second clean");
        assert!(!changed_again);
        fs::remove_dir_all(root).expect("cleanup test directory");
    }

    #[test]
    fn clean_codex_config_deletes_file_when_only_managed_content() {
        let root = env::temp_dir().join(format!(
            "codex-assistant-clean-empty-{}-{}",
            std::process::id(),
            unix_timestamp()
        ));
        fs::create_dir_all(&root).expect("create test directory");
        let config = root.join("config.toml");
        fs::write(
            &config,
            format!(
                "model = \"llama3.1\"\nmodel_provider = \"{PROVIDER_ID}\"\n\n[model_providers.{PROVIDER_ID}]\nbase_url = \"http://127.0.0.1:11434/v1\"\n"
            ),
        )
        .expect("write config");

        let changed = clean_codex_config(&config).expect("clean config");
        assert!(changed);
        assert!(!config.exists());

        let missing = clean_codex_config(&config).expect("clean missing config");
        assert!(!missing);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cdp_target_validation_is_loopback_only() {
        assert!(valid_cdp_websocket_url(
            "ws://127.0.0.1:9335/devtools/page/abc-123",
            9335
        ));
        assert!(!valid_cdp_websocket_url(
            "ws://192.168.1.10:9335/devtools/page/abc-123",
            9335
        ));
        assert!(!valid_cdp_websocket_url(
            "ws://127.0.0.1:9335/devtools/browser/abc-123",
            9335
        ));
    }

    #[test]
    fn focus_theme_payload_is_scoped_and_reversible() {
        let prepared = PreparedTheme {
            attr: "focus".to_string(),
            css: FOCUS_THEME_CSS.to_string(),
            image_data_url: None,
        };
        let source = theme_injection_source(&prepared).expect("focus payload");
        assert!(source.contains("codex-assistant-theme-style"));
        assert!(source.contains("data-codex-assistant-theme"));
        assert!(!source.contains("http://"));
    }

    #[test]
    fn custom_theme_requires_a_valid_background() {
        let prepared = PreparedTheme {
            attr: "custom".to_string(),
            css: CUSTOM_THEME_CSS.to_string(),
            image_data_url: Some("data:image/png;base64,AA==".to_string()),
        };
        let source = theme_injection_source(&prepared).expect("custom payload");
        assert!(source.contains("codex-assistant-art"));
        assert!(source.contains("data:image/png;base64,AA=="));
        assert!(source.contains("URL.createObjectURL"));
        assert!(source.contains("new Blob"));
        assert!(!source.contains("--codex-assistant-art: url("));
    }

    #[test]
    fn theme_image_validation_checks_type_and_dimensions() {
        let mut png = vec![0u8; 24];
        png[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        png[12..16].copy_from_slice(b"IHDR");
        png[16..20].copy_from_slice(&1920u32.to_be_bytes());
        png[20..24].copy_from_slice(&1080u32.to_be_bytes());
        assert_eq!(
            validate_theme_image("image/png", &png).unwrap(),
            ("png", 1920, 1080)
        );
        assert!(validate_theme_image("image/jpeg", &png).is_err());
        png[16..20].copy_from_slice(&20_000u32.to_be_bytes());
        assert!(validate_theme_image("image/png", &png).is_err());
    }

    #[test]
    fn windows_setup_probe_is_read_only_and_specific() {
        assert!(WINDOWS_SETUP_PROBE.contains("Finish Windows setup"));
        assert!(WINDOWS_SETUP_PROBE.contains("document.body?.innerText"));
        assert!(!WINDOWS_SETUP_PROBE.contains("click("));
    }

    #[test]
    fn generated_config_uses_responses_and_headless_helper() {
        let root = env::temp_dir().join(format!("codex-assistant-config-{}", unix_timestamp()));
        fs::create_dir_all(&root).unwrap();
        let state = InstallState {
            version: VERSION.to_string(),
            provider_id: PROVIDER_ID.to_string(),
            provider_display_name: PROVIDER_NAME.to_string(),
            model: "llama3.1".to_string(),
            gateway_base_url: DEFAULT_GATEWAY.to_string(),
            token_mode: "static".to_string(),
            wire_api: "responses".to_string(),
            available_models: vec!["llama3.1".to_string()],
            key_path: Some(root.join("router-key.secret").to_string_lossy().to_string()),
            secret_storage: Some(token_support::SECRET_STORAGE_DPAPI.to_string()),
            model_catalog_path: Some(root.join("models.json").to_string_lossy().to_string()),
            installed_at: unix_timestamp(),
        };
        let helper = TokenHelperCommand {
            command: "C:\\Program Files\\Codex 助手\\codex-assistant.exe".to_string(),
            args: vec![
                "--codex-assistant-token-helper".to_string(),
                "state.json".to_string(),
            ],
        };
        let path = root.join("config.toml");
        write_codex_config(&path, &state, Some(&helper)).unwrap();
        let config = fs::read_to_string(&path).unwrap();
        assert!(config.parse::<DocumentMut>().is_ok());
        assert!(config.contains("wire_api = \"responses\""));
        assert!(config.contains("[model_providers.codex_assistant_router.auth]"));
        assert!(config.contains("--codex-assistant-token-helper"));
        assert!(!config.contains("powershell.exe"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    #[ignore = "requires a running local Ollama server"]
    fn local_ollama_live_model_discovery() {
        let models = fetch_models(DEFAULT_GATEWAY, None).expect("query local Ollama");
        assert!(!models.is_empty());
    }
}
