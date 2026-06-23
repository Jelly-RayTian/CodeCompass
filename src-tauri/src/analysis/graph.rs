use std::collections::{HashMap, HashSet};

use crate::db::Database;
use crate::error::AppError;

/// A node in the dependency graph (one indexed file).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    pub file_id: i64,
    pub name: String,
    pub relative_path: String,
    pub extension: Option<String>,
    pub incoming_count: i64,
    pub outgoing_count: i64,
}

/// A directed edge representing an import relationship.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    pub source_file_id: i64,
    pub target_file_id: i64,
    pub import_type: String,
    pub is_external: bool,
}

/// Summary of cycles found in the graph.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleInfo {
    pub file_ids: Vec<i64>,
    pub file_paths: Vec<String>,
}

/// The complete dependency graph for a workspace.
///
/// When the number of import-participating nodes exceeds `MAX_GRAPH_NODES`,
/// the graph is **truncated** rather than refused: `nodes`/`edges`/`cycles`
/// contain only the first `MAX_GRAPH_NODES` nodes (ordered by relative path),
/// `total_graph_nodes` holds the true count, and `truncated` is `true`.
/// The frontend uses this to show a clear warning and offer filters instead
/// of leaving the user with a silent failure.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub cycles: Vec<CycleInfo>,
    pub total_files: i64,
    pub total_imports: i64,
    /// True number of nodes that *would* participate in the graph before
    /// truncation. Equals `nodes.len()` when not truncated.
    pub total_graph_nodes: i64,
    /// `true` when the graph was truncated to `MAX_GRAPH_NODES` nodes.
    pub truncated: bool,
}

/// Maximum number of nodes returned in a single graph response. Larger
/// workspaces are truncated with `truncated = true` to keep the UI
/// responsive; the user can narrow the view with path/directory filters.
pub const MAX_GRAPH_NODES: usize = 500;

