use crate::db::Database;
use crate::error::AppError;

/// A candidate entry point with confidence and evidence.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryPoint {
    pub file_id: i64,
    pub relative_path: String,
    pub name: String,
    pub confidence: f64,
    pub reasons: Vec<String>,
}

/// Detects potential entry-point files for a workspace using reproducible
/// heuristics. Never claims certainty — always provides evidence.
pub fn detect_entry_points(db: &Database, workspace_id: i64) -> Result<Vec<EntryPoint>, AppError> {
    let conn = db.lock()?;

    // Collect all present TS/JS files
    let mut stmt = conn.prepare(
        "SELECT id, relative_path, name, extension FROM indexed_files \
         WHERE workspace_id = ?1 AND is_present = 1 \
         AND extension IN ('ts', 'tsx', 'js', 'jsx')",
    )?;
    let files: Vec<(i64, String, String, Option<String>)> = stmt
        .query_map(rusqlite::params![workspace_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Get import out-degree and in-degree for scoring.
    let mut out_degree: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    let mut in_degree: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    {
        let mut es = conn.prepare(
            "SELECT source_file_id, resolved_target_file_id FROM imports \
             WHERE resolved_target_file_id IS NOT NULL AND is_external = 0",
        )?;
        let edges: Vec<(i64, i64)> = es
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        for (src, tgt) in &edges {
            *out_degree.entry(*src).or_insert(0) += 1;
            *in_degree.entry(*tgt).or_insert(0) += 1;
        }
    }

    let mut candidates: Vec<EntryPoint> = Vec::new();
    let _total_out: f64 = out_degree.values().sum::<i64>() as f64;

    for (id, rel_path, name, _ext) in &files {
        let mut reasons: Vec<String> = Vec::new();
        let mut score: f64 = 0.0;

        let lower = rel_path.to_lowercase();
        let name_lower = name.to_lowercase();

        // Heuristic 1: Well-known entry-point filenames.
        let known_names = [
            "main.ts",
            "main.tsx",
            "main.js",
            "main.jsx",
            "index.ts",
            "index.tsx",
            "index.js",
            "index.jsx",
            "app.ts",
            "app.tsx",
            "app.js",
            "app.jsx",
            "server.ts",
            "server.js",
            "cli.ts",
            "cli.js",
            "entry.ts",
            "entry.tsx",
        ];
        for kn in &known_names {
            if lower.ends_with(kn) || name_lower == *kn {
                reasons.push(format!("filename matches '{}'", kn));
                score += 0.5;
                break;
            }
        }

        // Heuristic 2: Located in conventional directories.
        if lower.contains("/src/") || lower.starts_with("src/") {
            reasons.push("located in src/ directory".to_string());
            score += 0.1;
        }
        if lower.contains("/pages/") || lower.starts_with("pages/") {
            reasons.push("located in pages/ directory".to_string());
            score += 0.1;
        }

        // Heuristic 3: High import in-degree (imported by many, imports few).
        let out = out_degree.get(id).copied().unwrap_or(0);
        let inc = in_degree.get(id).copied().unwrap_or(0);
        if inc >= 3 && inc > out {
            reasons.push(format!(
                "imported by {} files (outgoing: {}) — likely shared module",
                inc, out
            ));
            // Only boost if heavily referenced with relatively few outgoing.
            if out <= inc / 2 {
                score += 0.2;
            }
        }

        // Heuristic 4: High out-degree (imports many, imported by few) —
        // typical of orchestration / bootstrap files.
        if out >= 5 && inc < out / 2 {
            reasons.push(format!(
                "imports {} modules but only imported by {} — likely orchestrator",
                out, inc
            ));
            score += 0.15;
        }

        // Heuristic 5: Contains export keywords "bootstrap", "start", "run".
        // We approximate by name only (not parsing full exports).
        if name_lower.contains("bootstrap")
            || name_lower.contains("start")
            || name_lower.contains("run")
        {
            reasons.push("name suggests startup logic".to_string());
            score += 0.1;
        }

        // Normalise confidence: clamp to [0, 1].
        if score > 0.0 {
            let confidence = score.min(1.0);
            candidates.push(EntryPoint {
                file_id: *id,
                relative_path: rel_path.clone(),
                name: name.clone(),
                confidence,
                reasons,
            });
        }
    }

    // Sort by confidence descending, at most 10 candidates.
    candidates.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
    candidates.truncate(10);

    Ok(candidates)
}
