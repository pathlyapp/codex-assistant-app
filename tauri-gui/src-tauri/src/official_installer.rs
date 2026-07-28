use serde::Serialize;
#[cfg(target_os = "windows")]
use std::time::{Duration, Instant};
use tauri::AppHandle;

#[cfg(target_os = "windows")]
use crate::official_app::{detect_chatgpt_app, DesktopAppInfo};

#[cfg(any(target_os = "windows", test))]
const WINDOWS_STORE_ID: &str = "9PLM9XGG6VKS";
#[cfg(target_os = "windows")]
const INSTALL_TIMEOUT: Duration = Duration::from_secs(12 * 60);
#[cfg(target_os = "windows")]
const VERIFY_TIMEOUT: Duration = Duration::from_secs(90);
#[cfg(target_os = "windows")]
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);

#[cfg(any(target_os = "windows", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OfficialInstallerKind {
    WingetStore,
}

#[cfg(any(target_os = "windows", test))]
impl OfficialInstallerKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::WingetStore => "winget-store",
        }
    }
}

// DEC-011 remains open. Only the already proven winget path is enabled until
// download licensing, redirect, signature, cancellation, and fallback gates close.
#[cfg(any(target_os = "windows", test))]
const WINDOWS_INSTALLER_POLICY: &[OfficialInstallerKind] = &[OfficialInstallerKind::WingetStore];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OfficialInstallerAvailability {
    pub(crate) adapter: String,
    pub(crate) source: String,
    pub(crate) available: bool,
    pub(crate) detail: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OfficialInstallReceipt {
    pub(crate) adapter: String,
    pub(crate) source: String,
    pub(crate) product_id: String,
    pub(crate) app_version: Option<String>,
    pub(crate) app_source: String,
    pub(crate) app_trusted: bool,
}

#[cfg(target_os = "windows")]
trait OfficialAppInstaller {
    fn kind(&self) -> OfficialInstallerKind;
    fn source(&self) -> &'static str;
    fn availability(&self) -> OfficialInstallerAvailability;
    fn install(&self, app: &AppHandle) -> Result<OfficialInstallReceipt, String>;
}

#[cfg(target_os = "windows")]
#[derive(Clone, Debug)]
struct WingetStoreInstaller {
    executable: Option<String>,
    unavailable_reason: Option<String>,
}

#[cfg(target_os = "windows")]
impl WingetStoreInstaller {
    fn discover() -> Self {
        match resolve_winget() {
            Ok(executable) => Self {
                executable: Some(executable),
                unavailable_reason: None,
            },
            Err(error) => Self {
                executable: None,
                unavailable_reason: Some(error),
            },
        }
    }
}

#[cfg(target_os = "windows")]
impl OfficialAppInstaller for WingetStoreInstaller {
    fn kind(&self) -> OfficialInstallerKind {
        OfficialInstallerKind::WingetStore
    }

    fn source(&self) -> &'static str {
        "microsoft-store"
    }

    fn availability(&self) -> OfficialInstallerAvailability {
        let available = self.executable.is_some();
        OfficialInstallerAvailability {
            adapter: self.kind().as_str().to_string(),
            source: self.source().to_string(),
            available,
            detail: if available {
                format!(
                    "Microsoft Store 官方安装渠道可用：{}",
                    self.executable.as_deref().unwrap_or("winget")
                )
            } else {
                self.unavailable_reason
                    .clone()
                    .unwrap_or_else(|| "winget 不可用".to_string())
            },
        }
    }

    fn install(&self, app: &AppHandle) -> Result<OfficialInstallReceipt, String> {
        let executable = self
            .executable
            .as_deref()
            .ok_or_else(|| self.availability().detail)?;
        crate::emit_log(
            app,
            "正在调用 Microsoft Store 官方安装渠道，此过程可能出现系统确认窗口。\n",
        );
        crate::run_command_stream(
            app,
            "winget install ChatGPT",
            executable,
            &winget_install_args(),
            INSTALL_TIMEOUT,
            HEARTBEAT_INTERVAL,
        )?;
        let detected = wait_for_trusted_chatgpt(VERIFY_TIMEOUT)?;
        Ok(OfficialInstallReceipt {
            adapter: self.kind().as_str().to_string(),
            source: self.source().to_string(),
            product_id: WINDOWS_STORE_ID.to_string(),
            app_version: detected.version,
            app_source: detected.source,
            app_trusted: detected.trusted,
        })
    }
}

