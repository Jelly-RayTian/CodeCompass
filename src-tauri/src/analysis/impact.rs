use std::collections::{HashMap, HashSet};

use crate::db::Database;
use crate::error::AppError;

/// An item potentially affected by a change to a given symbol.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AffectedItem {
    pub kind: String, // "symbol" or "file"
    pub id: i64,
    pub name: String,
    pub path: String,
    pub depth: i64,
    pub is_exported: bool,
    pub has_cycles: bool,
    pub reason: String,
}

/// Change risk assessment for a symbol.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRisk {
    pub symbol_id: i64,
    pub name: String,
    pub risk_level: String, // "low", "medium", "high"
    pub risk_score: f64,
    pub direct_dependents: i64,
    pub transitive_dependents: i64,
    pub is_exported: bool,
    pub has_cycles: bool,
    pub affected_files: Vec<AffectedItem>,
    pub affected_symbols: Vec<AffectedItem>,
    pub explanation: String,
    pub limitation: String,
}

/// Computes the change impact for a given symbol.
pub fn compute_impact(
    db: &Database,
    workspace_id: i64,
    symbol_id: i64,
) -> Result<ChangeRisk, AppError> {
    let conn = db.lock()?;

    // Get symbol info.
    let (sym_name, sym_kind, _sym_path, is_exported): (String, String, String, bool) = conn
        .query_row(
            "SELECT s.name, s.kind, f.relative_path, s.is_exported \
             FROM symbols s JOIN indexed_files f ON s.file_id = f.id \
             WHERE s.id = ?1",
            rusqlite::params![symbol_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get::<_, i64>(3)? != 0,
                ))
            },
        )?;

    // Build adjacency from references.
    let mut stmt = conn.prepare(
        "SELECT caller_symbol_id, resolved_callee_symbol_id \
         FROM symbol_references \
         WHERE workspace_id = ?1 \
         AND caller_symbol_id IS NOT NULL AND resolved_callee_symbol_id IS NOT NULL",
    )?;
    let edges: Vec<(i64, i64)> = stmt
        .query_map(rusqlite::params![workspace_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut callee_to_caller: HashMap<i64, Vec<i64>> = HashMap::new();
    for (caller, callee) in &edges {
        callee_to_caller.entry(*callee).or_default().push(*caller);
    }

    // BFS to find all transitive dependents.
    let mut visited: HashSet<i64> = HashSet::new();
    let mut queue: Vec<(i64, i64)> = vec![(symbol_id, 0)];
    let mut direct = 0;
    while let Some((current, depth)) = queue.pop() {
        if !visited.insert(current) {
            continue;
        }
        if current != symbol_id && depth == 1 {
            direct += 1;
        }
        if depth >= 5 {
            continue;
        }
        if let Some(callers) = callee_to_caller.get(&current) {
            for &c in callers {
                queue.push((c, depth + 1));
            }
        }
    }
    visited.remove(&symbol_id);

    let transitive = visited.len() as i64;

    // Detect cycles involving this symbol.
    let has_cycles = detect_cycle(&callee_to_caller, symbol_id);

    // Compute affected items.
    let mut affected_symbols: Vec<AffectedItem> = Vec::new();
    let mut affected_files: HashSet<i64> = HashSet::new();

    let mut sym_stmt = conn.prepare(
        "SELECT s.id, s.name, s.kind, f.relative_path, f.id as file_id, s.is_exported \
         FROM symbols s JOIN indexed_files f ON s.file_id = f.id WHERE s.id = ?1",
    )?;

    for &id in &visited {
        if let Ok((sid, name, _kind, path, fid, exported)) =
            sym_stmt.query_row(rusqlite::params![id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)? != 0,
                ))
            })
        {
            affected_files.insert(fid);
            affected_symbols.push(AffectedItem {
                kind: "symbol".to_string(),
                id: sid,
                name,
                path: path.clone(),
                depth: 0,
                is_exported: exported,
                has_cycles: detect_cycle(&callee_to_caller, sid),
                reason: "transitive dependent".to_string(),
            });
        }
    }

    let file_items: Vec<AffectedItem> = affected_files
        .into_iter()
        .filter_map(|fid| {
            conn.query_row(
                "SELECT relative_path FROM indexed_files WHERE id = ?1",
                rusqlite::params![fid],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .map(|path| AffectedItem {
                kind: "file".to_string(),
                id: fid,
                name: path.split('/').next_back().unwrap_or(&path).to_string(),
                path,
                depth: 0,
                is_exported: false,
                has_cycles: false,
                reason: "contains affected symbol(s)".to_string(),
            })
        })
        .collect();

    // Risk formula: scale based on transitive count, export, cycles.
    let risk_score = (transitive as f64 * 0.15
        + if is_exported { 2.0 } else { 0.0 }
        + if has_cycles { 3.0 } else { 0.0 })
    .min(10.0);

    let risk_level = if risk_score >= 7.0 {
        "high"
    } else if risk_score >= 3.0 {
        "medium"
    } else {
        "low"
    };

    let sym_name_display = sym_name.clone();
    Ok(ChangeRisk {
        symbol_id,
        name: sym_name_display.clone(),
        risk_level: risk_level.to_string(),
        risk_score,
        direct_dependents: direct,
        transitive_dependents: transitive,
        is_exported,
        has_cycles,
        affected_files: file_items,
        affected_symbols,
        explanation: format!(
            "{} is a {} with {} direct and {} transitive dependents.{}",
            sym_name_display,
            sym_kind,
            direct,
            transitive,
            if is_exported {
                " It is exported, increasing its blast radius."
            } else {
                ""
            }
        ),
        limitation: "Impact analysis is based on statically detected references only. Dynamic calls, reflection, and runtime dependency injection are not tracked.".to_string(),
    })
}

fn detect_cycle(adj: &HashMap<i64, Vec<i64>>, start: i64) -> bool {
    let mut visited: HashSet<i64> = HashSet::new();
    let mut stack: Vec<i64> = Vec::new();
    let mut on_stack: HashSet<i64> = HashSet::new();
    dfs(start, adj, &mut visited, &mut stack, &mut on_stack)
}

fn dfs(
    u: i64,
    adj: &HashMap<i64, Vec<i64>>,
    visited: &mut HashSet<i64>,
    stack: &mut Vec<i64>,
    on_stack: &mut HashSet<i64>,
) -> bool {
    visited.insert(u);
    stack.push(u);
    on_stack.insert(u);
    if let Some(neighbors) = adj.get(&u) {
        for &v in neighbors {
            if !visited.contains(&v) {
                if dfs(v, adj, visited, stack, on_stack) {
                    return true;
                }
            } else if on_stack.contains(&v) {
                return true;
            }
        }
    }
    stack.pop();
    on_stack.remove(&u);
    false
}
