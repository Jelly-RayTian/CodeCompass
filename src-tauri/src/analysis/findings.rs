use std::collections::HashMap;

use crate::db::Database;
use crate::error::AppError;

/// A single structural finding about the codebase.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralFinding {
    pub category: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub evidence: Vec<String>,
    pub limitation: String,
    pub investigation: String,
}

/// Composes all structural findings for a workspace.
pub fn collect_findings(
    db: &Database,
    workspace_id: i64,
) -> Result<Vec<StructuralFinding>, AppError> {
    let mut findings: Vec<StructuralFinding> = Vec::new();

    findings.extend(unresolved_imports(db, workspace_id)?);
    findings.extend(large_files(db, workspace_id)?);
    findings.extend(highly_connected(db, workspace_id)?);
    findings.extend(orphaned_files(db, workspace_id)?);
    findings.extend(potentially_unused_exports(db, workspace_id)?);

    Ok(findings)
}

fn unresolved_imports(
    db: &Database,
    workspace_id: i64,
) -> Result<Vec<StructuralFinding>, AppError> {
    let mut findings = Vec::new();
    let conn = db.lock()?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM imports i \
         JOIN indexed_files f ON i.source_file_id = f.id \
         WHERE f.workspace_id = ?1 AND i.resolved_target_file_id IS NULL AND i.is_external = 0",
        rusqlite::params![workspace_id],
        |row| row.get(0),
    )?;

    if count > 0 {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT i.target_specifier FROM imports i \
             JOIN indexed_files f ON i.source_file_id = f.id \
             WHERE f.workspace_id = ?1 AND i.resolved_target_file_id IS NULL AND i.is_external = 0 \
             LIMIT 10",
        )?;
        let unresolved: Vec<String> = stmt
            .query_map(rusqlite::params![workspace_id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        findings.push(StructuralFinding {
            category: "unresolved_import".to_string(),
            severity: "warning".to_string(),
            title: format!("{} file(s) have unresolved local imports", count),
            description: "These imports could not be resolved to a file within the workspace. This may indicate missing or misnamed files.".to_string(),
            evidence: unresolved.into_iter().map(|s| format!("unresolved: {}", s)).collect(),
            limitation: "Resolution only checks .ts/.tsx/.js/.jsx extensions; files with non-standard extensions or build output may be missed.".to_string(),
            investigation: "Search for the imported path in the workspace or adjust the resolver's extension list.".to_string(),
        });
    }
    Ok(findings)
}

