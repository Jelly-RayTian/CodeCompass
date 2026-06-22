use tauri::State;

use crate::analysis::call_graph::{build_call_graph, CallGraph};
use crate::analysis::impact::{compute_impact, ChangeRisk};
use crate::db::Database;
use crate::error::AppError;

#[tauri::command]
pub fn get_call_graph(
    db: State<'_, Database>,
    workspace_id: i64,
    focus_symbol_id: Option<i64>,
    max_depth: Option<i64>,
) -> Result<CallGraph, AppError> {
    build_call_graph(&db, workspace_id, focus_symbol_id, max_depth.unwrap_or(3))
}

#[tauri::command]
pub fn get_change_impact(
    db: State<'_, Database>,
    workspace_id: i64,
    symbol_id: i64,
) -> Result<ChangeRisk, AppError> {
    compute_impact(&db, workspace_id, symbol_id)
}
