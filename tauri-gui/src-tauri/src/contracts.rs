use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

pub const SCHEMA_VERSION_V1: u8 = 1;

#[derive(Clone, Debug)]
pub struct SystemStatusInput {
    pub platform: String,
    pub architecture: String,
    pub app_installed: bool,
    pub app_name: String,
    pub app_version: Option<String>,
    pub app_detail: String,
    pub config_present: bool,
    pub config_path: String,
    pub router_reachable: bool,
    pub router_detail: String,
    pub router_responses_verified: bool,
    pub router_last_verified_at: Option<String>,
    pub configured_gateway: Option<String>,
    pub configured_model: Option<String>,
    pub key_configured: bool,
    pub backup_available: bool,
    pub last_transaction_id: Option<String>,
    pub transaction_recovery_failed: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatusV1 {
    pub schema_version: u8,
    pub overall: String,
    pub platform: String,
    pub architecture: String,
    pub app: AppStatusV1,
    pub router: RouterStatusV1,
    pub config: ConfigStatusV1,
    pub recommended_action: RecommendedActionV1,
    #[serde(flatten)]
    pub legacy: LegacySystemStatusV1,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatusV1 {
    pub state: String,
    pub installed: bool,
    pub name: String,
    pub version: Option<String>,
    pub detail: String,
    pub trusted: bool,
    pub source: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouterStatusV1 {
    pub state: String,
    pub reachable: bool,
    pub detail: String,
    pub gateway: Option<String>,
    pub model: Option<String>,
    pub key_configured: bool,
    pub last_verified_at: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigStatusV1 {
    pub state: String,
    pub present: bool,
    pub path: String,
    pub effective_source: String,
    pub backup_available: bool,
    pub last_transaction_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedActionV1 {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacySystemStatusV1 {
    pub app_installed: bool,
    pub app_name: String,
    pub app_version: Option<String>,
    pub app_detail: String,
    pub config_present: bool,
    pub config_path: String,
    pub router_reachable: bool,
    pub router_detail: String,
    pub configured_gateway: Option<String>,
    pub configured_model: Option<String>,
    pub key_configured: bool,
    pub backup_available: bool,
    pub ready: bool,
}

impl SystemStatusV1 {
    pub fn from_input(input: SystemStatusInput) -> Self {
        let ready = !input.transaction_recovery_failed
            && input.app_installed
            && input.config_present
            && input.router_reachable
            && input.router_responses_verified;
        let overall = if input.transaction_recovery_failed {
            "blocked"
        } else if ready {
            "ready"
        } else if input.app_installed && input.config_present && !input.router_reachable {
            "blocked"
        } else {
            "action_required"
        };
        let recommended_action = recommended_action(&input, ready);
        let app_state = if input.app_installed {
            "installed"
        } else {
            "missing"
        };
        let router_state = if input.router_reachable && input.router_responses_verified {
            "responses_verified"
        } else if input.router_reachable {
            "models_verified"
        } else if input.configured_gateway.is_some() {
            "unreachable"
        } else {
            "not_configured"
        };
        let config_state = if input.transaction_recovery_failed {
            "rollback_failed"
        } else if input.config_present {
            "verified"
        } else {
            "missing"
        };

        Self {
            schema_version: SCHEMA_VERSION_V1,
            overall: overall.to_string(),
            platform: input.platform.clone(),
            architecture: input.architecture.clone(),
            app: AppStatusV1 {
                state: app_state.to_string(),
                installed: input.app_installed,
                name: input.app_name.clone(),
                version: input.app_version.clone(),
                detail: input.app_detail.clone(),
                trusted: input.app_installed,
                source: if input.app_installed {
                    "official-package"
                } else {
                    "not-detected"
                }
                .to_string(),
            },
            router: RouterStatusV1 {
                state: router_state.to_string(),
                reachable: input.router_reachable,
                detail: input.router_detail.clone(),
                gateway: input.configured_gateway.clone(),
                model: input.configured_model.clone(),
                key_configured: input.key_configured,
                last_verified_at: input.router_last_verified_at.clone(),
            },
            config: ConfigStatusV1 {
                state: config_state.to_string(),
                present: input.config_present,
                path: input.config_path.clone(),
                effective_source: if input.config_present { "user" } else { "none" }.to_string(),
                backup_available: input.backup_available,
                last_transaction_id: input.last_transaction_id.clone(),
            },
            recommended_action,
            legacy: LegacySystemStatusV1 {
                app_installed: input.app_installed,
                app_name: input.app_name,
                app_version: input.app_version,
                app_detail: input.app_detail,
                config_present: input.config_present,
                config_path: input.config_path,
                router_reachable: input.router_reachable,
                router_detail: input.router_detail,
                configured_gateway: input.configured_gateway,
                configured_model: input.configured_model,
                key_configured: input.key_configured,
                backup_available: input.backup_available,
                ready,
            },
        }
    }
}

fn recommended_action(input: &SystemStatusInput, ready: bool) -> RecommendedActionV1 {
    let (id, label) = if input.transaction_recovery_failed {
        ("open_diagnostics", "查看恢复详情")
    } else if ready {
        ("open_chatgpt", "打开 ChatGPT")
    } else if !input.app_installed && input.platform == "Windows" {
        ("install_chatgpt", "安装并配置")
    } else if !input.app_installed {
        ("open_install_guide", "查看安装说明")
    } else if !input.config_present {
        ("configure_router", "开始配置")
    } else {
        ("retry_router", "检查 Router")
    };
    RecommendedActionV1 {
        id: id.to_string(),
        label: label.to_string(),
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEnvelopeV1 {
    pub schema_version: u8,
    pub code: String,
    pub stage: String,
    pub title: String,
    pub message: String,
    pub recoverable: bool,
    pub suggested_action: String,
    pub support_id: String,
    pub technical: Value,
}

impl ErrorEnvelopeV1 {
    pub fn from_legacy(stage: impl Into<String>, safe_detail: impl Into<String>) -> Self {
        let stage = stage.into();
        let safe_detail = safe_detail.into();
        let code = classify_legacy_error(&stage, &safe_detail);
        let (title, message, recoverable, suggested_action) = error_copy(code);
        Self {
            schema_version: SCHEMA_VERSION_V1,
            code: code.to_string(),
            stage,
            title: title.to_string(),
            message: message.to_string(),
            recoverable,
            suggested_action: suggested_action.to_string(),
            support_id: support_id(),
            technical: json!({ "detail": redact_technical_detail(&safe_detail) }),
        }
    }
}

fn classify_legacy_error(stage: &str, detail: &str) -> &'static str {
    let lower = detail.to_lowercase();
    if stage.starts_with("appearance_") {
        classify_appearance_error(stage, &lower)
    } else if stage == "rollback" {
        "ROLLBACK_FAILED"
    } else if (lower.contains("publisher")
        && (lower.contains("mismatch")
            || lower.contains("untrusted")
            || lower.contains("unknown")
            || lower.contains("不匹配")
            || lower.contains("不可信")))
        || lower.contains("untrusted package")
        || lower.contains("package signature")
        || lower.contains("签名不可信")
        || lower.contains("发布者不匹配")
    {
        "APP_PACKAGE_UNTRUSTED"
    } else if lower.contains("restart required")
        || lower.contains("reboot required")
        || lower.contains("需要重启系统")
        || lower.contains("退出码 3010")
    {
        "APP_RESTART_REQUIRED"
    } else if lower.contains("official")
        && (lower.contains("still")
            || lower.contains("仍未检测到")
            || lower.contains("未检测到 chatgpt"))
    {
        "APP_INSTALL_FAILED"
    } else if lower.contains("未检测到 chatgpt") || lower.contains("chatgpt 尚未安装") {
        "APP_NOT_INSTALLED"
    } else if lower.contains("当前平台") || lower.contains("unsupported platform") {
        "UNSUPPORTED_PLATFORM"
    } else if lower.contains("router url")
        && (lower.contains("格式")
            || lower.contains("必须使用")
            || lower.contains("缺少主机")
            || lower.contains("查询参数"))
    {
        "ROUTER_URL_INVALID"
    } else if lower.contains("407")
        || lower.contains("proxy authentication")
        || lower.contains("proxy auth")
        || lower.contains("代理认证")
    {
        "PROXY_AUTH_REQUIRED"
    } else if lower.contains("请输入 access key")
        || lower.contains("401")
        || lower.contains("403")
        || lower.contains("unauthorized")
        || lower.contains("forbidden")
    {
        "ROUTER_AUTH_FAILED"
    } else if lower.contains("windows arm64")
        && lower.contains("127.0.0.1")
        && lower.contains("windows vm")
    {
        "ROUTER_VM_LOOPBACK"
    } else if lower.contains("未检测到本机 ollama") {
        "ROUTER_LOCAL_SERVICE_MISSING"
    } else if lower.contains("无法连接 ollama")
        && (lower.contains("parallels") || lower.contains("宿主机") || lower.contains("网络接口"))
    {
        "ROUTER_OLLAMA_HOST_UNREACHABLE"
    } else if lower.contains("connection refused")
        || lower.contains("actively refused")
        || lower.contains("os error 10061")
        || lower.contains("拒绝连接")
        || lower.contains("端口没有服务监听")
        || lower.contains("未检测到 windows 本机 ollama")
        || lower.contains("未检测到本机 ollama")
    {
        "ROUTER_CONNECTION_REFUSED"
    } else if lower.contains("dns")
        || lower.contains("resolve host")
        || lower.contains("no such host")
        || lower.contains("name or service not known")
        || lower.contains("nodename nor servname provided")
        || lower.contains("failed to lookup address information")
    {
        "ROUTER_DNS_FAILED"
    } else if lower.contains("timeout") || lower.contains("timed out") || lower.contains("超时") {
        "ROUTER_TIMEOUT"
    } else if lower.contains("certificate")
        || lower.contains("tls")
        || lower.contains("ssl")
        || lower.contains("证书")
    {
        "ROUTER_TLS_FAILED"
    } else if lower.contains("未返回模型") || lower.contains("模型已不在") {
        "ROUTER_MODEL_UNAVAILABLE"
    } else if stage == "validate_router_response"
        || lower.contains("/responses unsupported")
        || lower.contains("/responses 不兼容")
        || lower.contains("/responses 无法完成")
    {
        "ROUTER_RESPONSES_UNSUPPORTED"
    } else if (lower.contains("/models")
        && (lower.contains("json")
            || lower.contains("为空")
            || lower.contains("没有返回可用模型")
            || lower.contains("返回 http")
            || lower.contains("读取")))
        || lower.contains("ollama 已启动，但尚未返回模型")
    {
        "ROUTER_MODELS_INVALID"
    } else if lower.contains("permission denied")
        || lower.contains("access is denied")
        || lower.contains("权限")
    {
        "CONFIG_PERMISSION_DENIED"
    } else if lower.contains("toml")
        && (lower.contains("invalid") || lower.contains("not valid") || lower.contains("不是有效"))
    {
        "CONFIG_PARSE_FAILED"
    } else if lower.contains("config overridden")
        || lower.contains("managed config")
        || lower.contains("administrator config")
        || lower.contains("配置覆盖")
        || lower.contains("管理员配置")
    {
        "CONFIG_OVERRIDDEN"
    } else if lower.contains("安全保存 access key")
        || lower.contains("dpapi")
        || lower.contains("keychain")
    {
        "SECRET_STORE_FAILED"
    } else if lower.contains("复核失败") || lower.contains("verify") {
        "CONFIG_VERIFY_FAILED"
    } else if stage == "install_chatgpt" {
        "APP_INSTALL_FAILED"
    } else {
        "INTERNAL_TASK_FAILED"
    }
}

fn classify_appearance_error(stage: &str, lower: &str) -> &'static str {
    if lower.contains("未检测到 chatgpt")
        || lower.contains("chatgpt 尚未安装")
        || lower.contains("请先安装官方 chatgpt")
    {
        "APP_NOT_INSTALLED"
    } else if lower.contains("任务失败") || lower.contains("task failed") {
        "INTERNAL_TASK_FAILED"
    } else if lower.contains("permission denied")
        || lower.contains("access is denied")
        || lower.contains("权限")
        || lower.contains("保存外观状态失败")
        || lower.contains("创建外观状态目录失败")
        || lower.contains("保存主题")
        || lower.contains("提交主题")
    {
        "APPEARANCE_STORAGE_FAILED"
    } else if lower.contains("校验和不一致")
        || lower.contains("大小与清单不一致")
        || lower.contains("不是有效的 zip")
        || lower.contains("不安全路径")
        || lower.contains("文件数量过多")
        || lower.contains("缺少 theme.json")
        || lower.contains("主题包超过")
    {
        "APPEARANCE_PACKAGE_INVALID"
    } else if stage == "appearance_import"
        || lower.contains("mime")
        || lower.contains("base64")
        || lower.contains("图片不能超过")
        || lower.contains("图片尺寸过大")
        || lower.contains("无法识别主题图片")
    {
        "APPEARANCE_IMAGE_INVALID"
    } else if lower.contains("当前平台不支持")
        || lower.contains("不支持的主题")
        || lower.contains("unsupported appearance")
    {
        "APPEARANCE_UNSUPPORTED"
    } else if stage == "appearance_gallery" || lower.contains("在线主题库") {
        "APPEARANCE_GALLERY_UNAVAILABLE"
    } else if stage == "appearance_status" || stage == "appearance_presets" {
        "APPEARANCE_STATE_FAILED"
    } else {
        "APPEARANCE_APPLY_FAILED"
    }
}

fn error_copy(code: &str) -> (&'static str, &'static str, bool, &'static str) {
    match code {
        "APP_NOT_INSTALLED" => (
            "尚未安装 ChatGPT",
            "请先通过助手或 OpenAI 官方渠道安装 ChatGPT。",
            true,
            "install_app",
        ),
        "APP_INSTALL_FAILED" => (
            "ChatGPT 安装未完成",
            "官方安装流程未能完成，或安装后仍未检测到应用。",
            true,
            "retry_install",
        ),
        "APP_PACKAGE_UNTRUSTED" => (
            "ChatGPT 安装来源无法验证",
            "检测到的安装包发布者、签名或来源不符合可信要求。",
            false,
            "open_diagnostics",
        ),
        "APP_RESTART_REQUIRED" => (
            "需要重启 Windows",
            "系统需要完成重启后才能继续安装或检测 ChatGPT。",
            true,
            "restart_system",
        ),
        "UNSUPPORTED_PLATFORM" => (
            "当前平台暂不支持此操作",
            "请查看对应平台的官方安装说明。",
            false,
            "open_install_guide",
        ),
        "ROUTER_URL_INVALID" => (
            "Router 地址格式不正确",
            "请填写以 http:// 或 https:// 开头的完整服务地址。",
            true,
            "edit_gateway",
        ),
        "ROUTER_DNS_FAILED" => (
            "无法解析 Router 地址",
            "请检查服务地址、DNS 或企业网络设置。",
            true,
            "edit_gateway",
        ),
        "ROUTER_CONNECTION_REFUSED" => (
            "Router 拒绝连接",
            "请确认服务已启动、地址和端口正确，并允许当前设备访问。",
            true,
            "check_router",
        ),
        "ROUTER_TIMEOUT" => (
            "Router 响应超时",
            "服务暂时没有响应，请稍后重试或联系服务管理员。",
            true,
            "retry_router",
        ),
        "ROUTER_TLS_FAILED" => (
            "Router 安全连接失败",
            "请检查证书、系统时间或企业 CA 配置。",
            true,
            "open_diagnostics",
        ),
        "ROUTER_AUTH_FAILED" => (
            "Access Key 无效",
            "Router 可以访问，但拒绝了当前 Access Key。",
            true,
            "edit_key",
        ),
        "ROUTER_VM_LOOPBACK" => (
            "Windows ARM64 本机未检测到 Ollama",
            "127.0.0.1 只指向当前 Windows VM；请启动 Windows 内的 Ollama，或填写宿主机可访问地址。",
            true,
            "edit_gateway",
        ),
        "ROUTER_LOCAL_SERVICE_MISSING" => (
            "本机 Ollama 未启动",
            "请先安装并启动 Ollama；虚拟机内需要填写宿主机可访问地址。",
            true,
            "check_router",
        ),
        "ROUTER_OLLAMA_HOST_UNREACHABLE" => (
            "宿主机 Ollama 无法连接",
            "请确认 Ollama 已监听虚拟机可访问的接口，或启动 Parallels 专用桥接。",
            true,
            "edit_gateway",
        ),
        "ROUTER_MODELS_INVALID" => (
            "Router 模型接口不兼容",
            "服务没有返回可用的 OpenAI 兼容模型列表。",
            true,
            "check_router",
        ),
        "ROUTER_MODEL_UNAVAILABLE" => (
            "所选模型不可用",
            "请重新检测 Router 并选择当前可用模型。",
            true,
            "select_model",
        ),
        "ROUTER_RESPONSES_UNSUPPORTED" => (
            "Router 响应接口不兼容",
            "服务无法完成最小 Responses 请求，请检查兼容模式或联系服务管理员。",
            true,
            "check_router",
        ),
        "PROXY_AUTH_REQUIRED" => (
            "企业代理需要认证",
            "当前网络代理拒绝了未认证请求，请配置代理凭据后重试。",
            true,
            "configure_proxy",
        ),
        "CONFIG_PERMISSION_DENIED" => (
            "无法写入 Codex 配置",
            "当前用户没有所需文件权限，请修复权限后重试。",
            true,
            "repair_permissions",
        ),
        "CONFIG_PARSE_FAILED" => (
            "现有 Codex 配置无法读取",
            "配置文件格式异常，请先导出诊断或恢复备份。",
            true,
            "restore_config",
        ),
        "CONFIG_OVERRIDDEN" => (
            "Codex 配置被更高优先级覆盖",
            "项目或管理员配置覆盖了当前用户配置，请查看实际生效来源。",
            true,
            "show_effective_source",
        ),
        "CONFIG_VERIFY_FAILED" => (
            "Codex 配置复核失败",
            "写入结果没有通过复核，请恢复配置后重试。",
            true,
            "restore_config",
        ),
        "ROLLBACK_FAILED" => (
            "配置自动恢复失败",
            "请停止继续修改并联系支持人员处理现有文件和快照。",
            false,
            "contact_support",
        ),
        "SECRET_STORE_FAILED" => (
            "Access Key 无法安全保存",
            "系统凭据存储不可用，请修复后重试。",
            true,
            "open_diagnostics",
        ),
        "APPEARANCE_UNSUPPORTED" => (
            "当前环境不支持主题功能",
            "此平台、ChatGPT 版本或主题类型暂时无法使用。",
            false,
            "open_diagnostics",
        ),
        "APPEARANCE_IMAGE_INVALID" => (
            "主题图片无法使用",
            "请选择符合尺寸和格式要求的 PNG、JPEG 或 WebP 图片。",
            true,
            "choose_image",
        ),
        "APPEARANCE_GALLERY_UNAVAILABLE" => (
            "在线主题库暂时不可用",
            "请检查网络或代理设置，稍后重新加载主题库。",
            true,
            "retry_gallery",
        ),
        "APPEARANCE_PACKAGE_INVALID" => (
            "主题包未通过安全校验",
            "下载内容不完整或不符合安全要求，已停止应用。",
            true,
            "retry_gallery",
        ),
        "APPEARANCE_STORAGE_FAILED" => (
            "主题文件无法保存",
            "助手无法写入本地主题目录，请检查文件权限后重试。",
            true,
            "repair_permissions",
        ),
        "APPEARANCE_STATE_FAILED" => (
            "主题状态无法读取",
            "本地主题状态暂时不可用，请重新打开外观页面。",
            true,
            "retry_appearance",
        ),
        "APPEARANCE_APPLY_FAILED" => (
            "主题未能应用",
            "ChatGPT 未完成主题启动或页面注入，请恢复官方外观后重试。",
            true,
            "retry_appearance",
        ),
        _ => (
            "操作未完成",
            "助手遇到未分类错误，请重试并在仍失败时导出诊断。",
            true,
            "retry",
        ),
    }
}

fn redact_technical_detail(detail: &str) -> String {
    static BEARER: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"(?i)\bBearer\s+[^\s,;"']+"#).expect("bearer regex"));
    static ASSIGNMENT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?i)\b(access[_-]?key|api[_-]?key|token|key)=([^&\s,;"']+)"#)
            .expect("assignment regex")
    });
    static JSON_SECRET: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?i)("(?:access[_-]?key|api[_-]?key|token|key)"\s*:\s*")[^"]+"#)
            .expect("json secret regex")
    });
    static URL_USERINFO: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?i)(https?://)[^/\s:@]+(?::[^/\s@]*)?@"#).expect("url userinfo regex")
    });
    static WINDOWS_HOME: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?i)\b[A-Z]:\\Users\\[^\\\s/:*?"<>|]+"#).expect("windows home regex")
    });
    static MAC_HOME: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"/Users/[^/\s]+"#).expect("mac home regex"));
    static LINUX_HOME: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"/home/[^/\s]+"#).expect("linux home regex"));

    let output = BEARER.replace_all(detail, "Bearer [redacted]");
    let output = ASSIGNMENT.replace_all(&output, "$1=[redacted]");
    let output = JSON_SECRET.replace_all(&output, "$1[redacted]");
    let output = URL_USERINFO.replace_all(&output, "$1[redacted]@");
    let output = WINDOWS_HOME.replace_all(&output, "%USERPROFILE%");
    let output = MAC_HOME.replace_all(&output, "$$HOME");
    LINUX_HOME.replace_all(&output, "$$HOME").into_owned()
}

