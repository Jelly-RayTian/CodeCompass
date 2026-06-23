use thiserror::Error;

/// The unified error type for all CodeCompass backend operations.
///
/// Every recoverable error flows through this enum.  It implements
/// `serde::Serialize` so Tauri can send the message to the frontend
/// when a command fails.
///
/// Error messages are written for end users: they name what failed, the
/// likely cause, and the next action the user can take.  Raw
/// `rusqlite`/`io` messages are kept for the log but the user-facing
/// string explains recovery.
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] refinery::Error),

    #[error("Database lock error: {0}")]
    DatabaseLock(String),

    #[error("Application directory error: {0}")]
    AppDir(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Folder not found: {0}")]
    FolderNotFound(String),

    #[error("Path is not a directory: {0}")]
    NotADirectory(String),

    #[error("Folder already indexed: {0}")]
    DuplicateFolder(String),

    #[error("Scan cancelled")]
    ScanCancelled,

    #[error("Scan already running for folder {0}")]
    ScanAlreadyRunning(i64),

    #[error("Scan run not found: {0}")]
    ScanRunNotFound(i64),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Path traversal detected: {0}")]
    PathTraversal(String),

    #[error("Analysis already running for workspace {0}")]
    AnalysisAlreadyRunning(i64),

    #[error("File too large to display: {size} bytes (limit {limit} bytes) at {path}")]
    OversizedFile { path: String, size: u64, limit: u64 },
}

impl AppError {
    /// Returns a short, human-readable code identifying the error category.
    /// The frontend can switch on this to render tailored recovery UI.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Database(_) => "database_error",
            Self::Migration(_) => "migration_error",
            Self::DatabaseLock(_) => "database_lock",
            Self::AppDir(_) => "app_dir",
            Self::Io(_) => "io_error",
            Self::FolderNotFound(_) => "folder_not_found",
            Self::NotADirectory(_) => "not_a_directory",
            Self::DuplicateFolder(_) => "duplicate_folder",
            Self::ScanCancelled => "scan_cancelled",
            Self::ScanAlreadyRunning(_) => "scan_already_running",
            Self::ScanRunNotFound(_) => "scan_run_not_found",
            Self::InvalidInput(_) => "invalid_input",
            Self::PathTraversal(_) => "path_traversal",
            Self::AnalysisAlreadyRunning(_) => "analysis_already_running",
            Self::OversizedFile { .. } => "oversized_file",
        }
    }

    /// A user-facing, actionable message distinct from the technical
    /// `Display` string. Explains what failed, likely cause, and the next
    /// step the user can try.
    pub fn user_message(&self) -> String {
        match self {
            Self::FolderNotFound(p) => format!(
                "The folder could not be found: {p}.\n\n\
                 Likely cause: the folder was moved, renamed, or the drive is \
                 not mounted.\n\n\
                 Your saved index data is safe. Re-add the folder at its new \
                 location, or remove the missing workspace from the list."
            ),
            Self::NotADirectory(p) => format!(
                "The selected path is a file, not a folder: {p}.\n\n\
                 Choose a directory that contains your source code."
            ),
            Self::DuplicateFolder(p) => format!(
                "This folder is already in your workspace list: {p}.\n\n\
                 Select the existing entry to scan or analyze it again."
            ),
            Self::ScanAlreadyRunning(id) => format!(
                "A scan is already running for workspace {id}.\n\n\
                 Wait for it to finish, or cancel it before starting a new one."
            ),
            Self::AnalysisAlreadyRunning(id) => format!(
                "Analysis is already running for workspace {id}.\n\n\
                 Wait for it to finish, or cancel it before restarting."
            ),
            Self::ScanCancelled => "The scan was cancelled. Files indexed before \
                 cancellation are preserved; deleted files are not reconciled \
                 until a full scan completes."
                .to_string(),
            Self::ScanRunNotFound(id) => format!(
                "Scan run {id} was not found. It may have been cleaned up. \
                 Start a new scan to refresh the index."
            ),
            Self::InvalidInput(msg) => {
                format!("Invalid input: {msg}.\n\nPlease check the value and try again.")
            }
            Self::PathTraversal(p) => format!(
                "A path was rejected because it escapes the workspace root: {p}.\n\n\
                 This is a safety check. Choose a path inside the workspace."
            ),
            Self::DatabaseLock(msg) => format!(
                "The index database is busy: {msg}.\n\n\
                 Another operation may be in progress. Wait a moment and retry. \
                 Your data is not lost."
            ),
            Self::AppDir(msg) => format!(
                "CodeCompass could not access its application data directory: {msg}.\n\n\
                 Check that the app data folder is writable and the disk is not full."
            ),
            Self::Database(e) => format!(
                "A database error occurred: {e}.\n\n\
                 Your source files are unaffected. Restart CodeCompass and retry. \
                 If the problem persists, removing the index database in the app \
                 data folder will rebuild it from scratch."
            ),
            Self::Migration(e) => format!(
                "The index database could not be upgraded: {e}.\n\n\
                 Your source files are unaffected. Removing the database file in \
                 the app data folder will recreate it on next launch."
            ),
            Self::Io(e) => format!(
                "A file system error occurred: {e}.\n\n\
                 Check that the folder exists and that you have permission to \
                 read it, then retry."
            ),
            Self::OversizedFile { path, size, limit } => format!(
                "The file {path} is too large to display ({size} bytes; limit {limit}).\n\n\
                 CodeCompass shows the first {limit} bytes for performance. \
                 Open the file in a dedicated editor to view the full contents."
            ),
        }
    }

    /// A serialisable payload combining the technical message, the
    /// user-facing guidance, and the stable code. The frontend can show
    /// `message` while logging `detail`.
    pub fn to_payload(&self) -> ErrorPayload {
        ErrorPayload {
            code: self.code(),
            message: self.user_message(),
            detail: self.to_string(),
        }
    }
}

/// Structured error payload sent to the frontend.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub code: &'static str,
    pub message: String,
    pub detail: String,
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable_and_unique() {
        let cases = [
            (AppError::FolderNotFound("x".into()), "folder_not_found"),
            (AppError::NotADirectory("x".into()), "not_a_directory"),
            (AppError::DuplicateFolder("x".into()), "duplicate_folder"),
            (AppError::ScanCancelled, "scan_cancelled"),
            (AppError::ScanAlreadyRunning(1), "scan_already_running"),
            (AppError::InvalidInput("x".into()), "invalid_input"),
            (AppError::PathTraversal("x".into()), "path_traversal"),
        ];
        let mut seen = std::collections::HashSet::new();
        for (err, expected) in cases {
            assert_eq!(err.code(), expected);
            assert!(seen.insert(expected), "duplicate code {expected}");
        }
    }

    #[test]
    fn user_message_explains_recovery() {
        let err = AppError::FolderNotFound("/missing/path".into());
        let msg = err.user_message();
        assert!(msg.contains("could not be found"), "{msg}");
        assert!(msg.contains("safe"), "should reassure data safety: {msg}");
    }

    #[test]
    fn payload_round_trips_codes_and_messages() {
        let err = AppError::ScanCancelled;
        let p = err.to_payload();
        assert_eq!(p.code, "scan_cancelled");
        assert!(!p.message.is_empty());
        assert_eq!(p.detail, err.to_string());
    }
}