/// Builds a file-level dependency graph from the `imports` table for a
/// workspace. Nodes are indexed files that have imports or are imported by
/// other files. Edges are resolved import relationships.
///
/// Large-repo safety: when more than `MAX_GRAPH_NODES` files participate in
/// imports, only the first `MAX_GRAPH_NODES` (by relative path) are returned
/// and `truncated` is set. Edges and cycles are computed over the returned
/// node set only, so the response size is bounded.
pub fn build_graph(db: &Database, workspace_id: i64) -> Result<DependencyGraph, AppError> {
    let conn = db.lock()?;

    // Count all indexed files for summary.
    let total_files: i64 = conn.query_row(
        "SELECT COUNT(*) FROM indexed_files WHERE workspace_id = ?1 AND is_present = 1",
        rusqlite::params![workspace_id],
        |row| row.get(0),
    )?;

    // Count resolved imports (non-external, with a target).
    let total_imports: i64 = conn.query_row(
        "SELECT COUNT(*) FROM imports i \
         JOIN indexed_files f ON i.source_file_id = f.id \
         WHERE f.workspace_id = ?1 AND i.is_external = 0",
        rusqlite::params![workspace_id],
        |row| row.get(0),
    )?;

    // Collect node info: files that participate in imports (source or target).
    let mut stmt = conn.prepare(
        "SELECT id, name, relative_path, extension FROM indexed_files \
         WHERE workspace_id = ?1 AND is_present = 1 AND id IN \
         (SELECT source_file_id FROM imports WHERE is_external = 0 \
          UNION \
          SELECT resolved_target_file_id FROM imports WHERE is_external = 0 AND resolved_target_file_id IS NOT NULL) \
         ORDER BY relative_path",
    )?;

    let file_rows: Vec<(i64, String, String, Option<String>)> = stmt
        .query_map(rusqlite::params![workspace_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let total_graph_nodes = file_rows.len() as i64;
    let truncated = file_rows.len() > MAX_GRAPH_NODES;
    let kept_rows: Vec<(i64, String, String, Option<String>)> = if truncated {
        file_rows.into_iter().take(MAX_GRAPH_NODES).collect()
    } else {
        file_rows
    };

    // Keep only edges whose both endpoints are in the returned node set.
    let kept_ids: HashSet<i64> = kept_rows.iter().map(|(id, _, _, _)| *id).collect();

    // Collect edges.
    let mut edge_stmt = conn.prepare(
        "SELECT i.source_file_id, i.resolved_target_file_id, i.import_type, i.is_external \
         FROM imports i \
         JOIN indexed_files f ON i.source_file_id = f.id \
         WHERE f.workspace_id = ?1 AND i.is_external = 0 AND i.resolved_target_file_id IS NOT NULL",
    )?;

    let edge_rows: Vec<(i64, i64, String, bool)> = edge_stmt
        .query_map(rusqlite::params![workspace_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get::<_, i64>(3)? != 0,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Build adjacency for incoming/outgoing counts.
    // Counts are computed over the *full* edge set so the returned nodes
    // show realistic degree even when the graph is truncated.
    let mut incoming: HashMap<i64, i64> = HashMap::new();
    let mut outgoing: HashMap<i64, i64> = HashMap::new();

    for (src, tgt, _typ, _ext) in &edge_rows {
        *outgoing.entry(*src).or_insert(0) += 1;
        *incoming.entry(*tgt).or_insert(0) += 1;
    }

    let nodes: Vec<GraphNode> = kept_rows
        .iter()
        .map(|(id, name, relative_path, ext)| GraphNode {
            file_id: *id,
            name: name.clone(),
            relative_path: relative_path.clone(),
            extension: ext.clone(),
            incoming_count: incoming.get(id).copied().unwrap_or(0),
            outgoing_count: outgoing.get(id).copied().unwrap_or(0),
        })
        .collect();

    // Edges returned to the frontend are limited to those between kept nodes
    // to avoid referencing missing nodes.
    let edges: Vec<GraphEdge> = edge_rows
        .iter()
        .filter(|(src, tgt, _, _)| kept_ids.contains(src) && kept_ids.contains(tgt))
        .map(|(src, tgt, typ, ext)| GraphEdge {
            source_file_id: *src,
            target_file_id: *tgt,
            import_type: typ.clone(),
            is_external: *ext,
        })
        .collect();

    let cycles = find_cycles(&nodes, &edges);

    Ok(DependencyGraph {
        nodes,
        edges,
        cycles,
        total_files,
        total_imports,
        total_graph_nodes,
        truncated,
    })
}

/// Detects cycles in the directed graph using DFS with colour marks.
/// Returns at most 20 cycles to avoid overwhelming output.
fn find_cycles(nodes: &[GraphNode], edges: &[GraphEdge]) -> Vec<CycleInfo> {
    let file_ids: HashSet<i64> = nodes.iter().map(|n| n.file_id).collect();
    let id_to_path: HashMap<i64, &str> = nodes
        .iter()
        .map(|n| (n.file_id, n.relative_path.as_str()))
        .collect();

    let mut adj: HashMap<i64, Vec<i64>> = HashMap::new();
    for e in edges {
        if file_ids.contains(&e.source_file_id) && file_ids.contains(&e.target_file_id) {
            adj.entry(e.source_file_id)
                .or_default()
                .push(e.target_file_id);
        }
    }

    let mut cycles: Vec<CycleInfo> = Vec::new();
    let mut visited: HashSet<i64> = HashSet::new();
    let mut stack: Vec<i64> = Vec::new();
    let mut on_stack: HashSet<i64> = HashSet::new();

    for node in nodes {
        if !visited.contains(&node.file_id) {
            dfs_cycles(
                node.file_id,
                &adj,
                &mut visited,
                &mut stack,
                &mut on_stack,
                &mut cycles,
                &id_to_path,
            );
        }
        if cycles.len() >= 20 {
            break;
        }
    }

    cycles
}

fn dfs_cycles(
    u: i64,
    adj: &HashMap<i64, Vec<i64>>,
    visited: &mut HashSet<i64>,
    stack: &mut Vec<i64>,
    on_stack: &mut HashSet<i64>,
    cycles: &mut Vec<CycleInfo>,
    id_to_path: &HashMap<i64, &str>,
) {
    if cycles.len() >= 20 {
        return;
    }
    visited.insert(u);
    stack.push(u);
    on_stack.insert(u);

    if let Some(neighbors) = adj.get(&u) {
        for &v in neighbors {
            if !visited.contains(&v) {
                dfs_cycles(v, adj, visited, stack, on_stack, cycles, id_to_path);
            } else if on_stack.contains(&v) {
                let cycle_start = stack.iter().position(|&x| x == v);
                if let Some(idx) = cycle_start {
                    let cycle_ids: Vec<i64> = stack[idx..].to_vec();
                    let file_paths: Vec<String> = cycle_ids
                        .iter()
                        .map(|id| id_to_path.get(id).unwrap_or(&"?").to_string())
                        .collect();
                    cycles.push(CycleInfo {
                        file_ids: cycle_ids,
                        file_paths,
                    });
                    if cycles.len() >= 20 {
                        return;
                    }
                }
            }
        }
    }

    stack.pop();
    on_stack.remove(&u);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::ts_js::{ImportRecord, ImportType};
    use crate::db::imports::replace_file_imports;
    use crate::db::indexed_files::upsert_files_batch;
    use crate::db::indexed_folders::insert_indexed_folder;
    use crate::db::Database;
    use tempfile::tempdir;

    fn insert_file(db: &Database, workspace_id: i64, rel: &str) -> i64 {
        use crate::db::indexed_files::FileUpsert;
        let mut batch = vec![FileUpsert {
            relative_path: rel.to_string(),
            name: rel.split('/').next_back().unwrap_or(rel).to_string(),
            parent_path: ".".to_string(),
            extension: Some("ts".to_string()),
            size_bytes: 100,
            created_at: Some(1),
            modified_at: Some(2),
            fingerprint: format!("fp:{}", rel),
            indexed_at: 1000,
            last_seen_at: 1000,
        }];
        upsert_files_batch(db, workspace_id, 0, &mut batch).unwrap();
        db.lock()
            .unwrap()
            .query_row(
                "SELECT id FROM indexed_files WHERE workspace_id = ?1 AND relative_path = ?2",
                rusqlite::params![workspace_id, rel],
                |row| row.get::<_, i64>(0),
            )
            .unwrap()
    }

    fn insert_import(
        db: &Database,
        source_id: i64,
        specifier: &str,
        import_type: ImportType,
        is_external: bool,
        resolved: Option<i64>,
    ) {
        let rec = ImportRecord {
            source_file_id: source_id,
            target_specifier: specifier.to_string(),
            resolved_target: None,
            import_type,
            is_external,
            start_line: Some(1),
            start_column: Some(1),
        };
        replace_file_imports(db, source_id, &[rec], 2000).unwrap();

        if let Some(tgt) = resolved {
            let conn = db.lock().unwrap();
            conn.execute(
                "UPDATE imports SET resolved_target_file_id = ?1, is_external = ?2 WHERE source_file_id = ?3",
                rusqlite::params![tgt, false as i64, source_id],
            )
            .unwrap();
        }
    }

    #[test]
    fn empty_graph() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        let folder = dir.path().join("root");
        std::fs::create_dir(&folder).unwrap();
        let ws_id = insert_indexed_folder(&db, &folder).unwrap().id;

        let graph = build_graph(&db, ws_id).unwrap();
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn two_node_graph() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        let folder = dir.path().join("root");
        std::fs::create_dir(&folder).unwrap();
        let ws_id = insert_indexed_folder(&db, &folder).unwrap().id;

        let a = insert_file(&db, ws_id, "a.ts");
        let b = insert_file(&db, ws_id, "b.ts");
        insert_import(&db, a, "./b", ImportType::StaticImport, false, Some(b));

        let graph = build_graph(&db, ws_id).unwrap();
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        let na = graph.nodes.iter().find(|n| n.file_id == a).unwrap();
        let nb = graph.nodes.iter().find(|n| n.file_id == b).unwrap();
        assert_eq!(na.outgoing_count, 1);
        assert_eq!(nb.incoming_count, 1);
    }

    #[test]
    fn cycle_detection() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        let folder = dir.path().join("root");
        std::fs::create_dir(&folder).unwrap();
        let ws_id = insert_indexed_folder(&db, &folder).unwrap().id;

        let a = insert_file(&db, ws_id, "a.ts");
        let b = insert_file(&db, ws_id, "b.ts");
        insert_import(&db, a, "./b", ImportType::StaticImport, false, Some(b));
        insert_import(&db, b, "./a", ImportType::StaticImport, false, Some(a));

        let graph = build_graph(&db, ws_id).unwrap();
        assert!(!graph.cycles.is_empty());
    }

    #[test]
    fn isolated_node_not_in_graph() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        let folder = dir.path().join("root");
        std::fs::create_dir(&folder).unwrap();
        let ws_id = insert_indexed_folder(&db, &folder).unwrap().id;

        let _a = insert_file(&db, ws_id, "a.ts");
        let b = insert_file(&db, ws_id, "b.ts");
        insert_import(&db, b, "react", ImportType::StaticImport, true, None);

        // a.ts has no imports, b.ts only imports external package → no internal edges.
        let graph = build_graph(&db, ws_id).unwrap();
        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn large_graph_is_truncated_not_refused() {
        // Build a graph with more than MAX_GRAPH_NODES participants and verify
        // it is returned with `truncated = true` rather than erroring.
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        let folder = dir.path().join("root");
        std::fs::create_dir(&folder).unwrap();
        let ws_id = insert_indexed_folder(&db, &folder).unwrap().id;

        // Create MAX_GRAPH_NODES + 50 files, each importing the next so they
        // all participate in the graph.
        let mut prev: Option<i64> = None;
        for i in 0..(MAX_GRAPH_NODES + 50) {
            let rel = format!("file{:04}.ts", i);
            let id = insert_file(&db, ws_id, &rel);
            if let Some(p) = prev {
                insert_import(
                    &db,
                    p,
                    &format!("./file{:04}", i),
                    ImportType::StaticImport,
                    false,
                    Some(id),
                );
            }
            prev = Some(id);
        }

        let graph = build_graph(&db, ws_id).unwrap();
        assert!(graph.truncated, "graph should be truncated, not refused");
        assert_eq!(graph.nodes.len(), MAX_GRAPH_NODES);
        assert!(
            graph.total_graph_nodes > MAX_GRAPH_NODES as i64,
            "total_graph_nodes should reflect the real count"
        );
        // Every returned edge must reference only kept nodes.
        let kept: HashSet<i64> = graph.nodes.iter().map(|n| n.file_id).collect();
        for e in &graph.edges {
            assert!(kept.contains(&e.source_file_id), "edge source must be kept");
            assert!(kept.contains(&e.target_file_id), "edge target must be kept");
        }
    }
}
