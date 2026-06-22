//! Platform-specific utilities.
//!
//! Handles filesystem path normalization and platform-aware path comparisons
//! so that indexed-folder registration and root-containment checks behave
//! correctly on Windows, macOS, and Linux.

use std::path::{Component, Path, PathBuf};

use crate::error::AppError;

/// The filename used for the SQLite database within the app data directory.
pub(crate) fn database_filename() -> &'static str {
    "codecompass.db"
}

/// Normalizes an existing directory path.
///
/// The path is canonicalized (symlinks resolved, made absolute) and then
/// simplified with `dunce` so that Windows verbatim `\\?\` prefixes are
/// removed when safe. The result uses the native separator for the platform.
pub fn normalize_existing_path(path: &Path) -> Result<PathBuf, AppError> {
    let canonical = std::fs::canonicalize(path)?;
    Ok(dunce::simplified(&canonical).to_path_buf())
}

/// Returns `true` if the platform uses case-insensitive filesystem matching
/// by default.
const fn is_case_insensitive_platform() -> bool {
    cfg!(target_family = "windows") || cfg!(target_os = "macos")
}

/// Compares two normal path components using the platform's default case
/// sensitivity.
fn components_equal(left: Component<'_>, right: Component<'_>) -> bool {
    match (left, right) {
        (Component::Prefix(a), Component::Prefix(b)) => a.as_os_str() == b.as_os_str(),
        (Component::RootDir, Component::RootDir) => true,
        (Component::CurDir, Component::CurDir) => true,
        (Component::ParentDir, Component::ParentDir) => true,
        (Component::Normal(a), Component::Normal(b)) => {
            if is_case_insensitive_platform() {
                a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
            } else {
                a == b
            }
        }
        _ => false,
    }
}

/// Compares two paths using the platform's default case sensitivity.
pub fn paths_equal(left: &Path, right: &Path) -> bool {
    let left_components: Vec<_> = left.components().collect();
    let right_components: Vec<_> = right.components().collect();
    if left_components.len() != right_components.len() {
        return false;
    }
    left_components
        .into_iter()
        .zip(right_components)
        .all(|(a, b)| components_equal(a, b))
}

/// Returns `true` if `candidate` is the same folder as `root` or a
/// descendant of it, respecting platform case sensitivity.
pub fn path_is_inside_or_equal(root: &Path, candidate: &Path) -> bool {
    let root_components: Vec<_> = root.components().collect();
    let candidate_components: Vec<_> = candidate.components().collect();
    if candidate_components.len() < root_components.len() {
        return false;
    }
    root_components
        .into_iter()
        .zip(candidate_components)
        .all(|(a, b)| components_equal(a, b))
}

/// Checks whether `candidate` is a strict descendant of `root`.
pub fn path_is_strict_descendant(root: &Path, candidate: &Path) -> bool {
    let root_components: Vec<_> = root.components().collect();
    let candidate_components: Vec<_> = candidate.components().collect();
    candidate_components.len() > root_components.len()
        && root_components
            .into_iter()
            .zip(candidate_components)
            .all(|(a, b)| components_equal(a, b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn normalize_existing_path_resolves_to_absolute() {
        let dir = tempdir().expect("create temp dir");
        let normalized = normalize_existing_path(dir.path()).expect("normalize");
        assert!(normalized.is_absolute());
        assert!(normalized.exists());
    }

    #[test]
    fn path_is_strict_descendant_detects_child() {
        let parent = PathBuf::from("/home/user/docs");
        let child = PathBuf::from("/home/user/docs/school");
        assert!(path_is_strict_descendant(&parent, &child));
        assert!(!path_is_strict_descendant(&child, &parent));
    }

    #[test]
    fn paths_equal_respects_case_on_case_insensitive_platforms() {
        let left = PathBuf::from("/Users/Projects");
        let right = PathBuf::from("/users/projects");
        if is_case_insensitive_platform() {
            assert!(paths_equal(&left, &right));
        } else {
            assert!(!paths_equal(&left, &right));
        }
    }
}
