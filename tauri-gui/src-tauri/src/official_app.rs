use std::path::Path;

#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::process::Command;
#[cfg(target_os = "windows")]
use std::time::Duration;

#[cfg(any(target_os = "windows", test))]
use serde::Deserialize;

#[cfg(any(target_os = "windows", test))]
const WINDOWS_PACKAGE_NAME: &str = "OpenAI.Codex";
#[cfg(any(target_os = "windows", test))]
const WINDOWS_PACKAGE_FAMILY: &str = "OpenAI.Codex_2p2nqsd0c76g0";
#[cfg(any(target_os = "windows", test))]
const WINDOWS_PUBLISHER: &str = "CN=50BDFD77-8903-4850-9FFE-6E8522F64D5B";
#[cfg(any(target_os = "windows", test))]
const WINDOWS_PUBLISHER_ID: &str = "2p2nqsd0c76g0";
#[cfg(target_os = "macos")]
const MACOS_BUNDLE_ID: &str = "com.openai.codex";
#[cfg(target_os = "macos")]
const MACOS_TEAM_ID: &str = "2DC432GLL2";

#[derive(Clone, Debug)]
pub(crate) struct DesktopAppInfo {
    pub(crate) installed: bool,
    pub(crate) state: String,
    pub(crate) trusted: bool,
    pub(crate) source: String,
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) detail: String,
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub(crate) package_family_name: Option<String>,
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub(crate) app_id: Option<String>,
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub(crate) executable_path: Option<String>,
}

impl DesktopAppInfo {
    fn missing(detail: impl Into<String>) -> Self {
        Self {
            installed: false,
            state: "not_installed".to_string(),
            trusted: false,
            source: "not-detected".to_string(),
            name: "ChatGPT".to_string(),
            version: None,
            detail: detail.into(),
            package_family_name: None,
            app_id: None,
            executable_path: None,
        }
    }

    fn needs_repair(
        detail: impl Into<String>,
        source: impl Into<String>,
        version: Option<String>,
        package_family_name: Option<String>,
        app_id: Option<String>,
        executable_path: Option<String>,
    ) -> Self {
        Self {
            installed: false,
            state: "needs_repair".to_string(),
            trusted: false,
            source: source.into(),
            name: "ChatGPT".to_string(),
            version,
            detail: detail.into(),
            package_family_name,
            app_id,
            executable_path,
        }
    }
}

pub(crate) fn detect_chatgpt_app() -> Result<DesktopAppInfo, String> {
    #[cfg(target_os = "windows")]
    {
        return detect_chatgpt_windows();
    }
    #[cfg(target_os = "macos")]
    {
        return detect_chatgpt_macos();
    }
    #[allow(unreachable_code)]
    Ok(DesktopAppInfo {
        installed: false,
        state: "unsupported".to_string(),
        trusted: false,
        source: "unsupported".to_string(),
        name: "ChatGPT".to_string(),
        version: None,
        detail: "当前平台不支持桌面应用检测".to_string(),
        package_family_name: None,
        app_id: None,
        executable_path: None,
    })
}

#[cfg(any(target_os = "windows", test))]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowsPackageEvidence {
    name: String,
    publisher: String,
    publisher_id: String,
    package_family_name: String,
    version: String,
    status: String,
    signature_kind: String,
    architecture: String,
    app_id: String,
    executable: String,
    executable_path: String,
    #[serde(default)]
    manifest_error: String,
}

#[cfg(any(target_os = "windows", test))]
fn classify_windows_evidence(
    evidence: WindowsPackageEvidence,
    executable_exists: bool,
) -> DesktopAppInfo {
    let version = non_empty(&evidence.version);
    let package_family_name = non_empty(&evidence.package_family_name);
    let app_id = non_empty(&evidence.app_id);
    let executable_path = non_empty(&evidence.executable_path);
    let mut problems = Vec::new();

    if evidence.name != WINDOWS_PACKAGE_NAME
        || evidence.publisher != WINDOWS_PUBLISHER
        || evidence.publisher_id != WINDOWS_PUBLISHER_ID
        || evidence.package_family_name != WINDOWS_PACKAGE_FAMILY
    {
        problems.push("发布者或包身份不匹配");
    }
    if evidence.signature_kind != "Store" {
        problems.push("签名类型不是 Microsoft Store");
    }
    if evidence.status != "Ok" {
        problems.push("软件包注册状态异常");
    }
    if !matches!(evidence.architecture.as_str(), "X64" | "Arm64") {
        problems.push("软件包架构不受支持");
    }
    if !valid_windows_version(&evidence.version) {
        problems.push("版本信息无效");
    }
    if !safe_app_id(&evidence.app_id) {
        problems.push("启动标识无效");
    }
    if !safe_chatgpt_executable(&evidence.executable) || !executable_exists {
        problems.push("程序文件缺失或路径异常");
    }
    if !evidence.manifest_error.trim().is_empty() {
        problems.push("应用清单无法读取");
    }

    if !problems.is_empty() {
        return DesktopAppInfo::needs_repair(
            format!("检测到 ChatGPT 软件包，但{}，需要修复", problems.join("、")),
            if evidence.signature_kind == "Store" {
                "microsoft-store"
            } else {
                "unknown-package"
            },
            version,
            package_family_name,
            app_id,
            executable_path,
        );
    }

    DesktopAppInfo {
        installed: true,
        state: "installed".to_string(),
        trusted: true,
        source: "microsoft-store".to_string(),
        name: "ChatGPT".to_string(),
        version: version.clone(),
        detail: format!(
            "Microsoft Store 官方应用{}",
            version
                .map(|value| format!(" · {value}"))
                .unwrap_or_default()
        ),
        package_family_name,
        app_id,
        executable_path,
    }
}

