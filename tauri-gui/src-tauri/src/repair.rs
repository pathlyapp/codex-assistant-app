use serde::{Deserialize, Serialize};

use crate::contracts::{SystemStatusV1, SCHEMA_VERSION_V1};

pub const ACTION_RECHECK_OFFICIAL_APP: &str = "recheck_official_app";
pub const ACTION_REVALIDATE_ROUTER: &str = "revalidate_router";
pub const ACTION_RESTORE_CONFIGURATION: &str = "restore_configuration";
pub const ACTION_CLEAR_APPEARANCE_SESSION: &str = "clear_appearance_session";

const ROUTER_ERROR_CODES: &[&str] = &[
    "ROUTER_DNS_FAILED",
    "ROUTER_CONNECTION_REFUSED",
    "ROUTER_TIMEOUT",
    "ROUTER_TLS_FAILED",
    "ROUTER_AUTH_FAILED",
    "ROUTER_VM_LOOPBACK",
    "ROUTER_LOCAL_SERVICE_MISSING",
    "ROUTER_OLLAMA_HOST_UNREACHABLE",
    "ROUTER_MODELS_INVALID",
    "ROUTER_MODEL_UNAVAILABLE",
    "ROUTER_RESPONSES_UNSUPPORTED",
    "PROXY_AUTH_REQUIRED",
];
const CONFIG_ERROR_CODES: &[&str] = &[
    "CONFIG_PERMISSION_DENIED",
    "CONFIG_PARSE_FAILED",
    "CONFIG_VERIFY_FAILED",
];
const APPEARANCE_ERROR_CODES: &[&str] = &[
    "APPEARANCE_APPLY_FAILED",
    "APPEARANCE_STATE_FAILED",
    "APPEARANCE_STORAGE_FAILED",
];

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairPlanRequest {
    #[serde(default)]
    pub error_code: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairRunRequest {
    pub action_id: String,
    #[serde(default)]
    pub error_code: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairActionV1 {
    pub id: String,
    pub label: String,
    pub description: String,
    pub requires_confirmation: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairPlanV1 {
    pub schema_version: u8,
    pub state: String,
    pub title: String,
    pub detail: String,
    pub error_code: Option<String>,
    pub action: Option<RepairActionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairSnapshotV1 {
    pub overall: String,
    pub app_state: String,
    pub router_state: String,
    pub config_state: String,
    pub appearance_state: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairResultV1 {
    pub schema_version: u8,
    pub action_id: String,
    pub success: bool,
    pub changed: bool,
    pub summary: String,
    pub before: RepairSnapshotV1,
    pub after: RepairSnapshotV1,
}

pub fn plan_repair(
    status: &SystemStatusV1,
    error_code: &str,
    appearance_state: &str,
) -> RepairPlanV1 {
    let error_code = normalize_error_code(error_code);
    let action = if status.app.state == "needs_repair" {
        Some(action(
            ACTION_RECHECK_OFFICIAL_APP,
            "重新检测官方应用",
            "重新检查 Microsoft Store 或 macOS 的应用信息；不会覆盖或卸载现有应用。",
            false,
        ))
    } else if status.config.state == "rollback_failed" {
        None
    } else if APPEARANCE_ERROR_CODES.contains(&error_code.as_str())
        && appearance_state != "official"
    {
        Some(action(
            ACTION_CLEAR_APPEARANCE_SESSION,
            "恢复官方外观",
            "清除失效的主题状态；不会修改 ChatGPT 安装文件或 Codex 设置。",
            true,
        ))
    } else if CONFIG_ERROR_CODES.contains(&error_code.as_str())
        && status.config.state != "verified"
        && status.config.backup_available
    {
        Some(action(
            ACTION_RESTORE_CONFIGURATION,
            "恢复上次的完整设置",
            "从备份中恢复助手管理的设置；执行前会再次备份当前状态。",
            true,
        ))
    } else if status.config.present
        && status.router.state != "responses_verified"
        && (ROUTER_ERROR_CODES.contains(&error_code.as_str())
            || matches!(
                status.router.state.as_str(),
                "unreachable" | "models_verified"
            ))
    {
        Some(action(
            ACTION_REVALIDATE_ROUTER,
            "重新检查模型服务",
            "用已保存的地址、模型和密钥重新检查服务，不会改动 Codex 设置。",
            false,
        ))
    } else {
        None
    };

    let (state, title, detail) = if let Some(action) = action.as_ref() {
        (
            "action_available",
            "已找到针对当前问题的修复",
            action.description.as_str(),
        )
    } else if status.overall == "ready" {
        (
            "not_needed",
            "当前无需修复",
            "ChatGPT、模型服务和 Codex 设置都已通过检查。",
        )
    } else if status.config.state == "rollback_failed" {
        (
            "manual_required",
            "自动恢复未完成",
            "为了避免覆盖备份，助手没有继续自动修改。请先导出诊断包。",
        )
    } else {
        (
            "manual_required",
            "需要人工确认",
            "当前问题暂无安全的自动修复动作，请导出诊断包，或返回设置页处理。",
        )
    };

    RepairPlanV1 {
        schema_version: SCHEMA_VERSION_V1,
        state: state.to_string(),
        title: title.to_string(),
        detail: detail.to_string(),
        error_code: (!error_code.is_empty()).then_some(error_code),
        action,
    }
}

pub fn snapshot(status: &SystemStatusV1, appearance_state: &str) -> RepairSnapshotV1 {
    RepairSnapshotV1 {
        overall: status.overall.clone(),
        app_state: status.app.state.clone(),
        router_state: status.router.state.clone(),
        config_state: status.config.state.clone(),
        appearance_state: appearance_state.to_string(),
    }
}

fn action(id: &str, label: &str, description: &str, requires_confirmation: bool) -> RepairActionV1 {
    RepairActionV1 {
        id: id.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        requires_confirmation,
    }
}

fn normalize_error_code(value: &str) -> String {
    let value = value.trim();
    if value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
    {
        value.to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::SystemStatusInput;

    fn status(
        app_state: &str,
        config_present: bool,
        router_reachable: bool,
        responses_verified: bool,
        backup_available: bool,
        recovery_failed: bool,
    ) -> SystemStatusV1 {
        SystemStatusV1::from_input(SystemStatusInput {
            platform: "Windows".to_string(),
            architecture: "aarch64".to_string(),
            app_installed: app_state == "installed",
            app_state: app_state.to_string(),
            app_trusted: app_state == "installed",
            app_source: "microsoft-store".to_string(),
            app_name: "ChatGPT".to_string(),
            app_version: Some("1.2.3.4".to_string()),
            app_detail: "fixture".to_string(),
            config_present,
            config_path: "fixture".to_string(),
            router_reachable,
            router_detail: "fixture".to_string(),
            router_responses_verified: responses_verified,
            router_last_verified_at: responses_verified.then(|| "2026-07-28T00:00:00Z".into()),
            configured_gateway: config_present.then(|| "http://router.test/v1".into()),
            configured_model: config_present.then(|| "model-a".into()),
            key_configured: false,
            backup_available,
            last_transaction_id: None,
            transaction_recovery_failed: recovery_failed,
        })
    }

    #[test]
    fn ready_status_has_no_repair_action() {
        let plan = plan_repair(
            &status("installed", true, true, true, true, false),
            "ROUTER_RESPONSES_UNSUPPORTED",
            "official",
        );
        assert_eq!(plan.state, "not_needed");
        assert!(plan.action.is_none());
    }

    #[test]
    fn router_failure_maps_to_revalidation() {
        let plan = plan_repair(
            &status("installed", true, false, false, true, false),
            "ROUTER_CONNECTION_REFUSED",
            "official",
        );
        assert_eq!(
            plan.action.expect("repair action").id,
            ACTION_REVALIDATE_ROUTER
        );
    }

    #[test]
    fn config_failure_maps_to_confirmed_restore() {
        let plan = plan_repair(
            &status("installed", false, false, false, true, false),
            "CONFIG_PARSE_FAILED",
            "official",
        );
        let action = plan.action.expect("repair action");
        assert_eq!(action.id, ACTION_RESTORE_CONFIGURATION);
        assert!(action.requires_confirmation);
    }

    #[test]
    fn rollback_failure_never_offers_automatic_mutation() {
        let plan = plan_repair(
            &status("installed", false, false, false, true, true),
            "ROLLBACK_FAILED",
            "official",
        );
        assert_eq!(plan.state, "manual_required");
        assert!(plan.action.is_none());
    }

    #[test]
    fn stale_theme_error_maps_to_session_clear() {
        let plan = plan_repair(
            &status("installed", true, true, true, true, false),
            "APPEARANCE_APPLY_FAILED",
            "custom",
        );
        assert_eq!(
            plan.action.expect("repair action").id,
            ACTION_CLEAR_APPEARANCE_SESSION
        );
    }

    #[test]
    fn malformed_error_code_is_not_used_for_action_selection() {
        let plan = plan_repair(
            &status("installed", false, false, false, true, false),
            "CONFIG_PARSE_FAILED<script>",
            "official",
        );
        assert!(plan.error_code.is_none());
        assert!(plan.action.is_none());
    }
}
