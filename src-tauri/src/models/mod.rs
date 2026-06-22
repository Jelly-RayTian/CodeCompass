use serde::{Deserialize, Serialize};

/// Information about the running application, returned by
/// `get_application_info`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationInfo {
    pub name: String,
    pub version: String,
    #[serde(rename = "buildTimestamp")]
    pub build_timestamp: String,
}

/// Status of the local SQLite database, returned by `get_database_status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStatus {
    pub connected: bool,
    #[serde(rename = "databasePath")]
    pub database_path: String,
    #[serde(rename = "migrationVersion")]
    pub migration_version: i64,
}

/// An indexed folder registered by the user.
///
/// This model deliberately keeps the `workspace` terminology in the database
/// layer (where the `workspaces` table already exists) while exposing the
/// concept as an "indexed folder" to the rest of the application.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexedFolder {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub added_at: i64,
    pub last_successful_scan_at: Option<i64>,
    pub availability: String,
    pub monitoring_enabled: bool,
    pub scan_status: String,
}

/// Result of adding a new indexed folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFolderResult {
    pub folder: IndexedFolder,
    pub warning: Option<String>,
}

/// A single metadata scan pass over an indexed folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRun {
    pub id: i64,
    pub workspace_id: i64,
    pub status: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub files_processed: i64,
    pub files_indexed: i64,
    pub warning_count: i64,
    pub error_count: i64,
    pub error_message: Option<String>,
    pub phase: Option<String>,
}

/// Snapshot returned to the frontend while a scan is in progress or after it
/// finishes. It combines the latest scan run with the current file count for
/// the folder, so the UI never has to load the full file list into memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatus {
    pub run: ScanRun,
    pub file_count: i64,
}

/// A single indexed file entry returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub id: i64,
    pub workspace_id: i64,
    pub relative_path: String,
    pub name: String,
    pub parent_path: String,
    pub extension: Option<String>,
    pub size_bytes: i64,
    pub created_at: Option<i64>,
    pub modified_at: Option<i64>,
    pub indexed_at: Option<i64>,
    pub last_seen_at: Option<i64>,
    pub fingerprint: Option<String>,
    pub previous_fingerprint: Option<String>,
    pub is_present: bool,
    pub change_status: String,
}

/// Emitted by the scanner while a run is in progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgressEvent {
    pub run_id: i64,
    pub workspace_id: i64,
    pub status: String,
    pub files_processed: i64,
    pub files_indexed: i64,
    pub warning_count: i64,
    pub error_count: i64,
    pub phase: Option<String>,
}