fn support_id() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let suffix = Uuid::new_v4().simple().to_string();
    format!("CA-{seconds}-{}", suffix[..8].to_ascii_uppercase())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SetupStageV1 {
    Preflight,
    InstallChatgpt,
    ValidateRouter,
    ValidateRouterResponse,
    ConfigureCodex,
    Verify,
    Rollback,
}

impl SetupStageV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Preflight => "preflight",
            Self::InstallChatgpt => "install_chatgpt",
            Self::ValidateRouter => "validate_router",
            Self::ValidateRouterResponse => "validate_router_response",
            Self::ConfigureCodex => "configure_codex",
            Self::Verify => "verify",
            Self::Rollback => "rollback",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum StageStatusV1 {
    Waiting,
    Running,
    Complete,
    Skipped,
    Failed,
    Restored,
}

impl StageStatusV1 {
    fn can_transition_to(self, next: Self) -> bool {
        self == next
            || matches!(
                (self, next),
                (Self::Waiting, Self::Running | Self::Skipped | Self::Failed)
                    | (
                        Self::Running,
                        Self::Complete | Self::Skipped | Self::Failed | Self::Restored
                    )
                    | (Self::Failed, Self::Restored)
            )
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StageEventV1 {
    pub schema_version: u8,
    pub operation_id: String,
    pub stage: SetupStageV1,
    pub label: String,
    pub status: StageStatusV1,
    pub message: String,
    pub current: usize,
    pub total: usize,
    pub cancellable: bool,
    pub recoverable: bool,
    pub details: Value,
}

impl StageEventV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn running(
        operation_id: impl Into<String>,
        stage: SetupStageV1,
        label: impl Into<String>,
        message: impl Into<String>,
        current: usize,
        total: usize,
        cancellable: bool,
        details: Value,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION_V1,
            operation_id: operation_id.into(),
            stage,
            label: label.into(),
            status: StageStatusV1::Running,
            message: message.into(),
            current,
            total,
            cancellable,
            recoverable: false,
            details,
        }
    }

