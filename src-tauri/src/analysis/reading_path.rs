use std::collections::{HashMap, HashSet, VecDeque};

use crate::db::Database;
use crate::error::AppError;

/// A recommended reading order item.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingPathItem {
    pub order: i64,
    pub file_id: i64,
    pub relative_path: String,
    pub name: String,
    pub depth: i64,
    pub reason: String,
}

/// Generates a beginner-friendly reading path starting from detected entry
/// points, following imports in BFS order with cycle-safe handling.
pub fn generate_reading_path(
    db: &Database,
    workspace_id: i64,
    entry_file_ids: &[i64],
) -> Result<Vec<ReadingPathItem>, AppError> {
    let conn = db.lock()?;

    // Build adjacency list from imports.
    let mut adj: HashMap<i64, Vec<i64>> = HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT source_file_id, resolved_target_file_id FROM imports \
             WHERE resolved_target_file_id IS NOT NULL AND is_external = 0",
        )?;
        let edges: Vec<(i64, i64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        for (src, tgt) in edges {
            adj.entry(src).or_default().push(tgt);
        }
    }

    // File path lookup.
    let mut stmt =
        conn.prepare("SELECT id, relative_path, name FROM indexed_files WHERE workspace_id = ?1")?;
    let file_info: HashMap<i64, (String, String)> = stmt
        .query_map(rusqlite::params![workspace_id], |row| {
            Ok((row.get::<_, i64>(0)?, (row.get(1)?, row.get(2)?)))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut visited: HashSet<i64> = HashSet::new();
    let mut queue: VecDeque<(i64, i64, String)> = VecDeque::new();
    let mut result: Vec<ReadingPathItem> = Vec::new();
    let mut order: i64 = 0;

    // Seed with entry points.
    for &id in entry_file_ids {
        if visited.insert(id) {
            let (path, name) = file_info.get(&id).cloned().unwrap_or_default();
            queue.push_back((id, 0, "entry point".to_string()));
            result.push(ReadingPathItem {
                order,
                file_id: id,
                relative_path: path.clone(),
                name,
                depth: 0,
                reason: "entry point".to_string(),
            });
            order += 1;
        }
    }

    // If no entry points provided, start with files that have high in-degree.
    if entry_file_ids.is_empty() {
        let mut in_deg: HashMap<i64, i64> = HashMap::new();
        for targets in adj.values() {
            for t in targets {
                *in_deg.entry(*t).or_insert(0) += 1;
            }
        }
        let mut top: Vec<(i64, i64)> = in_deg.into_iter().collect();
        top.sort_by_key(|b| std::cmp::Reverse(b.1));
        for (id, _) in top.iter().take(3) {
            if visited.insert(*id) {
                let (path, name) = file_info.get(id).cloned().unwrap_or_default();
                queue.push_back((*id, 0, "highly imported module".to_string()));
                result.push(ReadingPathItem {
                    order,
                    file_id: *id,
                    relative_path: path.clone(),
                    name,
                    depth: 0,
                    reason: "highly imported module".to_string(),
                });
                order += 1;
            }
        }
    }

    // BFS traversal.
    while let Some((current_id, depth, reason)) = queue.pop_front() {
        if let Some(neighbors) = adj.get(&current_id) {
            for &neighbor_id in neighbors {
                if visited.insert(neighbor_id) {
                    let (path, name) = file_info.get(&neighbor_id).cloned().unwrap_or_default();
                    let new_reason = format!("imported by {}", reason);
                    queue.push_back((neighbor_id, depth + 1, path.clone()));
                    result.push(ReadingPathItem {
                        order,
                        file_id: neighbor_id,
                        relative_path: path.clone(),
                        name,
                        depth: depth + 1,
                        reason: new_reason,
                    });
                    order += 1;

                    if order >= 200 {
                        break;
                    }
                }
            }
        }
        if order >= 200 {
            break;
        }
    }

    Ok(result)
}
