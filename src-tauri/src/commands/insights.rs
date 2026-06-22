use tauri::State;

use crate::analysis::entrypoint::{detect_entry_points, EntryPoint};
use crate::analysis::findings::{collect_findings, StructuralFinding};
use crate::analysis::reading_path::{generate_reading_path, ReadingPathItem};
use crate::db::Database;
use crate::error::AppError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInsights {
    pub entry_points: Vec<EntryPoint>,
    pub reading_path: Vec<ReadingPathItem>,
    pub findings: Vec<StructuralFinding>,
}

#[tauri::command]
pub fn get_workspace_insights(
    db: State<'_, Database>,
    workspace_id: i64,
) -> Result<WorkspaceInsights, AppError> {
    let entry_points = detect_entry_points(&db, workspace_id)?;
    let entry_ids: Vec<i64> = entry_points
        .iter()
        .filter(|e| e.confidence >= 0.4)
        .map(|e| e.file_id)
        .collect();
    let reading_path = generate_reading_path(&db, workspace_id, &entry_ids)?;
    let findings = collect_findings(&db, workspace_id)?;

    Ok(WorkspaceInsights {
        entry_points,
        reading_path,
        findings,
    })
}
