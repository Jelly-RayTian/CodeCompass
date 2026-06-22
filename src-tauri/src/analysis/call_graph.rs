use std::collections::{HashMap, HashSet};

use crate::db::Database;
use crate::error::AppError;

/// A node in the call graph.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallGraphNode {
    pub symbol_id: i64,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub callers_count: i64,
    pub callees_count: i64,
    pub is_exported: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallGraphEdge {
    pub caller_id: i64,
    pub callee_id: i64,
    pub reference_type: String,
    pub source_line: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallGraph {
    pub nodes: Vec<CallGraphNode>,
    pub edges: Vec<CallGraphEdge>,
    pub cycles: Vec<Vec<i64>>,
    pub depth_limit_reached: bool,
}

/// Builds a call graph centered on a symbol with depth limit.
pub fn build_call_graph(
    db: &Database,
    workspace_id: i64,
    focus_symbol_id: Option<i64>,
    max_depth: i64,
) -> Result<CallGraph, AppError> {
    let conn = db.lock()?;

    // Collect all reference edges for the workspace.
    let mut stmt = conn.prepare(
        "SELECT caller_symbol_id, resolved_callee_symbol_id, reference_type, source_line \
         FROM symbol_references \
         WHERE workspace_id = ?1 \
         AND caller_symbol_id IS NOT NULL \
         AND resolved_callee_symbol_id IS NOT NULL",
    )?;
    let edges: Vec<(i64, i64, String, i64)> = stmt
        .query_map(rusqlite::params![workspace_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Symbol info lookup.
    let mut stmt2 = conn.prepare(
        "SELECT s.id, s.name, s.kind, f.relative_path, s.is_exported \
         FROM symbols s JOIN indexed_files f ON s.file_id = f.id \
         WHERE s.workspace_id = ?1",
    )?;
    let symbols: HashMap<i64, (String, String, String, bool)> = stmt2
        .query_map(rusqlite::params![workspace_id], |row| {
            Ok((
                row.get(0)?,
                (
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get::<_, i64>(4)? != 0,
                ),
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Build adjacency for traversal.
    let mut caller_to_callee: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut callee_to_caller: HashMap<i64, Vec<i64>> = HashMap::new();
    for (caller, callee, _, _) in &edges {
        caller_to_callee.entry(*caller).or_default().push(*callee);
        callee_to_caller.entry(*callee).or_default().push(*caller);
    }

    // Determine nodes to include (BFS from focus, limited depth).
    let root_ids: Vec<i64> = match focus_symbol_id {
        Some(id) => vec![id],
        None => symbols.keys().copied().collect(),
    };

    let mut included: HashSet<i64> = HashSet::new();
    let mut depth_limit_reached = false;
    for &root in &root_ids {
        let mut queue: Vec<(i64, i64)> = vec![(root, 0)];
        let mut visited: HashSet<i64> = HashSet::new();
        while let Some((id, depth)) = queue.pop() {
            if !visited.insert(id) {
                continue;
            }
            if depth > max_depth {
                depth_limit_reached = true;
                continue;
            }
            included.insert(id);
            // Follow callers (who calls this symbol).
            if let Some(callers) = callee_to_caller.get(&id) {
                for &c in callers {
                    queue.push((c, depth + 1));
                }
            }
            // Follow callees (who this symbol calls).
            if let Some(callees) = caller_to_callee.get(&id) {
                for &c in callees {
                    queue.push((c, depth + 1));
                }
            }
        }
    }

    let graph_edges: Vec<CallGraphEdge> = edges
        .iter()
        .filter(|(caller, callee, _, _)| included.contains(caller) || included.contains(callee))
        .map(|(caller, callee, rt, line)| CallGraphEdge {
            caller_id: *caller,
            callee_id: *callee,
            reference_type: rt.clone(),
            source_line: *line,
        })
        .collect();

    let graph_nodes: Vec<CallGraphNode> = included
        .iter()
        .filter_map(|id| {
            symbols
                .get(id)
                .map(|(name, kind, path, exported)| CallGraphNode {
                    symbol_id: *id,
                    name: name.clone(),
                    kind: kind.clone(),
                    file_path: path.clone(),
                    callers_count: callee_to_caller.get(id).map_or(0, |v| v.len() as i64),
                    callees_count: caller_to_callee.get(id).map_or(0, |v| v.len() as i64),
                    is_exported: *exported,
                })
        })
        .collect();

    // Simple cycle detection on the subgraph.
    let mut cycles: Vec<Vec<i64>> = Vec::new();
    {
        let mut adj: HashMap<i64, Vec<i64>> = HashMap::new();
        for e in &graph_edges {
            adj.entry(e.caller_id).or_default().push(e.callee_id);
        }
        let mut visited: HashSet<i64> = HashSet::new();
        let mut stack: Vec<i64> = Vec::new();
        let mut on_stack: HashSet<i64> = HashSet::new();
        for &node in &included {
            if !visited.contains(&node) {
                dfs_cycles(
                    node,
                    &adj,
                    &mut visited,
                    &mut stack,
                    &mut on_stack,
                    &mut cycles,
                );
            }
            if cycles.len() >= 10 {
                break;
            }
        }
    }

    Ok(CallGraph {
        nodes: graph_nodes,
        edges: graph_edges,
        cycles,
        depth_limit_reached,
    })
}

fn dfs_cycles(
    u: i64,
    adj: &HashMap<i64, Vec<i64>>,
    visited: &mut HashSet<i64>,
    stack: &mut Vec<i64>,
    on_stack: &mut HashSet<i64>,
    cycles: &mut Vec<Vec<i64>>,
) {
    if cycles.len() >= 10 {
        return;
    }
    visited.insert(u);
    stack.push(u);
    on_stack.insert(u);
    if let Some(neighbors) = adj.get(&u) {
        for &v in neighbors {
            if !visited.contains(&v) {
                dfs_cycles(v, adj, visited, stack, on_stack, cycles);
            } else if on_stack.contains(&v) {
                if let Some(idx) = stack.iter().position(|&x| x == v) {
                    cycles.push(stack[idx..].to_vec());
                    if cycles.len() >= 10 {
                        return;
                    }
                }
            }
        }
    }
    stack.pop();
    on_stack.remove(&u);
}
