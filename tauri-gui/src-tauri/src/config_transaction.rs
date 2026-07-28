use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const TRANSACTION_SCHEMA_VERSION: u8 = 2;
const ACTIVE_TRANSACTION_FILE: &str = "transaction.active.json";
const LAST_TRANSACTION_FILE: &str = "transaction.last.json";
const SNAPSHOT_DIRECTORY: &str = "snapshots";

#[derive(Clone, Debug)]
pub struct ManagedFile {
    pub id: String,
    pub path: PathBuf,
}

impl ManagedFile {
    pub fn new(id: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    SnapshotCreated,
    Writing,
    Committed,
    RolledBack,
    RollbackFailed,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotFile {
    pub id: String,
    pub target_path: String,
    pub existed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionManifest {
    pub schema_version: u8,
    pub transaction_id: String,
    pub operation: String,
    pub created_at: String,
    #[serde(default)]
    pub created_order: String,
    pub app_version: String,
    pub status: TransactionStatus,
    pub files: Vec<SnapshotFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActiveTransaction {
    schema_version: u8,
    transaction_id: String,
    manifest_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSummary {
    pub schema_version: u8,
    pub transaction_id: String,
    pub operation: String,
    pub status: TransactionStatus,
    pub manifest_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
}

#[derive(Debug)]
pub struct ConfigTransaction {
    install_root: PathBuf,
    manifest_path: PathBuf,
    manifest: TransactionManifest,
    managed_files: Vec<ManagedFile>,
}

impl ConfigTransaction {
    pub fn begin(
        install_root: &Path,
        transaction_id: &str,
        operation: &str,
        created_at: &str,
        app_version: &str,
        managed_files: &[ManagedFile],
    ) -> Result<Self, String> {
        validate_transaction_id(transaction_id)?;
        validate_managed_files(managed_files)?;
        fs::create_dir_all(install_root).map_err(|error| format!("创建事务目录失败: {error}"))?;
        let active_path = active_transaction_path(install_root);
        if active_path.exists() {
            return Err(format!(
                "检测到未完成配置事务，请先恢复: {}",
                active_path.display()
            ));
        }

        let snapshot_root = snapshot_root(install_root, transaction_id);
        fs::create_dir_all(&snapshot_root)
            .map_err(|error| format!("创建事务快照目录失败: {error}"))?;
        let result = (|| {
            let mut files = Vec::with_capacity(managed_files.len());
            for managed in managed_files {
                let existed = managed.path.is_file();
                if managed.path.exists() && !existed {
                    return Err(format!(
                        "受管配置路径不是普通文件: {}",
                        managed.path.display()
                    ));
                }
                let (backup_file, sha256) = if existed {
                    let data = fs::read(&managed.path)
                        .map_err(|error| format!("读取 {} 快照失败: {error}", managed.id))?;
                    let backup_file = format!("{}.bak", managed.id);
                    atomic_write(&snapshot_root.join(&backup_file), &data)?;
                    (Some(backup_file), Some(sha256_hex(&data)))
                } else {
                    (None, None)
                };
                files.push(SnapshotFile {
                    id: managed.id.clone(),
                    target_path: managed.path.to_string_lossy().to_string(),
                    existed,
                    backup_file,
                    sha256,
                });
            }

            let manifest_path = snapshot_root.join("manifest.json");
            let manifest = TransactionManifest {
                schema_version: TRANSACTION_SCHEMA_VERSION,
                transaction_id: transaction_id.to_string(),
                operation: operation.to_string(),
                created_at: created_at.to_string(),
                created_order: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|error| format!("生成事务顺序失败: {error}"))?
                    .as_nanos()
                    .to_string(),
                app_version: app_version.to_string(),
                status: TransactionStatus::SnapshotCreated,
                files,
                completed_at: None,
                failure: None,
            };
            write_json(&manifest_path, &manifest)?;
            write_json(
                &active_path,
                &ActiveTransaction {
                    schema_version: TRANSACTION_SCHEMA_VERSION,
                    transaction_id: transaction_id.to_string(),
                    manifest_path: manifest_path.to_string_lossy().to_string(),
                },
            )?;
            Ok(Self {
                install_root: install_root.to_path_buf(),
                manifest_path,
                manifest,
                managed_files: managed_files.to_vec(),
            })
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(&snapshot_root);
        }
        result
    }

    pub fn transaction_id(&self) -> &str {
        &self.manifest.transaction_id
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    pub fn mark_writing(&mut self) -> Result<(), String> {
        self.manifest.status = TransactionStatus::Writing;
        self.persist_manifest()
    }

    pub fn commit(&mut self, completed_at: &str) -> Result<(), String> {
        self.manifest.status = TransactionStatus::Committed;
        self.manifest.completed_at = Some(completed_at.to_string());
        self.manifest.failure = None;
        self.persist_manifest()?;
        write_last_summary(&self.install_root, &self.manifest_path, &self.manifest)?;
        remove_active_transaction(&self.install_root)
    }

    pub fn rollback(&mut self, completed_at: &str, failure: &str) -> Result<(), String> {
        match restore_manifest_files(&self.manifest_path, &self.manifest, &self.managed_files) {
            Ok(()) => {
                self.manifest.status = TransactionStatus::RolledBack;
                self.manifest.completed_at = Some(completed_at.to_string());
                self.manifest.failure = Some(failure.to_string());
                self.persist_manifest()?;
                write_last_summary(&self.install_root, &self.manifest_path, &self.manifest)?;
                remove_active_transaction(&self.install_root)
            }
            Err(rollback_error) => {
                self.manifest.status = TransactionStatus::RollbackFailed;
                self.manifest.completed_at = Some(completed_at.to_string());
                self.manifest.failure = Some(format!("{failure}; rollback: {rollback_error}"));
                let _ = self.persist_manifest();
                let _ = write_last_summary(&self.install_root, &self.manifest_path, &self.manifest);
                let _ = write_json(
                    &active_transaction_path(&self.install_root),
                    &ActiveTransaction {
                        schema_version: TRANSACTION_SCHEMA_VERSION,
                        transaction_id: self.manifest.transaction_id.clone(),
                        manifest_path: self.manifest_path.to_string_lossy().to_string(),
                    },
                );
                Err(format!(
                    "自动恢复失败: {rollback_error}; 事务清单: {}",
                    self.manifest_path.display()
                ))
            }
        }
    }

    fn persist_manifest(&self) -> Result<(), String> {
        write_json(&self.manifest_path, &self.manifest)
    }
}

pub fn recover_interrupted(
    install_root: &Path,
    managed_files: &[ManagedFile],
    completed_at: &str,
) -> Result<Option<TransactionSummary>, String> {
    let active_path = active_transaction_path(install_root);
    if !active_path.is_file() {
        return Ok(None);
    }
    let active: ActiveTransaction = read_json(&active_path)?;
    if active.schema_version != TRANSACTION_SCHEMA_VERSION {
        return Err(format!("不支持的活动事务版本: {}", active.schema_version));
    }
    let manifest_path = PathBuf::from(&active.manifest_path);
    let allowed_root = install_root.join(SNAPSHOT_DIRECTORY);
    if !manifest_path.starts_with(&allowed_root) {
        return Err(format!(
            "活动事务清单不在受管目录: {}",
            manifest_path.display()
        ));
    }
    let manifest: TransactionManifest = read_json(&manifest_path)?;
    if manifest.transaction_id != active.transaction_id {
        return Err("活动事务 ID 与清单不一致".to_string());
    }
    if matches!(
        manifest.status,
        TransactionStatus::Committed | TransactionStatus::RolledBack
    ) {
        write_last_summary(install_root, &manifest_path, &manifest)?;
        remove_active_transaction(install_root)?;
        return Ok(Some(summary_from_manifest(&manifest_path, &manifest)));
    }

    let mut transaction = ConfigTransaction {
        install_root: install_root.to_path_buf(),
        manifest_path,
        manifest,
        managed_files: managed_files.to_vec(),
    };
    transaction.rollback(completed_at, "检测到上次未完成的配置事务")?;
    Ok(Some(summary_from_manifest(
        &transaction.manifest_path,
        &transaction.manifest,
    )))
}

pub fn latest_committed_snapshot(
    install_root: &Path,
    managed_files: &[ManagedFile],
) -> Result<Option<(PathBuf, TransactionManifest)>, String> {
    let snapshots = install_root.join(SNAPSHOT_DIRECTORY);
    if !snapshots.is_dir() {
        return Ok(None);
    }
    let mut candidates = Vec::new();
    for entry in fs::read_dir(&snapshots).map_err(|error| format!("读取事务快照失败: {error}"))?
    {
        let Ok(entry) = entry else {
            continue;
        };
        let manifest_path = entry.path().join("manifest.json");
        if !manifest_path.is_file() {
            continue;
        }
        let Ok(manifest) = read_json::<TransactionManifest>(&manifest_path) else {
            continue;
        };
        if manifest.status != TransactionStatus::Committed
            || validate_manifest_files(&manifest, managed_files).is_err()
        {
            continue;
        }
        candidates.push((
            manifest.created_order.clone(),
            manifest.created_at.clone(),
            manifest_path,
            manifest,
        ));
    }
    candidates.sort_by(|left, right| {
        left.0
            .len()
            .cmp(&right.0.len())
            .then_with(|| left.0.cmp(&right.0))
            .then_with(|| left.1.cmp(&right.1))
    });
    Ok(candidates
        .pop()
        .map(|(_, _, manifest_path, manifest)| (manifest_path, manifest)))
}

pub fn restore_snapshot(
    manifest_path: &Path,
    manifest: &TransactionManifest,
    managed_files: &[ManagedFile],
) -> Result<(), String> {
    restore_manifest_files(manifest_path, manifest, managed_files)
}

pub fn last_transaction(install_root: &Path) -> Result<Option<TransactionSummary>, String> {
    let path = install_root.join(LAST_TRANSACTION_FILE);
    if !path.is_file() {
        return Ok(None);
    }
    read_json(&path).map(Some)
}

pub fn active_transaction_failed(install_root: &Path) -> bool {
    let active_path = active_transaction_path(install_root);
    if !active_path.exists() {
        return false;
    }
    let Ok(active) = read_json::<ActiveTransaction>(&active_path) else {
        return true;
    };
    let Ok(manifest) = read_json::<TransactionManifest>(Path::new(&active.manifest_path)) else {
        return true;
    };
    manifest.status == TransactionStatus::RollbackFailed
}

pub fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("无法确定写入目录: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("创建写入目录 {} 失败: {error}", parent.display()))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("写入文件名无效: {}", path.display()))?;
    let temporary = parent.join(format!(".{file_name}.{}.tmp", Uuid::new_v4().simple()));
    let result = (|| {
        let mut file = File::create(&temporary)
            .map_err(|error| format!("创建临时文件 {} 失败: {error}", temporary.display()))?;
        file.write_all(data)
            .map_err(|error| format!("写入临时文件 {} 失败: {error}", temporary.display()))?;
        file.sync_all()
            .map_err(|error| format!("同步临时文件 {} 失败: {error}", temporary.display()))?;
        replace_file(&temporary, path)?;
        sync_parent(parent)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn restore_manifest_files(
    manifest_path: &Path,
    manifest: &TransactionManifest,
    managed_files: &[ManagedFile],
) -> Result<(), String> {
    validate_manifest_files(manifest, managed_files)?;
    let snapshot_root = manifest_path
        .parent()
        .ok_or_else(|| format!("事务清单目录无效: {}", manifest_path.display()))?;
    for file in &manifest.files {
        let target = PathBuf::from(&file.target_path);
        if file.existed {
            let backup_name = file
                .backup_file
                .as_deref()
                .ok_or_else(|| format!("事务快照缺少 {} 的备份文件名", file.id))?;
            if Path::new(backup_name).components().count() != 1 {
                return Err(format!("事务快照备份路径无效: {backup_name}"));
            }
            let backup = snapshot_root.join(backup_name);
            let data = fs::read(&backup)
                .map_err(|error| format!("读取 {} 的事务快照失败: {error}", file.id))?;
            let actual = sha256_hex(&data);
            if file.sha256.as_deref() != Some(actual.as_str()) {
                return Err(format!("{} 的事务快照校验和不一致", file.id));
            }
            atomic_write(&target, &data)?;
        } else if target.exists() {
            if !target.is_file() {
                return Err(format!("无法清理非文件路径: {}", target.display()));
            }
            fs::remove_file(&target)
                .map_err(|error| format!("清理 {} 失败: {error}", target.display()))?;
            if let Some(parent) = target.parent() {
                sync_parent(parent)?;
            }
        }
    }
    Ok(())
}

fn validate_managed_files(files: &[ManagedFile]) -> Result<(), String> {
    if files.is_empty() {
        return Err("配置事务没有受管文件".to_string());
    }
    let mut ids = HashMap::new();
    for file in files {
        if file.id.is_empty()
            || !file
                .id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(format!("受管文件 ID 无效: {}", file.id));
        }
        if ids.insert(file.id.as_str(), &file.path).is_some() {
            return Err(format!("受管文件 ID 重复: {}", file.id));
        }
    }
    Ok(())
}

fn validate_manifest_files(
    manifest: &TransactionManifest,
    managed_files: &[ManagedFile],
) -> Result<(), String> {
    validate_managed_files(managed_files)?;
    if manifest.schema_version != TRANSACTION_SCHEMA_VERSION {
        return Err(format!("不支持的事务清单版本: {}", manifest.schema_version));
    }
    if manifest.files.len() != managed_files.len() {
        return Err("事务清单受管文件数量不一致".to_string());
    }
    for managed in managed_files {
        let Some(snapshot) = manifest.files.iter().find(|file| file.id == managed.id) else {
            return Err(format!("事务清单缺少受管文件: {}", managed.id));
        };
        if Path::new(&snapshot.target_path) != managed.path {
            return Err(format!("事务清单目标路径不一致: {}", managed.id));
        }
    }
    Ok(())
}

fn validate_transaction_id(transaction_id: &str) -> Result<(), String> {
    if transaction_id.is_empty()
        || !transaction_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err("配置事务 ID 无效".to_string());
    }
    Ok(())
}

fn snapshot_root(install_root: &Path, transaction_id: &str) -> PathBuf {
    install_root.join(SNAPSHOT_DIRECTORY).join(transaction_id)
}

fn active_transaction_path(install_root: &Path) -> PathBuf {
    install_root.join(ACTIVE_TRANSACTION_FILE)
}

fn remove_active_transaction(install_root: &Path) -> Result<(), String> {
    let path = active_transaction_path(install_root);
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|error| format!("清理活动事务记录 {} 失败: {error}", path.display()))?;
        sync_parent(install_root)?;
    }
    Ok(())
}

fn write_last_summary(
    install_root: &Path,
    manifest_path: &Path,
    manifest: &TransactionManifest,
) -> Result<(), String> {
    write_json(
        &install_root.join(LAST_TRANSACTION_FILE),
        &summary_from_manifest(manifest_path, manifest),
    )
}

fn summary_from_manifest(
    manifest_path: &Path,
    manifest: &TransactionManifest,
) -> TransactionSummary {
    TransactionSummary {
        schema_version: TRANSACTION_SCHEMA_VERSION,
        transaction_id: manifest.transaction_id.clone(),
        operation: manifest.operation.clone(),
        status: manifest.status.clone(),
        manifest_path: manifest_path.to_string_lossy().to_string(),
        completed_at: manifest.completed_at.clone(),
        failure: manifest.failure.clone(),
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let mut data =
        serde_json::to_vec_pretty(value).map_err(|error| format!("生成 JSON 失败: {error}"))?;
    serde_json::from_slice::<serde_json::Value>(&data)
        .map_err(|error| format!("校验 JSON 失败: {error}"))?;
    data.push(b'\n');
    atomic_write(path, &data)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let data = fs::read(path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
    serde_json::from_slice(&data).map_err(|error| format!("解析 {} 失败: {error}", path.display()))
}

fn sha256_hex(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

#[cfg(not(target_os = "windows"))]
fn replace_file(source: &Path, destination: &Path) -> Result<(), String> {
    fs::rename(source, destination)
        .map_err(|error| format!("原子替换 {} 失败: {error}", destination.to_string_lossy()))
}

#[cfg(target_os = "windows")]
fn replace_file(source: &Path, destination: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let source = source
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let replaced = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if replaced == 0 {
        return Err(format!(
            "原子替换文件失败: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn sync_parent(parent: &Path) -> Result<(), String> {
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("同步目录 {} 失败: {error}", parent.display()))
}

#[cfg(not(unix))]
fn sync_parent(_parent: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "codex-assistant-transaction-{name}-{}",
            Uuid::new_v4().simple()
        ))
    }

    fn managed_files(root: &Path) -> Vec<ManagedFile> {
        vec![
            ManagedFile::new("config", root.join("user").join("config.toml")),
            ManagedFile::new("state", root.join("runtime").join("config.json")),
        ]
    }

    #[test]
    fn committed_snapshot_has_hashes_and_is_discoverable() {
        let root = test_root("commit");
        let runtime = root.join("runtime");
        let files = managed_files(&root);
        atomic_write(&files[0].path, b"model = 'old'\n").unwrap();
        let mut transaction = ConfigTransaction::begin(
            &runtime,
            "tx-commit",
            "configure",
            "2026-07-28T10:00:00Z",
            "0.8.8",
            &files,
        )
        .unwrap();
        transaction.mark_writing().unwrap();
        atomic_write(&files[0].path, b"model = 'new'\n").unwrap();
        atomic_write(&files[1].path, b"{\"ready\":true}\n").unwrap();
        transaction.commit("2026-07-28T10:00:01Z").unwrap();

        let (_, manifest) = latest_committed_snapshot(&runtime, &files)
            .unwrap()
            .expect("latest transaction");
        assert_eq!(manifest.transaction_id, "tx-commit");
        assert_eq!(manifest.status, TransactionStatus::Committed);
        assert!(manifest.files[0].sha256.is_some());
        assert!(!active_transaction_path(&runtime).exists());
        assert_eq!(
            last_transaction(&runtime)
                .unwrap()
                .expect("last transaction")
                .transaction_id,
            "tx-commit"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rollback_restores_existing_files_and_removes_new_files() {
        let root = test_root("rollback");
        let runtime = root.join("runtime");
        let files = managed_files(&root);
        atomic_write(&files[0].path, b"model = 'old'\n").unwrap();
        let mut transaction = ConfigTransaction::begin(
            &runtime,
            "tx-rollback",
            "configure",
            "2026-07-28T10:00:00Z",
            "0.8.8",
            &files,
        )
        .unwrap();
        transaction.mark_writing().unwrap();
        atomic_write(&files[0].path, b"model = 'new'\n").unwrap();
        atomic_write(&files[1].path, b"{\"ready\":true}\n").unwrap();
        transaction
            .rollback("2026-07-28T10:00:01Z", "injected verify failure")
            .unwrap();

        assert_eq!(fs::read(&files[0].path).unwrap(), b"model = 'old'\n");
        assert!(!files[1].path.exists());
        assert_eq!(
            last_transaction(&runtime)
                .unwrap()
                .expect("last transaction")
                .status,
            TransactionStatus::RolledBack
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn interrupted_transaction_recovers_on_next_status_check() {
        let root = test_root("recovery");
        let runtime = root.join("runtime");
        let files = managed_files(&root);
        atomic_write(&files[0].path, b"model = 'old'\n").unwrap();
        let mut transaction = ConfigTransaction::begin(
            &runtime,
            "tx-interrupted",
            "configure",
            "2026-07-28T10:00:00Z",
            "0.8.8",
            &files,
        )
        .unwrap();
        transaction.mark_writing().unwrap();
        atomic_write(&files[0].path, b"model = 'partial'\n").unwrap();
        drop(transaction);

        let recovered = recover_interrupted(&runtime, &files, "2026-07-28T10:00:02Z")
            .unwrap()
            .expect("recovery result");
        assert_eq!(recovered.status, TransactionStatus::RolledBack);
        assert_eq!(fs::read(&files[0].path).unwrap(), b"model = 'old'\n");
        assert!(!active_transaction_path(&runtime).exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn interrupted_commit_finalizes_summary_without_rolling_back() {
        let root = test_root("commit-recovery");
        let runtime = root.join("runtime");
        let files = managed_files(&root);
        atomic_write(&files[0].path, b"model = 'old'\n").unwrap();
        let mut transaction = ConfigTransaction::begin(
            &runtime,
            "tx-commit-recovery",
            "configure",
            "2026-07-28T10:00:00Z",
            "0.8.8",
            &files,
        )
        .unwrap();
        transaction.mark_writing().unwrap();
        atomic_write(&files[0].path, b"model = 'new'\n").unwrap();
        transaction.manifest.status = TransactionStatus::Committed;
        transaction.manifest.completed_at = Some("2026-07-28T10:00:01Z".to_string());
        transaction.persist_manifest().unwrap();
        drop(transaction);

        let recovered = recover_interrupted(&runtime, &files, "2026-07-28T10:00:02Z")
            .unwrap()
            .expect("commit recovery result");
        assert_eq!(recovered.status, TransactionStatus::Committed);
        assert_eq!(fs::read(&files[0].path).unwrap(), b"model = 'new'\n");
        assert_eq!(
            last_transaction(&runtime)
                .unwrap()
                .expect("commit recovery summary")
                .transaction_id,
            "tx-commit-recovery"
        );
        assert!(!active_transaction_path(&runtime).exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn corrupted_snapshot_keeps_rollback_failure_evidence() {
        let root = test_root("corrupt");
        let runtime = root.join("runtime");
        let files = managed_files(&root);
        atomic_write(&files[0].path, b"model = 'old'\n").unwrap();
        let mut transaction = ConfigTransaction::begin(
            &runtime,
            "tx-corrupt",
            "configure",
            "2026-07-28T10:00:00Z",
            "0.8.8",
            &files,
        )
        .unwrap();
        transaction.mark_writing().unwrap();
        atomic_write(&files[0].path, b"model = 'partial'\n").unwrap();
        let backup = transaction
            .manifest_path()
            .parent()
            .unwrap()
            .join("config.bak");
        atomic_write(&backup, b"corrupted").unwrap();

        let error = transaction
            .rollback("2026-07-28T10:00:02Z", "injected failure")
            .expect_err("rollback must fail");
        assert!(error.contains("校验和不一致"));
        assert!(active_transaction_failed(&runtime));
        let manifest: TransactionManifest = read_json(transaction.manifest_path()).unwrap();
        assert_eq!(manifest.status, TransactionStatus::RollbackFailed);
        assert_eq!(
            last_transaction(&runtime)
                .unwrap()
                .expect("failed transaction summary")
                .transaction_id,
            "tx-corrupt"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
