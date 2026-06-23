use tauri::State;

use crate::db::indexed_folders::get_folder_path;
use crate::db::Database;
use crate::error::AppError;
use crate::platform::path_is_inside_or_equal;

const MAX_FILE_SIZE: u64 = 1_000_000; // 1 MB

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFile {
    pub content: String,
    pub language: String,
    pub total_lines: i64,
    pub truncated: bool,
}

/// Core logic for reading a source file, separated from the Tauri
/// `State<Database>` wrapper so it can be unit/integration-tested with a
/// plain `&Database`.
pub fn read_source_file_struct(
    db: &Database,
    workspace_id: i64,
    relative_path: &str,
) -> Result<SourceFile, AppError> {
    let root = match get_folder_path(db, workspace_id)? {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            return Err(AppError::FolderNotFound(format!(
                "workspace {}",
                workspace_id
            )))
        }
    };

    let absolute = root.join(relative_path);

    // Security: ensure the resolved path stays inside the workspace root.
    if !path_is_inside_or_equal(&root, &absolute) {
        return Err(AppError::PathTraversal(
            absolute.to_string_lossy().to_string(),
        ));
    }

    if !absolute.is_file() {
        return Err(AppError::InvalidInput(format!(
            "file not found: {}",
            relative_path
        )));
    }

    let meta = std::fs::metadata(&absolute)?;
    let truncated = meta.len() > MAX_FILE_SIZE;

    let content = if truncated {
        let mut buf = vec![0u8; MAX_FILE_SIZE as usize];
        use std::io::Read;
        let mut f = std::fs::File::open(&absolute)?;
        let n = f.read(&mut buf)?;
        buf.truncate(n);
        String::from_utf8_lossy(&buf).to_string()
    } else {
        std::fs::read_to_string(&absolute)?
    };

    let language = detect_language(relative_path);
    let total_lines = content.lines().count() as i64;

    Ok(SourceFile {
        content,
        language,
        total_lines,
        truncated,
    })
}

#[tauri::command]
pub fn read_source_file(
    db: State<'_, Database>,
    workspace_id: i64,
    relative_path: String,
) -> Result<SourceFile, AppError> {
    read_source_file_struct(&db, workspace_id, &relative_path)
}

fn detect_language(path: &str) -> String {
    let lower = path.to_lowercase();
    if lower.ends_with(".tsx") {
        "typescriptreact".to_string()
    } else if lower.ends_with(".ts") {
        "typescript".to_string()
    } else if lower.ends_with(".jsx") {
        "javascriptreact".to_string()
    } else if lower.ends_with(".js") {
        "javascript".to_string()
    } else {
        "plaintext".to_string()
    }
}