pub(crate) fn preferred_installer_availability() -> Result<OfficialInstallerAvailability, String> {
    #[cfg(target_os = "windows")]
    {
        return Ok(selected_windows_installer()?.availability());
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("当前平台不支持自动安装 ChatGPT，请使用 OpenAI 官方渠道".to_string())
    }
}

pub(crate) fn install_official_chatgpt(app: &AppHandle) -> Result<OfficialInstallReceipt, String> {
    #[cfg(target_os = "windows")]
    {
        return selected_windows_installer()?.install(app);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        Err("当前平台不支持自动安装 ChatGPT，请使用 OpenAI 官方渠道".to_string())
    }
}

#[cfg(target_os = "windows")]
fn selected_windows_installer() -> Result<Box<dyn OfficialAppInstaller>, String> {
    let kind = WINDOWS_INSTALLER_POLICY
        .first()
        .copied()
        .ok_or_else(|| "未配置可用的 Windows 官方应用安装适配器".to_string())?;
    match kind {
        OfficialInstallerKind::WingetStore => Ok(Box::new(WingetStoreInstaller::discover())),
    }
}

#[cfg(any(target_os = "windows", test))]
fn winget_install_args() -> [&'static str; 8] {
    [
        "install",
        "--id",
        WINDOWS_STORE_ID,
        "-e",
        "-s",
        "msstore",
        "--accept-source-agreements",
        "--accept-package-agreements",
    ]
}

#[cfg(target_os = "windows")]
fn wait_for_trusted_chatgpt(timeout: Duration) -> Result<DesktopAppInfo, String> {
    let deadline = Instant::now() + timeout;
    loop {
        let detected = detect_chatgpt_app()?;
        if detected.installed && detected.trusted {
            return Ok(detected);
        }
        if detected.state == "needs_repair" {
            return Err(format!(
                "官方安装命令已结束，但 ChatGPT 软件包未通过可信检测：{}",
                detected.detail
            ));
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "官方安装命令已结束，但 {} 秒内仍未检测到可信 ChatGPT 软件包",
                timeout.as_secs()
            ));
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

#[cfg(target_os = "windows")]
fn resolve_winget() -> Result<String, String> {
    for name in ["winget.exe", "winget"] {
        if command_exists(name) {
            return Ok(name.to_string());
        }
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let path = std::path::Path::new(&local)
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

#[cfg(target_os = "windows")]
fn command_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|path| path.join(name).is_file()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installer_kind_ids_are_stable() {
        assert_eq!(OfficialInstallerKind::WingetStore.as_str(), "winget-store");
    }

    #[test]
    fn current_policy_does_not_enable_unverified_download_adapters() {
        assert_eq!(
            WINDOWS_INSTALLER_POLICY,
            &[OfficialInstallerKind::WingetStore]
        );
    }

    #[test]
    fn winget_arguments_pin_official_store_product() {
        assert_eq!(
            winget_install_args(),
            [
                "install",
                "--id",
                "9PLM9XGG6VKS",
                "-e",
                "-s",
                "msstore",
                "--accept-source-agreements",
                "--accept-package-agreements",
            ]
        );
    }

    #[test]
    fn install_receipt_uses_versioned_field_names() {
        let receipt = OfficialInstallReceipt {
            adapter: "winget-store".to_string(),
            source: "microsoft-store".to_string(),
            product_id: WINDOWS_STORE_ID.to_string(),
            app_version: Some("26.721.4979.0".to_string()),
            app_source: "microsoft-store".to_string(),
            app_trusted: true,
        };
        let value = serde_json::to_value(receipt).expect("serialize receipt");
        assert_eq!(value["adapter"], "winget-store");
        assert_eq!(value["productId"], WINDOWS_STORE_ID);
        assert_eq!(value["appTrusted"], true);
    }
}
