use crate::db::Database;
use crate::error::AppError;

/// Per-file health metrics aggregated from multiple data sources.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHealth {
    pub file_id: i64,
    pub relative_path: String,
    pub name: String,
    pub size_bytes: i64,
    pub line_count: i64,
    pub import_out_degree: i64,
    pub import_in_degree: i64,
    pub symbol_count: i64,
    pub diagnostic_count: i64,
    pub change_count: i64,
    pub is_in_cycle: bool,
    pub risk_score: f64,
    pub risk_category: String,
}

/// Aggregated workspace-level statistics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthSummary {
    pub total_files: i64,
    pub files_analyzed: i64,
    pub total_imports: i64,
    pub total_symbols: i64,
    pub cycle_count: i64,
    pub avg_risk_score: f64,
    pub files_low_risk: i64,
    pub files_medium_risk: i64,
    pub files_high_risk: i64,
    pub files_critical_risk: i64,
}

/// Complete repository health report for a workspace.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryHealth {
    pub summary: HealthSummary,
    pub top_risk_files: Vec<FileHealth>,
    pub all_files: Vec<FileHealth>,
}

/// Clamps a value between 0.0 and 1.0.
fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

/// Returns a category label based on the numeric risk score (0–100).
fn risk_category(score: f64) -> String {
    if score >= 75.0 {
        "critical"
    } else if score >= 50.0 {
        "high"
    } else if score >= 25.0 {
        "medium"
    } else {
        "low"
    }
    .to_string()
}

/// Computes a composite risk score (0–100) for a file based on weighted
/// sub-scores. Heavier weight on size and complexity.
///
/// Limitations:
///  - Does not measure actual cyclomatic complexity; uses line count as proxy.
///  - Change count is only available when Git analysis is enabled.
///  - Static import counts may over/under-represent true coupling.
fn compute_risk_score(
    size_bytes: i64,
    line_count: i64,
    import_degree: i64,
    change_count: i64,
    diagnostic_count: i64,
) -> f64 {
    let size_score = clamp01(size_bytes as f64 / 100_000.0);
    let complexity_score = clamp01(line_count as f64 / 500.0);
    let dependency_score = clamp01(import_degree as f64 / 30.0);
    let churn_score = clamp01(change_count as f64 / 20.0);
    let diagnostic_score = clamp01(diagnostic_count as f64 / 10.0);

    let composite = 0.25 * size_score
        + 0.25 * complexity_score
        + 0.20 * dependency_score
        + 0.15 * churn_score
        + 0.15 * diagnostic_score;

    (composite * 100.0).clamp(0.0, 100.0)
}