    pub fn transition(
        &self,
        status: StageStatusV1,
        message: impl Into<String>,
        cancellable: bool,
        recoverable: bool,
        details: Value,
    ) -> Result<Self, String> {
        if !self.status.can_transition_to(status) {
            return Err(format!(
                "illegal workflow transition: {:?} -> {:?}",
                self.status, status
            ));
        }
        Ok(Self {
            schema_version: self.schema_version,
            operation_id: self.operation_id.clone(),
            stage: self.stage,
            label: self.label.clone(),
            status,
            message: message.into(),
            current: self.current,
            total: self.total,
            cancellable,
            recoverable,
            details,
        })
    }
}

pub fn new_operation_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status_input(
        app_installed: bool,
        config_present: bool,
        router_reachable: bool,
    ) -> SystemStatusInput {
        SystemStatusInput {
            platform: "Windows".to_string(),
            architecture: "aarch64".to_string(),
            app_installed,
            app_name: "ChatGPT".to_string(),
            app_version: Some("1.0".to_string()),
            app_detail: "test".to_string(),
            config_present,
            config_path: "C:/Users/test/.codex/config.toml".to_string(),
            router_reachable,
            router_detail: "test".to_string(),
            router_responses_verified: router_reachable,
            router_last_verified_at: router_reachable.then(|| "2026-07-28T00:00:00Z".to_string()),
            configured_gateway: config_present.then(|| "http://router/v1".to_string()),
            configured_model: config_present.then(|| "model".to_string()),
            key_configured: config_present,
            backup_available: false,
            last_transaction_id: None,
            transaction_recovery_failed: false,
        }
    }

    #[test]
    fn system_status_overall_and_action_are_core_owned() {
        let cases = [
            (false, false, false, "action_required", "install_chatgpt"),
            (false, false, true, "action_required", "install_chatgpt"),
            (false, true, false, "action_required", "install_chatgpt"),
            (false, true, true, "action_required", "install_chatgpt"),
            (true, false, false, "action_required", "configure_router"),
            (true, false, true, "action_required", "configure_router"),
            (true, true, false, "blocked", "retry_router"),
            (true, true, true, "ready", "open_chatgpt"),
        ];
        for (app, config, router, expected_overall, expected_action) in cases {
            let status = SystemStatusV1::from_input(status_input(app, config, router));
            assert_eq!(status.overall, expected_overall);
            assert_eq!(status.recommended_action.id, expected_action);
        }

        let mut mac_missing = status_input(false, false, false);
        mac_missing.platform = "macOS".to_string();
        let mac_status = SystemStatusV1::from_input(mac_missing);
        assert_eq!(mac_status.recommended_action.id, "open_install_guide");

        let mut models_only = status_input(true, true, true);
        models_only.router_responses_verified = false;
        models_only.router_last_verified_at = None;
        let models_only_status = SystemStatusV1::from_input(models_only);
        assert_eq!(models_only_status.overall, "action_required");
        assert_eq!(models_only_status.router.state, "models_verified");
        assert_eq!(models_only_status.recommended_action.id, "retry_router");

        let mut rollback_failed = status_input(true, true, true);
        rollback_failed.transaction_recovery_failed = true;
        rollback_failed.last_transaction_id = Some("tx-failed".to_string());
        let rollback_status = SystemStatusV1::from_input(rollback_failed);
        assert_eq!(rollback_status.overall, "blocked");
        assert_eq!(rollback_status.config.state, "rollback_failed");
        assert_eq!(
            rollback_status.config.last_transaction_id.as_deref(),
            Some("tx-failed")
        );
        assert_eq!(rollback_status.recommended_action.id, "open_diagnostics");
        assert!(!rollback_status.legacy.ready);
    }

    #[test]
    fn status_serialization_keeps_v1_and_legacy_fields() {
        let value =
            serde_json::to_value(SystemStatusV1::from_input(status_input(true, true, true)))
                .expect("serialize status");
        assert_eq!(value["schemaVersion"], 1);
        assert_eq!(value["overall"], "ready");
        assert_eq!(value["app"]["state"], "installed");
        assert_eq!(value["router"]["state"], "responses_verified");
        assert_eq!(value["router"]["lastVerifiedAt"], "2026-07-28T00:00:00Z");
        assert_eq!(value["config"]["state"], "verified");
        assert_eq!(value["ready"], true);
        assert_eq!(value["appInstalled"], true);
    }

    #[test]
    fn legacy_errors_map_to_stable_codes() {
        let refused = ErrorEnvelopeV1::from_legacy(
            "validate_router_models",
            "connect error: No connection could be made because the target machine actively refused it. (os error 10061)",
        );
        assert_eq!(refused.code, "ROUTER_CONNECTION_REFUSED");
        assert_eq!(refused.suggested_action, "check_router");

        let auth = ErrorEnvelopeV1::from_legacy("validate_router_models", "HTTP 401 Unauthorized");
        assert_eq!(auth.code, "ROUTER_AUTH_FAILED");
        assert!(auth.recoverable);

        let config =
            ErrorEnvelopeV1::from_legacy("configure_codex", "existing config is not valid TOML");
        assert_eq!(config.code, "CONFIG_PARSE_FAILED");

        let ollama = ErrorEnvelopeV1::from_legacy(
            "validate_router_models",
            "无法连接 Ollama：http://10.211.55.2:11434/v1。该地址的 11434 端口没有服务监听",
        );
        assert_eq!(ollama.code, "ROUTER_CONNECTION_REFUSED");

        let invalid_url =
            ErrorEnvelopeV1::from_legacy("validate_router_models", "Router URL 格式不正确");
        assert_eq!(invalid_url.code, "ROUTER_URL_INVALID");

        let no_models = ErrorEnvelopeV1::from_legacy(
            "validate_router_models",
            "Router /models 没有返回可用模型",
        );
        assert_eq!(no_models.code, "ROUTER_MODELS_INVALID");
    }

    #[test]
    fn platform_and_network_error_fixtures_map_to_stable_codes() {
        let cases = [
            (
                "validate_router_models",
                "No connection could be made because the target machine actively refused it. (os error 10061)",
                "ROUTER_CONNECTION_REFUSED",
            ),
            (
                "validate_router_models",
                "tcp connect failed: Connection refused (os error 61)",
                "ROUTER_CONNECTION_REFUSED",
            ),
            (
                "validate_router_models",
                "nodename nor servname provided, or not known",
                "ROUTER_DNS_FAILED",
            ),
            (
                "validate_router_models",
                "The operation timed out after 20 seconds",
                "ROUTER_TIMEOUT",
            ),
            (
                "validate_router_models",
                "certificate verify failed: unable to get local issuer certificate",
                "ROUTER_TLS_FAILED",
            ),
            (
                "validate_router_models",
                "HTTP 407 Proxy Authentication Required",
                "PROXY_AUTH_REQUIRED",
            ),
            (
                "validate_router_models",
                "未检测到 Windows 本机 Ollama 服务。当前是 Windows ARM64；127.0.0.1 只指向此 Windows VM",
                "ROUTER_VM_LOOPBACK",
            ),
            (
                "validate_router_models",
                "未检测到本机 Ollama 服务。请先安装并启动 Ollama",
                "ROUTER_LOCAL_SERVICE_MISSING",
            ),
            (
                "validate_router_models",
                "无法连接 Ollama：http://10.211.55.2:11434/v1。若 Ollama 运行在 Parallels 的 macOS 宿主机，请启动桥接",
                "ROUTER_OLLAMA_HOST_UNREACHABLE",
            ),
            (
                "validate_router_response",
                "POST /responses unsupported by upstream",
                "ROUTER_RESPONSES_UNSUPPORTED",
            ),
            (
                "install_chatgpt",
                "installer returned restart required (exit code 3010)",
                "APP_RESTART_REQUIRED",
            ),
            (
                "install_chatgpt",
                "package signature publisher mismatch",
                "APP_PACKAGE_UNTRUSTED",
            ),
            (
                "verify",
                "user config overridden by administrator config",
                "CONFIG_OVERRIDDEN",
            ),
        ];

        for (stage, detail, expected) in cases {
            let envelope = ErrorEnvelopeV1::from_legacy(stage, detail);
            assert_eq!(envelope.code, expected, "{stage}: {detail}");
            if expected == "ROUTER_VM_LOOPBACK" {
                assert!(envelope.message.contains("Windows VM"));
                assert!(envelope.message.contains("127.0.0.1"));
            }
        }
    }

    #[test]
    fn appearance_errors_use_separate_stable_codes() {
        let cases = [
            (
                "appearance_import",
                "主题图片 base64 内容无效",
                "APPEARANCE_IMAGE_INVALID",
            ),
            (
                "appearance_gallery",
                "连接在线主题库失败: timeout",
                "APPEARANCE_GALLERY_UNAVAILABLE",
            ),
            (
                "appearance_apply",
                "主题包校验和不一致，已中止",
                "APPEARANCE_PACKAGE_INVALID",
            ),
            (
                "appearance_apply",
                "当前平台不支持 ChatGPT 个性化",
                "APPEARANCE_UNSUPPORTED",
            ),
            (
                "appearance_apply",
                "45 秒内未能连接 ChatGPT 调试端口",
                "APPEARANCE_APPLY_FAILED",
            ),
        ];
        for (stage, detail, expected) in cases {
            assert_eq!(
                ErrorEnvelopeV1::from_legacy(stage, detail).code,
                expected,
                "{stage}: {detail}"
            );
        }
    }

    #[test]
    fn technical_detail_redacts_secrets_and_user_homes() {
        let error = ErrorEnvelopeV1::from_legacy(
            "validate_router_models",
            r#"request failed at https://alice:secret@example.test/v1/models?token=top-secret&key=second-secret with Bearer third-secret; {"accessKey":"fourth-secret"} C:\Users\alice\.codex\config.toml /Users/bob/.codex/config.toml /home/carol/.codex/config.toml"#,
        );
        let detail = error.technical["detail"]
            .as_str()
            .expect("technical detail string");
        for secret in [
            "alice:secret",
            "top-secret",
            "second-secret",
            "third-secret",
            "fourth-secret",
            r"C:\Users\alice",
            "/Users/bob",
            "/home/carol",
        ] {
            assert!(!detail.contains(secret), "leaked {secret}: {detail}");
        }
        assert!(detail.contains("[redacted]"));
        assert!(detail.contains("%USERPROFILE%"));
        assert!(detail.contains("$HOME"));
    }

    #[test]
    fn error_and_stage_serialization_are_versioned() {
        let error = ErrorEnvelopeV1::from_legacy("validate_router_models", "HTTP 401 Unauthorized");
        let error_value = serde_json::to_value(error).expect("serialize error");
        assert_eq!(error_value["schemaVersion"], 1);
        assert_eq!(error_value["code"], "ROUTER_AUTH_FAILED");
        assert!(error_value["supportId"]
            .as_str()
            .is_some_and(|value| value.starts_with("CA-")));

        let event = StageEventV1::running(
            "operation",
            SetupStageV1::ValidateRouter,
            "验证 Router",
            "正在验证",
            3,
            6,
            false,
            json!({}),
        );
        let event_value = serde_json::to_value(event).expect("serialize stage");
        assert_eq!(event_value["schemaVersion"], 1);
        assert_eq!(event_value["operationId"], "operation");
        assert_eq!(event_value["stage"], "validate_router");
        assert_eq!(event_value["status"], "running");
        assert_eq!(event_value["cancellable"], false);
    }

    #[test]
    fn workflow_rejects_illegal_terminal_transition() {
        let running = StageEventV1::running(
            "operation",
            SetupStageV1::Preflight,
            "检查",
            "进行中",
            1,
            5,
            false,
            json!({}),
        );
        let complete = running
            .transition(StageStatusV1::Complete, "完成", false, false, json!({}))
            .expect("running to complete");
        assert!(complete
            .transition(StageStatusV1::Running, "重复启动", false, false, json!({}))
            .is_err());
        assert!(complete
            .transition(StageStatusV1::Complete, "重复完成", false, false, json!({}))
            .is_ok());
    }
}
