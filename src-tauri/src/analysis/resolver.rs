use std::path::{Path, PathBuf};

/// Supported extensions tried during relative import resolution, in order.
const RESOLVE_EXTENSIONS: &[&str] = &[".ts", ".tsx", ".js", ".jsx"];
const INDEX_FILES: &[&str] = &["index.ts", "index.tsx", "index.js", "index.jsx"];

/// Attempts to resolve an import specifier relative to a source file's
/// directory, within a workspace root.
///
/// Returns `Ok(Some(path))` on success, `Ok(None)` when the import is an
/// external package, and `Err(())` when resolution fails due to path
/// traversal or missing files.
pub fn resolve_import(
    workspace_root: &Path,
    source_dir: &Path,
    specifier: &str,
) -> Result<Option<PathBuf>, ()> {
    if is_external_package(specifier) {
        return Ok(None);
    }

    // Normalise away Windows backslashes that sometimes appear in
    // require() or dynamic import arguments.
    let specifier = specifier.replace('\\', "/");

    let candidate = source_dir.join(&specifier);

    // Safety: candidate must stay inside workspace_root.
    if !path_is_inside_or_equal(workspace_root, &candidate) {
        return Err(());
    }

    // 1. Exact file (already has extension).
    if candidate.is_file() {
        return Ok(Some(candidate));
    }

    // 2. Try appending known extensions.
    for ext in RESOLVE_EXTENSIONS {
        let with_ext = candidate.with_extension(&ext[1..]); // strip leading '.'
        if with_ext.is_file() {
            return Ok(Some(with_ext));
        }
    }

    // 3. Directory → index file.
    if candidate.is_dir() {
        for index_file in INDEX_FILES {
            let index_path = candidate.join(index_file);
            if index_path.is_file() {
                return Ok(Some(index_path));
            }
        }
    }

    // Unresolved local import.
    Err(())
}

/// Returns `true` when the specifier looks like a package name rather than
/// a relative or absolute file path.
fn is_external_package(specifier: &str) -> bool {
    if specifier.starts_with('.') || specifier.starts_with('/') {
        return false;
    }
    // Windows absolute path (rare inside source, but handle).
    if specifier.len() >= 2 && specifier.as_bytes().get(1) == Some(&b':') {
        return false;
    }
    true
}

/// Returns `true` if `candidate` is the same folder as `root` or a
/// descendant of it.
fn path_is_inside_or_equal(root: &Path, candidate: &Path) -> bool {
    candidate.starts_with(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn external_package_returns_none() {
        let dir = tempdir().expect("temp dir");
        let result = resolve_import(dir.path(), dir.path(), "react");
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn relative_import_with_extension() {
        let dir = tempdir().expect("temp dir");
        std::fs::write(dir.path().join("foo.ts"), "").unwrap();
        let result = resolve_import(dir.path(), dir.path(), "./foo.ts");
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn relative_import_without_extension() {
        let dir = tempdir().expect("temp dir");
        std::fs::write(dir.path().join("bar.ts"), "").unwrap();
        let result = resolve_import(dir.path(), dir.path(), "./bar");
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn index_file_fallback() {
        let dir = tempdir().expect("temp dir");
        let sub = dir.path().join("mylib");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("index.ts"), "").unwrap();
        let result = resolve_import(dir.path(), dir.path(), "./mylib");
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn unresolved_returns_err() {
        let dir = tempdir().expect("temp dir");
        let result = resolve_import(dir.path(), dir.path(), "./ghost");
        assert!(result.is_err());
    }

    #[test]
    fn path_traversal_returns_err() {
        let dir = tempdir().expect("temp dir");
        let inner = dir.path().join("inner");
        std::fs::create_dir(&inner).unwrap();
        std::fs::write(inner.join("ok.ts"), "").unwrap();
        let result = resolve_import(&inner, &inner, "../ghost");
        assert!(result.is_err());
    }
}
