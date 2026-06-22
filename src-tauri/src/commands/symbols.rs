use tauri::State;

use crate::db::symbols::{file_outline, search_symbols, SymbolSearchResult};
use crate::db::Database;
use crate::error::AppError;

#[tauri::command]
pub fn search_symbols_command(
    db: State<'_, Database>,
    workspace_id: i64,
    query: Option<String>,
    kind: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<SymbolSearchResult, AppError> {
    search_symbols(
        &db,
        workspace_id,
        query.as_deref(),
        kind.as_deref(),
        page.unwrap_or(1),
        page_size.unwrap_or(20),
    )
}

#[tauri::command]
pub fn get_file_outline_command(
    db: State<'_, Database>,
    file_id: i64,
) -> Result<Vec<crate::db::symbols::SymbolEntry>, AppError> {
    file_outline(&db, file_id)
}
