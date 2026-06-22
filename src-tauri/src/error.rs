use thiserror::Error;

/// The unified error type for all CodeCompass backend operations.
///
/// Every recoverable error flows through this enum.  It implements
/// `serde::Serialize` so Tauri can send the message to the frontend
/// when a command fails.
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
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
