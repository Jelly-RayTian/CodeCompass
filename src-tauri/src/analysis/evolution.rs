use crate::db::workspace_settings::CoChangePair;
use crate::db::Database;
use crate::error::AppError;

/// A single point on the commit timeline (monthly bucket).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelinePoint {
    pub month: String,
    pub commit_count: i64,
    pub file_changes: i64,
}

/// A file with its change frequency from git history.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChurn {
    pub relative_path: String,
    pub change_count: i64,
}

/// Aggregate evolution statistics for the workspace.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvolutionSummary {
    pub total_commits: i64,
    pub total_files_changed: i64,
    pub total_file_changes: i64,
    pub most_active_month: String,
    pub oldest_commit_ts: i64,
    pub newest_commit_ts: i64,
}

/// Complete repository evolution data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryEvolution {
    pub summary: EvolutionSummary,
    pub timeline: Vec<TimelinePoint>,
    pub top_churn_files: Vec<FileChurn>,
    pub top_hotspots: Vec<CoChangePair>,
}

fn epoch_to_month(epoch: i64) -> String {
    if epoch <= 0 {
        return "unknown".to_string();
    }
    // Simple approach: seconds since epoch → months since epoch
    let months_since = epoch / 2629800; // approx 30.44 days in seconds
    let year = 1970 + (months_since / 12);
    let month = (months_since % 12) + 1;
    format!("{:04}-{:02}", year, month)
}