fn large_files(db: &Database, workspace_id: i64) -> Result<Vec<StructuralFinding>, AppError> {
    let mut findings = Vec::new();
    let conn = db.lock()?;
    // Approximate: use size_bytes from indexed_files.
    let mut stmt = conn.prepare(
        "SELECT relative_path, size_bytes FROM indexed_files \
         WHERE workspace_id = ?1 AND is_present = 1 AND size_bytes > 50000 \
         ORDER BY size_bytes DESC LIMIT 10",
    )?;
    let large: Vec<(String, i64)> = stmt
        .query_map(rusqlite::params![workspace_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if !large.is_empty() {
        let evidence: Vec<String> = large
            .iter()
            .map(|(p, s)| format!("{} ({:.1} KB)", p, *s as f64 / 1024.0))
            .collect();
        findings.push(StructuralFinding {
            category: "large_file".to_string(),
            severity: "info".to_string(),
            title: format!("{} large file(s) (>50 KB)", large.len()),
            description: "These files are larger than typical source files. Consider whether they handle too many concerns.".to_string(),
            evidence,
            limitation: "File size alone does not indicate a problem; generated files or data files may also be large.".to_string(),
            investigation: "Open each file in the viewer and check if it can be split into smaller modules.".to_string(),
        });
    }
    Ok(findings)
}

fn highly_connected(db: &Database, workspace_id: i64) -> Result<Vec<StructuralFinding>, AppError> {
    let mut findings = Vec::new();
    let conn = db.lock()?;
    // Find nodes with high (in + out) degree.
    let mut stmt = conn.prepare(
        "SELECT f.relative_path, \
         (SELECT COUNT(*) FROM imports WHERE source_file_id = f.id) AS out_d, \
         (SELECT COUNT(*) FROM imports WHERE resolved_target_file_id = f.id) AS in_d \
         FROM indexed_files f \
         WHERE f.workspace_id = ?1 AND f.is_present = 1 \
         ORDER BY (out_d + in_d) DESC LIMIT 5",
    )?;
    let connected: Vec<(String, i64, i64)> = stmt
        .query_map(rusqlite::params![workspace_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .filter(|(_, out_d, in_d)| out_d + in_d > 15)
        .collect();

    if !connected.is_empty() {
        let evidence: Vec<String> = connected
            .iter()
            .map(|(p, out_d, in_d)| {
                format!(
                    "{} (out: {}, in: {}, total: {})",
                    p,
                    out_d,
                    in_d,
                    out_d + in_d
                )
            })
            .collect();
        findings.push(StructuralFinding {
            category: "highly_connected".to_string(),
            severity: "info".to_string(),
            title: "Highly connected modules detected".to_string(),
            description: "These files have the most import relationships. They may be difficult to change without cascading effects.".to_string(),
            evidence,
            limitation: "Connection count is based on static imports only; dynamic imports and runtime dependencies are not included.".to_string(),
            investigation: "Check if these modules can be decomposed or if their API can be narrowed.".to_string(),
        });
    }
    Ok(findings)
}

fn orphaned_files(db: &Database, workspace_id: i64) -> Result<Vec<StructuralFinding>, AppError> {
    let mut findings = Vec::new();
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT f.relative_path FROM indexed_files f \
         WHERE f.workspace_id = ?1 AND f.is_present = 1 \
         AND f.id NOT IN (SELECT DISTINCT source_file_id FROM imports) \
         AND f.id NOT IN (SELECT DISTINCT resolved_target_file_id FROM imports WHERE resolved_target_file_id IS NOT NULL) \
         LIMIT 10",
    )?;
    let orphans: Vec<String> = stmt
        .query_map(rusqlite::params![workspace_id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    if !orphans.is_empty() {
        findings.push(StructuralFinding {
            category: "orphaned_file".to_string(),
            severity: "info".to_string(),
            title: format!("{} file(s) have no import relationships", orphans.len()),
            description: "These files neither import nor are imported by any other file in the workspace. They may be unused, test fixtures, or configuration files.".to_string(),
            evidence: orphans,
            limitation: "Files used only at runtime (e.g., dynamic imports, configuration loaded via fs) will appear orphaned.".to_string(),
            investigation: "Open each file and assess whether it serves a purpose. Unused files can be removed or marked as documentation-only.".to_string(),
        });
    }
    Ok(findings)
}

fn potentially_unused_exports(
    db: &Database,
    workspace_id: i64,
) -> Result<Vec<StructuralFinding>, AppError> {
    let mut findings = Vec::new();
    let conn = db.lock()?;
    // Exported symbols whose file is never imported by other files.
    let mut stmt = conn.prepare(
        "SELECT s.name, f.relative_path FROM symbols s \
         JOIN indexed_files f ON s.file_id = f.id \
         WHERE s.workspace_id = ?1 AND s.is_exported = 1 \
         AND f.id NOT IN (SELECT DISTINCT resolved_target_file_id FROM imports WHERE resolved_target_file_id IS NOT NULL) \
         LIMIT 10",
    )?;
    let unused: Vec<(String, String)> = stmt
        .query_map(rusqlite::params![workspace_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if !unused.is_empty() {
        let evidence: Vec<String> = unused
            .iter()
            .map(|(name, path)| format!("{} in {}", name, path))
            .collect();
        findings.push(StructuralFinding {
            category: "unused_export".to_string(),
            severity: "info".to_string(),
            title: format!("{} exported symbol(s) may be unused within the workspace", unused.len()),
            description: "These symbols are exported but their file is not imported by any other workspace file. They may be part of a public API or genuinely unused.".to_string(),
            evidence,
            limitation: "Import resolution only covers static imports; runtime or external consumers are not tracked.".to_string(),
            investigation: "Check whether these exports are consumed via dynamic imports, external packages, or are intended as public API.".to_string(),
        });
    }
    Ok(findings)
}