#[cfg(target_os = "windows")]
fn detect_chatgpt_windows() -> Result<DesktopAppInfo, String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
$pkg = Get-AppxPackage -Name 'OpenAI.Codex' -ErrorAction SilentlyContinue |
  Sort-Object Version -Descending |
  Select-Object -First 1
if ($pkg) {
  $appId = ''
  $executable = ''
  $executablePath = ''
  $manifestError = ''
  try {
    $manifest = Get-AppxPackageManifest -Package $pkg
    $app = @($manifest.Package.Applications.Application) |
      Where-Object { ('' + $_.Executable) -match '(ChatGPT|Codex)\.exe$' } |
      Select-Object -First 1
    if (-not $app) {
      $app = @($manifest.Package.Applications.Application) | Select-Object -First 1
    }
    if ($app) {
      $appId = '' + $app.Id
      $executable = '' + $app.Executable
      $executablePath = Join-Path $pkg.InstallLocation $executable
    }
  } catch {
    $manifestError = $_.Exception.Message
  }
  [ordered]@{
    name = '' + $pkg.Name
    publisher = '' + $pkg.Publisher
    publisherId = '' + $pkg.PublisherId
    packageFamilyName = '' + $pkg.PackageFamilyName
    version = '' + $pkg.Version
    status = $pkg.Status.ToString()
    signatureKind = $pkg.SignatureKind.ToString()
    architecture = $pkg.Architecture.ToString()
    appId = $appId
    executable = $executable
    executablePath = $executablePath
    manifestError = $manifestError
  } | ConvertTo-Json -Compress
}
"#;
    let output = crate::run_command_capture(
        "powershell.exe",
        &["-NoProfile", "-NonInteractive", "-Command", script],
        Duration::from_secs(12),
    )?;
    let json = output.trim().trim_start_matches('\u{feff}');
    if json.is_empty() {
        return Ok(DesktopAppInfo::missing(
            "未检测到 Microsoft Store 官方 ChatGPT",
        ));
    }
    let evidence: WindowsPackageEvidence = serde_json::from_str(json)
        .map_err(|error| format!("读取 ChatGPT 软件包证据失败: {error}"))?;
    let executable_exists =
        !evidence.executable_path.is_empty() && Path::new(&evidence.executable_path).is_file();
    Ok(classify_windows_evidence(evidence, executable_exists))
}

