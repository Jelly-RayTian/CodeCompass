//! Safe Git command execution without shell interpolation.
//!
//! Runs `git` as a child process with argument lists (never string
//! interpolation) and treats Git failures as non-fatal: missing `git`
//! binary or non-Git directories simply return `None` or defaults.

use std::path::Path;
use std::process::Command;

/// Returns `true` if the given directory is inside a valid Git work tree.
pub fn is_git_repo(root: &Path) -> bool {
    Command::new("git")
        .args([
            "-C",
            &root.to_string_lossy(),
            "rev-parse",
            "--is-inside-work-tree",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Current short branch name (e.g. "main", "feature/foo").
pub fn current_branch(root: &Path) -> Option<String> {
    output_trim(root, &["rev-parse", "--abbrev-ref", "HEAD"])
}

/// Working-tree status: "clean" or "dirty".
pub fn working_tree_status(root: &Path) -> Option<String> {
    let out = output_trim(root, &["status", "--porcelain"])?;
    if out.is_empty() {
        Some("clean".to_string())
    } else {
        Some("dirty".to_string())
    }
}

/// Number of commits on the current branch.
pub fn commit_count(root: &Path) -> Option<i64> {
    output_trim(root, &["rev-list", "--count", "HEAD"])?
        .parse()
        .ok()
}

/// Last commit hash (abbreviated).
pub fn last_commit_short(root: &Path) -> Option<String> {
    output_trim(root, &["log", "-1", "--format=%h"])
}

/// Last commit author date (Unix epoch).
pub fn last_commit_timestamp(root: &Path) -> Option<i64> {
    output_trim(root, &["log", "-1", "--format=%ct"])?
        .parse()
        .ok()
}

/// Last commit message subject line.
pub fn last_commit_message(root: &Path) -> Option<String> {
    output_trim(root, &["log", "-1", "--format=%s"])
}

/// Last commit hash that touched a specific relative file path.
pub fn last_commit_for_file(root: &Path, relative_path: &str) -> Option<String> {
    output_trim(root, &["log", "-1", "--format=%h", "--", relative_path])
}

/// List of (commit_hash, relative_path) for recent commits (max 50 commits,
/// 200 files total).
pub fn recent_file_changes(root: &Path) -> Vec<(String, String)> {
    let out = match output_trim(
        root,
        &[
            "log",
            "-50",
            "--name-only",
            "--format=%H",
            "--",
            "*.ts",
            "*.tsx",
            "*.js",
            "*.jsx",
        ],
    ) {
        Some(o) => o,
        None => return vec![],
    };

    let mut results = Vec::new();
    let mut current_hash = String::new();
    for line in out.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Merge:") {
            continue;
        }
        if trimmed.len() == 40 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            current_hash = trimmed.to_string();
        } else if !current_hash.is_empty() {
            results.push((current_hash.clone(), trimmed.to_string()));
            if results.len() >= 200 {
                break;
            }
        }
    }
    results
}

fn output_trim(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root.as_os_str())
        .args(args)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Some(text)
}
