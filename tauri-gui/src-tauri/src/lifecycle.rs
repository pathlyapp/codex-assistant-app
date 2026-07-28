use serde::{Deserialize, Serialize};

use crate::contracts::SCHEMA_VERSION_V1;

pub const ACTION_UNINSTALL_ASSISTANT: &str = "uninstall_assistant";
pub const ACTION_RESTORE_PRE_ASSISTANT_CONFIG: &str = "restore_pre_assistant_config";
pub const ACTION_DELETE_ASSISTANT_DATA: &str = "delete_assistant_data";
pub const ACTION_OPEN_OFFICIAL_APP_MANAGEMENT: &str = "open_official_app_management";

const CONFIRM_UNINSTALL_ASSISTANT: &str = "UNINSTALL_ASSISTANT";
const CONFIRM_RESTORE_CONFIG: &str = "RESTORE_MANAGED_CONFIGURATION";
const CONFIRM_DELETE_DATA: &str = "DELETE_ASSISTANT_DATA";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleActionRequest {
    pub action_id: String,
    #[serde(default)]
    pub confirmation: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleSnapshotV1 {
    pub managed_config_present: bool,
    pub assistant_data_present: bool,
    pub official_app_installed: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleStatusV1 {
    pub schema_version: u8,
    pub assistant_uninstall_mode: String,
    pub assistant_uninstall_available: bool,
    pub managed_config_present: bool,
    pub assistant_data_present: bool,
    pub official_app_installed: bool,
    pub official_app_trusted: bool,
    pub data_removal_blocked: bool,
    pub default_preserves_config: bool,
    pub default_preserves_data: bool,
    pub default_preserves_official_app: bool,
}

impl LifecycleStatusV1 {
    pub fn new(
        assistant_uninstall_mode: impl Into<String>,
        assistant_uninstall_available: bool,
        managed_config_present: bool,
        assistant_data_present: bool,
        official_app_installed: bool,
        official_app_trusted: bool,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION_V1,
            assistant_uninstall_mode: assistant_uninstall_mode.into(),
            assistant_uninstall_available,
            managed_config_present,
            assistant_data_present,
            official_app_installed,
            official_app_trusted,
            data_removal_blocked: managed_config_present,
            default_preserves_config: true,
            default_preserves_data: true,
            default_preserves_official_app: true,
        }
    }

    pub fn snapshot(&self) -> LifecycleSnapshotV1 {
        LifecycleSnapshotV1 {
            managed_config_present: self.managed_config_present,
            assistant_data_present: self.assistant_data_present,
            official_app_installed: self.official_app_installed,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleActionResultV1 {
    pub schema_version: u8,
    pub action_id: String,
    pub status: String,
    pub changed: bool,
    pub app_exit_requested: bool,
    pub summary: String,
    pub before: LifecycleSnapshotV1,
    pub after: LifecycleSnapshotV1,
}

pub fn validate_action(
    request: &LifecycleActionRequest,
    status: &LifecycleStatusV1,
) -> Result<&'static str, String> {
    let action_id = request.action_id.trim();
    if !valid_action_id(action_id) {
        return Err("生命周期动作 ID 无效".to_string());
    }
    match action_id {
        ACTION_UNINSTALL_ASSISTANT => {
            require_confirmation(&request.confirmation, CONFIRM_UNINSTALL_ASSISTANT)?;
            if !status.assistant_uninstall_available {
                return Err("当前安装环境没有可信的助手卸载入口".to_string());
            }
            Ok(ACTION_UNINSTALL_ASSISTANT)
        }
        ACTION_RESTORE_PRE_ASSISTANT_CONFIG => {
            require_confirmation(&request.confirmation, CONFIRM_RESTORE_CONFIG)?;
            Ok(ACTION_RESTORE_PRE_ASSISTANT_CONFIG)
        }
        ACTION_DELETE_ASSISTANT_DATA => {
            require_confirmation(&request.confirmation, CONFIRM_DELETE_DATA)?;
            if status.data_removal_blocked {
                return Err("助手管理的 Codex 配置仍在使用本地数据，请先恢复原配置".to_string());
            }
            Ok(ACTION_DELETE_ASSISTANT_DATA)
        }
        ACTION_OPEN_OFFICIAL_APP_MANAGEMENT => Ok(ACTION_OPEN_OFFICIAL_APP_MANAGEMENT),
        _ => Err("生命周期动作 ID 无效".to_string()),
    }
}

fn valid_action_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn require_confirmation(value: &str, expected: &str) -> Result<(), String> {
    if value == expected {
        Ok(())
    } else {
        Err("该生命周期操作需要重新确认".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(config: bool, data: bool, uninstaller: bool) -> LifecycleStatusV1 {
        LifecycleStatusV1::new("nsis", uninstaller, config, data, true, true)
    }

    #[test]
    fn default_uninstall_preserves_all_user_state() {
        let value = status(true, true, true);
        assert!(value.default_preserves_config);
        assert!(value.default_preserves_data);
        assert!(value.default_preserves_official_app);
    }

    #[test]
    fn destructive_actions_require_exact_confirmation() {
        let request = LifecycleActionRequest {
            action_id: ACTION_DELETE_ASSISTANT_DATA.to_string(),
            confirmation: String::new(),
        };
        assert!(validate_action(&request, &status(false, true, true)).is_err());
    }

    #[test]
    fn data_cannot_be_deleted_while_managed_config_depends_on_it() {
        let request = LifecycleActionRequest {
            action_id: ACTION_DELETE_ASSISTANT_DATA.to_string(),
            confirmation: CONFIRM_DELETE_DATA.to_string(),
        };
        let error = validate_action(&request, &status(true, true, true)).unwrap_err();
        assert!(error.contains("先恢复原配置"));
    }

    #[test]
    fn official_app_management_is_a_separate_nondestructive_handoff() {
        let request = LifecycleActionRequest {
            action_id: ACTION_OPEN_OFFICIAL_APP_MANAGEMENT.to_string(),
            confirmation: String::new(),
        };
        assert_eq!(
            validate_action(&request, &status(true, true, true)).unwrap(),
            ACTION_OPEN_OFFICIAL_APP_MANAGEMENT
        );
    }

    #[test]
    fn malformed_or_unknown_action_is_rejected() {
        let request = LifecycleActionRequest {
            action_id: "../uninstall".to_string(),
            confirmation: CONFIRM_UNINSTALL_ASSISTANT.to_string(),
        };
        assert!(validate_action(&request, &status(true, true, true)).is_err());
    }
}