#[cfg(target_os = "macos")]
fn detect_chatgpt_macos() -> Result<DesktopAppInfo, String> {
    let mut candidates = vec![
        PathBuf::from("/Applications/ChatGPT.app"),
        PathBuf::from("/Applications/Codex.app"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        candidates.insert(1, PathBuf::from(home).join("Applications/ChatGPT.app"));
    }
    let Some(path) = candidates.into_iter().find(|candidate| candidate.is_dir()) else {
        return Ok(DesktopAppInfo::missing("未检测到 OpenAI 官方 ChatGPT 应用"));
    };

    let info_plist = path.join("Contents/Info.plist");
    let bundle_id = plutil_value(&info_plist, "CFBundleIdentifier");
    let version = plutil_value(&info_plist, "CFBundleShortVersionString");
    let executable_name = plutil_value(&info_plist, "CFBundleExecutable");
    let executable_path = executable_name
        .as_ref()
        .map(|name| path.join("Contents/MacOS").join(name));
    let team_id = codesign_team_id(&path);
    let mut problems = Vec::new();
    if bundle_id.as_deref() != Some(MACOS_BUNDLE_ID) {
        problems.push("Bundle ID 不匹配");
    }
    if team_id.as_deref() != Some(MACOS_TEAM_ID) {
        problems.push("签名团队不匹配");
    }
    if version.as_deref().is_none_or(str::is_empty) {
        problems.push("版本信息无效");
    }
    if executable_path
        .as_ref()
        .is_none_or(|value| !value.is_file())
    {
        problems.push("程序文件缺失");
    }

    let executable_path = executable_path.map(|value| value.to_string_lossy().to_string());
    if !problems.is_empty() {
        return Ok(DesktopAppInfo::needs_repair(
            format!("检测到 ChatGPT 应用，但{}，需要修复", problems.join("、")),
            "macos-bundle",
            version,
            None,
            bundle_id,
            executable_path,
        ));
    }

    Ok(DesktopAppInfo {
        installed: true,
        state: "installed".to_string(),
        trusted: true,
        source: "macos-signed-bundle".to_string(),
        name: "ChatGPT".to_string(),
        version: version.clone(),
        detail: format!(
            "OpenAI 签名应用{}",
            version
                .map(|value| format!(" · {value}"))
                .unwrap_or_default()
        ),
        package_family_name: None,
        app_id: bundle_id,
        executable_path,
    })
}

#[cfg(target_os = "macos")]
fn plutil_value(path: &Path, key: &str) -> Option<String> {
    let output = Command::new("/usr/bin/plutil")
        .args(["-extract", key, "raw", "-o", "-"])
        .arg(path)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(target_os = "macos")]
fn codesign_team_id(path: &Path) -> Option<String> {
    let output = Command::new("/usr/bin/codesign")
        .args(["-dv", "--verbose=4"])
        .arg(path)
        .output()
        .ok()?;
    let details = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    details.lines().find_map(|line| {
        line.trim()
            .strip_prefix("TeamIdentifier=")
            .map(str::trim)
            .filter(|value| !value.is_empty() && *value != "not set")
            .map(str::to_string)
    })
}

#[cfg(any(target_os = "windows", test))]
fn non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

#[cfg(any(target_os = "windows", test))]
fn valid_windows_version(value: &str) -> bool {
    let components = value.split('.').collect::<Vec<_>>();
    (3..=4).contains(&components.len())
        && components
            .iter()
            .all(|component| !component.is_empty() && component.parse::<u64>().is_ok())
}

#[cfg(any(target_os = "windows", test))]
fn safe_app_id(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

#[cfg(any(target_os = "windows", test))]
fn safe_chatgpt_executable(value: &str) -> bool {
    let normalized = value.trim().replace('\\', "/");
    !normalized.is_empty()
        && !normalized.starts_with('/')
        && !normalized.contains(':')
        && normalized
            .split('/')
            .all(|component| !component.is_empty() && component != "." && component != "..")
        && normalized
            .rsplit('/')
            .next()
            .is_some_and(|name| name.eq_ignore_ascii_case("ChatGPT.exe"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy_windows_evidence() -> WindowsPackageEvidence {
        WindowsPackageEvidence {
            name: WINDOWS_PACKAGE_NAME.to_string(),
            publisher: WINDOWS_PUBLISHER.to_string(),
            publisher_id: WINDOWS_PUBLISHER_ID.to_string(),
            package_family_name: WINDOWS_PACKAGE_FAMILY.to_string(),
            version: "26.721.4979.0".to_string(),
            status: "Ok".to_string(),
            signature_kind: "Store".to_string(),
            architecture: "Arm64".to_string(),
            app_id: "App".to_string(),
            executable: "app/ChatGPT.exe".to_string(),
            executable_path: "C:/Program Files/WindowsApps/OpenAI.Codex/app/ChatGPT.exe"
                .to_string(),
            manifest_error: String::new(),
        }
    }

    #[test]
    fn trusted_windows_store_package_is_installed() {
        let app = classify_windows_evidence(healthy_windows_evidence(), true);
        assert!(app.installed);
        assert!(app.trusted);
        assert_eq!(app.state, "installed");
        assert_eq!(app.source, "microsoft-store");
    }

    #[test]
    fn publisher_mismatch_is_not_trusted() {
        let mut evidence = healthy_windows_evidence();
        evidence.publisher = "CN=Other".to_string();
        let app = classify_windows_evidence(evidence, true);
        assert!(!app.installed);
        assert!(!app.trusted);
        assert_eq!(app.state, "needs_repair");
        assert!(app.detail.contains("发布者或包身份不匹配"));
    }

    #[test]
    fn broken_registration_or_executable_needs_repair() {
        let mut evidence = healthy_windows_evidence();
        evidence.status = "Error".to_string();
        evidence.app_id.clear();
        let app = classify_windows_evidence(evidence, false);
        assert_eq!(app.state, "needs_repair");
        assert!(app.detail.contains("软件包注册状态异常"));
        assert!(app.detail.contains("启动标识无效"));
        assert!(app.detail.contains("程序文件缺失或路径异常"));
    }

    #[test]
    fn rejects_unsafe_manifest_executable() {
        let mut evidence = healthy_windows_evidence();
        evidence.executable = "../ChatGPT.exe".to_string();
        let app = classify_windows_evidence(evidence, true);
        assert_eq!(app.state, "needs_repair");
        assert!(!app.trusted);
    }
}