/// Builds the repository evolution report.
pub fn build_evolution_report(
    db: &Database,
    workspace_id: i64,
) -> Result<RepositoryEvolution, AppError> {
    let (summary, timeline, top_churn_files) = {
        let conn = db.lock()?;

        // Count distinct commits (with real timestamps).
        let total_commits: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT commit_hash) FROM git_file_changes \
             WHERE workspace_id = ?1 AND timestamp > 0",
                rusqlite::params![workspace_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Count distinct files changed.
        let total_files_changed: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT relative_path) FROM git_file_changes \
             WHERE workspace_id = ?1",
                rusqlite::params![workspace_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Total file changes.
        let total_file_changes: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM git_file_changes WHERE workspace_id = ?1",
                rusqlite::params![workspace_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Oldest and newest commit timestamps.
        let oldest_ts: i64 = conn
            .query_row(
                "SELECT COALESCE(MIN(timestamp), 0) FROM git_file_changes \
             WHERE workspace_id = ?1 AND timestamp > 0",
                rusqlite::params![workspace_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let newest_ts: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(timestamp), 0) FROM git_file_changes \
             WHERE workspace_id = ?1",
                rusqlite::params![workspace_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let mut month_counts: std::collections::HashMap<String, i64> =
            std::collections::HashMap::new();
        if oldest_ts > 0 {
            let mut ts_stmt = conn.prepare(
                "SELECT timestamp FROM git_file_changes \
             WHERE workspace_id = ?1 AND timestamp > 0",
            )?;
            let ts_rows: Vec<i64> = ts_stmt
                .query_map(rusqlite::params![workspace_id], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .collect();
            for ts in ts_rows {
                let m = epoch_to_month(ts);
                *month_counts.entry(m).or_insert(0) += 1;
            }
        }

        let most_active_month = month_counts
            .into_iter()
            .max_by_key(|(_, c)| *c)
            .map(|(m, _)| m)
            .unwrap_or_else(|| "unknown".to_string());

        let summary = EvolutionSummary {
            total_commits,
            total_files_changed,
            total_file_changes,
            most_active_month,
            oldest_commit_ts: oldest_ts,
            newest_commit_ts: newest_ts,
        };

        // Build timeline: aggregate commits and file changes by month.
        let mut month_data: std::collections::BTreeMap<String, (i64, i64)> =
            std::collections::BTreeMap::new();

        let mut cmt_stmt = conn.prepare(
            "SELECT DISTINCT commit_hash, timestamp FROM git_file_changes \
         WHERE workspace_id = ?1 AND timestamp > 0",
        )?;
        let commit_rows: Vec<(String, i64)> = cmt_stmt
            .query_map(rusqlite::params![workspace_id], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();
        for (_hash, ts) in &commit_rows {
            let m = epoch_to_month(*ts);
            let entry = month_data.entry(m).or_insert((0, 0));
            entry.0 += 1;
        }

        // Count file changes per month from the raw table.
        let mut file_stmt = conn.prepare(
            "SELECT timestamp FROM git_file_changes WHERE workspace_id = ?1 AND timestamp > 0",
        )?;
        let file_ts_rows: Vec<i64> = file_stmt
            .query_map(rusqlite::params![workspace_id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        for ts in file_ts_rows {
            let m = epoch_to_month(ts);
            let entry = month_data.entry(m).or_insert((0, 0));
            entry.1 += 1;
        }

        let timeline: Vec<TimelinePoint> = month_data
            .into_iter()
            .map(|(month, (commits, changes))| TimelinePoint {
                month,
                commit_count: commits,
                file_changes: changes,
            })
            .collect();

        // Top churn files.
        let mut churn_stmt = conn.prepare(
            "SELECT relative_path, COUNT(*) AS cnt FROM git_file_changes \
         WHERE workspace_id = ?1 \
         GROUP BY relative_path ORDER BY cnt DESC LIMIT 20",
        )?;
        let top_churn_files: Vec<FileChurn> = churn_stmt
            .query_map(rusqlite::params![workspace_id], |row| {
                Ok(FileChurn {
                    relative_path: row.get(0)?,
                    change_count: row.get(1)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        (summary, timeline, top_churn_files)
    }; // Lock released here.

    use crate::db::workspace_settings::co_change_hotspots;
    let top_hotspots = co_change_hotspots(db, workspace_id)?;

    Ok(RepositoryEvolution {
        summary,
        timeline,
        top_churn_files,
        top_hotspots,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::indexed_folders::insert_indexed_folder;
    use crate::db::Database;
    use tempfile::tempdir;

    fn insert_git_changes(db: &Database, ws_id: i64, entries: &[(&str, &str, i64)]) {
        let conn = db.lock().unwrap();
        for (hash, path, ts) in entries {
            conn.execute(
                "INSERT INTO git_file_changes (workspace_id, commit_hash, relative_path, timestamp) \
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![ws_id, hash, path, ts],
            )
            .unwrap();
        }
    }

    #[test]
    fn empty_evolution_report() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        let folder = dir.path().join("root");
        std::fs::create_dir(&folder).unwrap();
        let ws_id = insert_indexed_folder(&db, &folder).unwrap().id;

        let report = build_evolution_report(&db, ws_id).unwrap();
        assert_eq!(report.summary.total_commits, 0);
        assert!(report.timeline.is_empty());
        assert!(report.top_churn_files.is_empty());
    }

    #[test]
    fn evolution_with_data() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        let folder = dir.path().join("root");
        std::fs::create_dir(&folder).unwrap();
        let ws_id = insert_indexed_folder(&db, &folder).unwrap().id;

        // Jan 2026: abc12345... (Unix: 1767225600 ≈ 2026-01-01), def67890... (1767312000 ≈ 2026-01-02)
        let ts_jan = 1767225600;
        let ts_feb = 1769904000; // 2026-02-01

        insert_git_changes(
            &db,
            ws_id,
            &[
                ("abc1234567890123456789012345678901234567", "a.ts", ts_jan),
                ("abc1234567890123456789012345678901234567", "b.ts", ts_jan),
                ("def6789012345678901234567890123456789012", "a.ts", ts_feb),
                ("def6789012345678901234567890123456789012", "c.ts", ts_feb),
            ],
        );

        let report = build_evolution_report(&db, ws_id).unwrap();
        assert_eq!(report.summary.total_commits, 2);
        assert_eq!(report.summary.total_files_changed, 3);
        assert_eq!(report.summary.total_file_changes, 4);
        assert!(report.timeline.len() >= 2);

        // a.ts changed twice — should be top churn.
        let a = report
            .top_churn_files
            .iter()
            .find(|f| f.relative_path == "a.ts")
            .unwrap();
        assert_eq!(a.change_count, 2);
    }

    #[test]
    fn epoch_to_month_conversion() {
        assert_eq!(epoch_to_month(1767225600), "2026-01");
        assert_eq!(epoch_to_month(0), "unknown");
    }
}