/// Collects all files that participate in cycles (source or target of any
/// edge that belongs to a cycle detected by DFS).
fn collect_cycle_file_ids(
    conn: &rusqlite::Connection,
    workspace_id: i64,
) -> Result<Vec<i64>, AppError> {
    // Gather all file IDs that have internal edges. We'll run DFS on them.
    let mut stmt = conn.prepare(
        "SELECT DISTINCT i.source_file_id, i.resolved_target_file_id \
         FROM imports i \
         JOIN indexed_files f ON i.source_file_id = f.id \
         WHERE f.workspace_id = ?1 AND i.is_external = 0 \
         AND i.resolved_target_file_id IS NOT NULL",
    )?;
    let edges: Vec<(i64, i64)> = stmt
        .query_map(rusqlite::params![workspace_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut adj: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
    for (src, tgt) in &edges {
        adj.entry(*src).or_default().push(*tgt);
    }

    let mut cycle_ids: Vec<i64> = Vec::new();
    let mut visited: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut on_stack: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut stack: Vec<i64> = Vec::new();

    let all_ids: std::collections::HashSet<i64> =
        edges.iter().flat_map(|(a, b)| [*a, *b]).collect();

    for &node in &all_ids {
        if !visited.contains(&node) {
            dfs_and_collect(
                node,
                &adj,
                &mut visited,
                &mut on_stack,
                &mut stack,
                &mut cycle_ids,
            );
        }
    }

    Ok(cycle_ids)
}

fn dfs_and_collect(
    u: i64,
    adj: &std::collections::HashMap<i64, Vec<i64>>,
    visited: &mut std::collections::HashSet<i64>,
    on_stack: &mut std::collections::HashSet<i64>,
    stack: &mut Vec<i64>,
    cycle_ids: &mut Vec<i64>,
) {
    visited.insert(u);
    stack.push(u);
    on_stack.insert(u);

    if let Some(neighbors) = adj.get(&u) {
        for &v in neighbors {
            if !visited.contains(&v) {
                dfs_and_collect(v, adj, visited, on_stack, stack, cycle_ids);
            } else if on_stack.contains(&v) {
                let pos = stack.iter().position(|&x| x == v);
                if let Some(idx) = pos {
                    for &id in &stack[idx..] {
                        if !cycle_ids.contains(&id) {
                            cycle_ids.push(id);
                        }
                    }
                }
            }
        }
    }

    stack.pop();
    on_stack.remove(&u);
}

type FileRow = (i64, String, String, i64, i64, i64, i64, i64, i64, i64);

/// Builds the full repository health report.
pub fn build_health_report(db: &Database, workspace_id: i64) -> Result<RepositoryHealth, AppError> {
    let conn = db.lock()?;

    // Count total present files.
    let total_files: i64 = conn.query_row(
        "SELECT COUNT(*) FROM indexed_files WHERE workspace_id = ?1 AND is_present = 1",
        rusqlite::params![workspace_id],
        |row| row.get(0),
    )?;

    // Count analyzed files.
    let files_analyzed: i64 = conn.query_row(
        "SELECT COUNT(*) FROM indexed_files \
         WHERE workspace_id = ?1 AND is_present = 1 AND analysis_status = 'analyzed'",
        rusqlite::params![workspace_id],
        |row| row.get(0),
    )?;

    // Count resolved internal imports.
    let total_imports: i64 = conn.query_row(
        "SELECT COUNT(*) FROM imports i \
         JOIN indexed_files f ON i.source_file_id = f.id \
         WHERE f.workspace_id = ?1 AND i.is_external = 0 \
         AND i.resolved_target_file_id IS NOT NULL",
        rusqlite::params![workspace_id],
        |row| row.get(0),
    )?;

    // Count total symbols.
    let total_symbols: i64 = conn.query_row(
        "SELECT COUNT(*) FROM symbols WHERE workspace_id = ?1",
        rusqlite::params![workspace_id],
        |row| row.get(0),
    )?;

    // Cycle file IDs.
    let cycle_ids = collect_cycle_file_ids(&conn, workspace_id)?;
    let unique_cycle_files: std::collections::HashSet<i64> = cycle_ids.iter().copied().collect();

    // Per-file metrics: join indexed_files with imports (degree), symbols,
    // diagnostics, and git changes.
    let mut stmt = conn.prepare(
        "SELECT \
         f.id, f.relative_path, f.name, \
         COALESCE(f.size_bytes, 0), \
         COALESCE(f.line_count, 0), \
         COALESCE(out_d.cnt, 0), \
         COALESCE(in_d.cnt, 0), \
         COALESCE(sym.cnt, 0), \
         COALESCE(diag.cnt, 0), \
         COALESCE(chg.cnt, 0) \
         FROM indexed_files f \
         LEFT JOIN (SELECT source_file_id, COUNT(*) AS cnt FROM imports GROUP BY source_file_id) out_d \
           ON f.id = out_d.source_file_id \
         LEFT JOIN (SELECT resolved_target_file_id, COUNT(*) AS cnt FROM imports WHERE resolved_target_file_id IS NOT NULL GROUP BY resolved_target_file_id) in_d \
           ON f.id = in_d.resolved_target_file_id \
         LEFT JOIN (SELECT file_id, COUNT(*) AS cnt FROM symbols GROUP BY file_id) sym \
           ON f.id = sym.file_id \
         LEFT JOIN (SELECT file_id, COUNT(*) AS cnt FROM analysis_diagnostics GROUP BY file_id) diag \
           ON f.id = diag.file_id \
         LEFT JOIN (SELECT relative_path, COUNT(*) AS cnt FROM git_file_changes GROUP BY relative_path) chg \
           ON f.relative_path = chg.relative_path \
         WHERE f.workspace_id = ?1 AND f.is_present = 1",
    )?;

    let file_rows: Vec<FileRow> = stmt
        .query_map(rusqlite::params![workspace_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut all_files: Vec<FileHealth> = Vec::with_capacity(file_rows.len());
    for (
        file_id,
        relative_path,
        name,
        size_bytes,
        line_count,
        out_d,
        in_d,
        symbol_count,
        diagnostic_count,
        change_count,
    ) in file_rows
    {
        let import_degree = out_d + in_d;
        let is_in_cycle = unique_cycle_files.contains(&file_id);
        // Boost risk slightly for files in cycles.
        let base_risk = compute_risk_score(
            size_bytes,
            line_count,
            import_degree,
            change_count,
            diagnostic_count,
        );
        let risk_score = if is_in_cycle {
            (base_risk * 1.15).min(100.0)
        } else {
            base_risk
        };
        let risk_category = risk_category(risk_score);

        all_files.push(FileHealth {
            file_id,
            relative_path,
            name,
            size_bytes,
            line_count,
            import_out_degree: out_d,
            import_in_degree: in_d,
            symbol_count,
            diagnostic_count,
            change_count,
            is_in_cycle,
            risk_score,
            risk_category,
        });
    }

    // Sort by risk descending.
    all_files.sort_by(|a, b| {
        b.risk_score
            .partial_cmp(&a.risk_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let top_risk_files: Vec<FileHealth> = all_files.iter().take(20).cloned().collect();

    let avg_risk_score = if all_files.is_empty() {
        0.0
    } else {
        all_files.iter().map(|f| f.risk_score).sum::<f64>() / all_files.len() as f64
    };

    let files_low_risk = all_files
        .iter()
        .filter(|f| f.risk_category == "low")
        .count() as i64;
    let files_medium_risk = all_files
        .iter()
        .filter(|f| f.risk_category == "medium")
        .count() as i64;
    let files_high_risk = all_files
        .iter()
        .filter(|f| f.risk_category == "high")
        .count() as i64;
    let files_critical_risk = all_files
        .iter()
        .filter(|f| f.risk_category == "critical")
        .count() as i64;

    let summary = HealthSummary {
        total_files,
        files_analyzed,
        total_imports,
        total_symbols,
        cycle_count: unique_cycle_files.len() as i64,
        avg_risk_score,
        files_low_risk,
        files_medium_risk,
        files_high_risk,
        files_critical_risk,
    };

    Ok(RepositoryHealth {
        summary,
        top_risk_files,
        all_files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::indexed_files::{upsert_files_batch, FileUpsert};
    use crate::db::indexed_folders::insert_indexed_folder;
    use crate::db::Database;
    use tempfile::tempdir;

    fn insert_file(db: &Database, ws_id: i64, rel: &str, line_count: i64, size_bytes: i64) -> i64 {
        let mut batch = vec![FileUpsert {
            relative_path: rel.to_string(),
            name: rel.split('/').next_back().unwrap_or(rel).to_string(),
            parent_path: ".".to_string(),
            extension: Some("ts".to_string()),
            size_bytes,
            created_at: Some(1),
            modified_at: Some(2),
            fingerprint: format!("fp:{}", rel),
            indexed_at: 1000,
            last_seen_at: 1000,
        }];
        upsert_files_batch(db, ws_id, 0, &mut batch).unwrap();
        let id = db
            .lock()
            .unwrap()
            .query_row(
                "SELECT id FROM indexed_files WHERE workspace_id = ?1 AND relative_path = ?2",
                rusqlite::params![ws_id, rel],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();
        // Set line count via raw update (simulates what runner does).
        db.lock()
            .unwrap()
            .execute(
                "UPDATE indexed_files SET line_count = ?1, analysis_status = 'analyzed' WHERE id = ?2",
                rusqlite::params![line_count, id],
            )
            .unwrap();
        id
    }

    fn insert_imports(db: &Database, source_id: i64, target_id: i64) {
        let conn = db.lock().unwrap();
        conn.execute(
            "INSERT INTO imports (source_file_id, target_specifier, resolved_target_file_id, import_type, is_external, created_at) \
             VALUES (?1, ?2, ?3, 'static', 0, 2000)",
            rusqlite::params![source_id, format!("./target{}", target_id), target_id],
        )
        .unwrap();
    }

    #[test]
    fn empty_health_report() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        let folder = dir.path().join("root");
        std::fs::create_dir(&folder).unwrap();
        let ws_id = insert_indexed_folder(&db, &folder).unwrap().id;

        let report = build_health_report(&db, ws_id).unwrap();
        assert_eq!(report.summary.total_files, 0);
        assert!(report.all_files.is_empty());
        assert!(report.top_risk_files.is_empty());
    }

    #[test]
    fn health_report_with_files() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        let folder = dir.path().join("root");
        std::fs::create_dir(&folder).unwrap();
        let ws_id = insert_indexed_folder(&db, &folder).unwrap().id;

        let _a = insert_file(&db, ws_id, "a.ts", 50, 2000);
        let _b = insert_file(&db, ws_id, "b.ts", 600, 120000); // large file
        let _c = insert_file(&db, ws_id, "c.ts", 30, 1500);

        // b is a high-risk file (600 lines).
        let report = build_health_report(&db, ws_id).unwrap();
        assert_eq!(report.summary.total_files, 3);
        assert_eq!(report.summary.files_analyzed, 3);

        // b.ts should be top risk.
        let top = &report.top_risk_files[0];
        assert_eq!(top.relative_path, "b.ts");
        assert!(
            top.risk_score >= 50.0,
            "expected >=50.0, got {}",
            top.risk_score
        );
    }

    #[test]
    fn cycle_files_flagged() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        let folder = dir.path().join("root");
        std::fs::create_dir(&folder).unwrap();
        let ws_id = insert_indexed_folder(&db, &folder).unwrap().id;

        let a = insert_file(&db, ws_id, "a.ts", 30, 2000);
        let b = insert_file(&db, ws_id, "b.ts", 30, 2000);
        insert_imports(&db, a, b);
        insert_imports(&db, b, a);

        let report = build_health_report(&db, ws_id).unwrap();
        assert!(report.summary.cycle_count >= 1);
        let a_health = report.all_files.iter().find(|f| f.file_id == a).unwrap();
        let b_health = report.all_files.iter().find(|f| f.file_id == b).unwrap();
        assert!(a_health.is_in_cycle);
        assert!(b_health.is_in_cycle);
    }

    #[test]
    fn risk_score_range() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        let folder = dir.path().join("root");
        std::fs::create_dir(&folder).unwrap();
        let ws_id = insert_indexed_folder(&db, &folder).unwrap().id;

        let _ = insert_file(&db, ws_id, "small.ts", 10, 500);
        let report = build_health_report(&db, ws_id).unwrap();
        let f = &report.all_files[0];
        assert!(f.risk_score >= 0.0 && f.risk_score <= 100.0);
        assert_eq!(f.risk_category, "low");
    }
}
