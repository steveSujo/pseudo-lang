use std::{
    fs,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

use crate::config::{ProjectConfig, load_project_config, resolve_sandbox_root};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FSError {
    NotInSandbox,
    NotFound,
    PermissionDenied,
    TooLarge,
    IOError(String),
    InvalidPath(String),
}

impl std::fmt::Display for FSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FSError::NotInSandbox => write!(f, "Path escapes sandbox"),
            FSError::NotFound => write!(f, "File or directory not found"),
            FSError::PermissionDenied => write!(f, "Permission denied"),
            FSError::TooLarge => write!(f, "File or sandbox size limit exceeded"),
            FSError::IOError(msg) => write!(f, "IO error: {msg}"),
            FSError::InvalidPath(msg) => write!(f, "Invalid path: {msg}"),
        }
    }
}

impl std::error::Error for FSError {}
static CONFIG: LazyLock<Mutex<Option<ProjectConfig>>> = LazyLock::new(|| Mutex::new(None));

/// Initialize sandbox with project config
pub fn init_sandbox_with_config(config: ProjectConfig) -> Result<(), FSError> {
    let mut guard = CONFIG.lock().map_err(|_| FSError::IOError("Lock poisoned".to_string()))?;
    if guard.is_some() {
        return Err(FSError::IOError("Sandbox already initialized".to_string()));
    }
    *guard = Some(config);
    init_sandbox(None)
}

fn get_config() -> ProjectConfig {
    CONFIG.lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(load_project_config)
}

/// Get config from static or load from project
fn config() -> ProjectConfig {
    get_config()
}

/// Get max file size from config
fn max_file_size() -> u64 {
    config().sandbox.max_file_size
}

fn max_sandbox_size() -> u64 {
    config().sandbox.max_sandbox_size
}

static SANDBOX_ROOT: LazyLock<Mutex<Option<PathBuf>>> = LazyLock::new(|| Mutex::new(None));

/// Initialize the sandbox root directory.
/// Must be called before any filesystem operations.
pub fn init_sandbox(root: Option<PathBuf>) -> Result<(), FSError> {
    let config = config();
    let root = root.unwrap_or_else(|| resolve_sandbox_root(&config));

    let canonical = match root.canonicalize() {
        Ok(c) => c,
        Err(e) => {
            // If sandbox doesn't exist, create it
            if e.kind() == std::io::ErrorKind::NotFound {
                fs::create_dir_all(&root).map_err(|e| FSError::IOError(e.to_string()))?;
                root.canonicalize().map_err(|e| FSError::IOError(e.to_string()))?
            } else {
                return Err(FSError::IOError(e.to_string()));
            }
        }
    };

    let mut guard = SANDBOX_ROOT.lock().map_err(|_| FSError::IOError("Lock poisoned".to_string()))?;
    if guard.is_some() {
        return Err(FSError::IOError("Sandbox already initialized".to_string()));
    }
    *guard = Some(canonical.clone());

    // Verify sandbox size limit
    check_sandbox_size(&canonical)?;

    Ok(())
}


/// Get the sandbox root path.
fn sandbox_root() -> Result<PathBuf, FSError> {
    let guard = SANDBOX_ROOT.lock().map_err(|_| FSError::IOError("Lock poisoned".to_string()))?;
    guard.clone().ok_or(FSError::IOError(
        "Sandbox not initialized. Call init_sandbox() first.".to_string(),
    ))
}

/// Resolve a user-provided path to a canonical path within the sandbox.
/// For writes, allow paths that don't exist yet (parent must exist).
fn resolve_path(user_path: &str) -> Result<PathBuf, FSError> {
    let root = sandbox_root()?;

    // Reject absolute paths
    let path = Path::new(user_path);
    if path.is_absolute() {
        return Err(FSError::NotInSandbox);
    }

    // Reject paths with .. components (before canonicalization)
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(FSError::NotInSandbox);
        }
    }

    // Join with sandbox root
    let full_path = root.join(path);

    // Try to canonicalize - if it doesn't exist, canonicalize the parent
    let canonical = if full_path.exists() {
        full_path.canonicalize().map_err(|e| FSError::IOError(e.to_string()))?
    } else {
        // For new files, canonicalize the parent directory
        let parent = full_path.parent().unwrap_or(&root);
        let canonical_parent = parent.canonicalize().map_err(|e| FSError::IOError(e.to_string()))?;
        canonical_parent.join(full_path.file_name().unwrap_or_default())
    };

    // Verify the canonical path is still within sandbox
    if !canonical.starts_with(root) {
        return Err(FSError::NotInSandbox);
    }

    Ok(canonical)
}

/// Calculate total size of sandbox directory.
fn sandbox_size(root: &Path) -> Result<u64, FSError> {
    let mut total = 0u64;
    for entry in fs::read_dir(root).map_err(|e| FSError::IOError(e.to_string()))? {
        let entry = entry.map_err(|e| FSError::IOError(e.to_string()))?;
        let metadata = entry.metadata().map_err(|e| FSError::IOError(e.to_string()))?;
        if metadata.is_file() {
            total += metadata.len();
        } else if metadata.is_dir() {
            total += sandbox_size(&entry.path())?;
        }
    }
    Ok(total)
}

/// Check sandbox size limit.
fn check_sandbox_size(root: &Path) -> Result<(), FSError> {
    let size = sandbox_size(root)?;
    if size > max_sandbox_size() {
        return Err(FSError::TooLarge);
    }
    Ok(())
}
 /// Check file size limit.
 fn check_file_size(path: &Path) -> Result<(), FSError> {
     let metadata = fs::metadata(path).map_err(|e| FSError::IOError(e.to_string()))?;
    if metadata.len() > max_file_size() {
         return Err(FSError::TooLarge);
     }
     Ok(())
 }


/// Read a file from the sandbox.
pub fn read_file(path: &str) -> Result<String, FSError> {
    let resolved = resolve_path(path)?;
    check_file_size(&resolved)?;
    fs::read_to_string(&resolved).map_err(|e| FSError::IOError(e.to_string()))
}

/// Write a file to the sandbox (atomic write via temp file).
pub fn write_file(path: &str, content: &str) -> Result<(), FSError> {
    let resolved = resolve_path(path)?;

    // Check content size before writing
    if content.len() as u64 > max_file_size() {
         return Err(FSError::TooLarge);
     }

    // Ensure parent directory exists
    if let Some(parent) = resolved.parent() {
        fs::create_dir_all(parent).map_err(|e| FSError::IOError(e.to_string()))?;
    }

    // Atomic write: write to temp file then rename
    let temp_path = resolved.with_extension("tmp");
    fs::write(&temp_path, content).map_err(|e| FSError::IOError(e.to_string()))?;
    fs::rename(&temp_path, &resolved).map_err(|e| FSError::IOError(e.to_string()))?;

    // Verify sandbox size after write
    let root = sandbox_root()?;
    check_sandbox_size(&root)?;

    Ok(())
}

/// List directory contents.
pub fn list_dir(path: &str) -> Result<Vec<String>, FSError> {
    let resolved = resolve_path(path)?;

    if !resolved.is_dir() {
        return Err(FSError::PermissionDenied);
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&resolved).map_err(|e| FSError::IOError(e.to_string()))? {
        let entry = entry.map_err(|e| FSError::IOError(e.to_string()))?;
        let name = entry.file_name().to_string_lossy().to_string();
        entries.push(name);
    }
    entries.sort();
    Ok(entries)
}

/// Check if path exists.
pub fn exists(path: &str) -> bool {
    resolve_path(path).map(|p| p.exists()).unwrap_or(false)
}

/// Check if path is a file.
pub fn is_file(path: &str) -> bool {
    resolve_path(path).map(|p| p.is_file()).unwrap_or(false)
}

/// Check if path is a directory.
pub fn is_dir(path: &str) -> bool {
    resolve_path(path).map(|p| p.is_dir()).unwrap_or(false)
}

/// Create a directory.
pub fn mkdir(path: &str) -> Result<(), FSError> {
    let resolved = resolve_path(path)?;

    // Check if already exists
    if resolved.exists() {
        return Err(FSError::PermissionDenied); // or custom error
    }

    fs::create_dir(&resolved).map_err(|e| FSError::IOError(e.to_string()))?;

    // Verify sandbox size
    let root = sandbox_root()?;
    check_sandbox_size(&root)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup_test_sandbox() -> PathBuf {
        let temp = std::env::temp_dir().join(format!("psudo_test_{}", std::process::id()));
        fs::create_dir_all(&temp).unwrap();
        temp
    }

    fn init_test_sandbox() -> PathBuf {
        let sandbox = setup_test_sandbox();
        INIT.call_once(|| {
            init_sandbox(Some(sandbox.clone())).unwrap();
        });
        sandbox
    }

    #[test]
    fn test_sandbox_init() {
        let sandbox = init_test_sandbox();
        assert_eq!(sandbox_root().unwrap(), sandbox.canonicalize().unwrap());
    }

    #[test]
    fn test_resolve_path() {
        let _sandbox = init_test_sandbox();

        // Valid path
        let resolved = resolve_path("test.txt").unwrap();
        assert!(resolved.starts_with(sandbox_root().unwrap()));

        // Absolute path rejected
        assert!(resolve_path("/etc/passwd").is_err());

        // Path traversal rejected
        assert!(resolve_path("../outside.txt").is_err());
    }

    #[test]
    fn test_read_write() {
        let _sandbox = init_test_sandbox();

        write_file("test.txt", "hello world").unwrap();
        let content = read_file("test.txt").unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_list_dir() {
        let _sandbox = init_test_sandbox();

        fs::create_dir_all(sandbox_root().unwrap().join("subdir")).unwrap();
        write_file("a.txt", "1").unwrap();
        write_file("b.txt", "2").unwrap();

        let entries = list_dir(".").unwrap();
        assert!(entries.contains(&"a.txt".to_string()));
        assert!(entries.contains(&"b.txt".to_string()));
        assert!(entries.contains(&"subdir".to_string()));
    }

    #[test]
    fn test_mkdir() {
        let _sandbox = init_test_sandbox();

        mkdir("newdir").unwrap();
        assert!(is_dir("newdir"));
    }

    #[test]
    fn test_file_size_limit() {
        let _sandbox = init_test_sandbox();

        let large = "x".repeat(max_file_size() as usize + 1);
        assert!(write_file("large.txt", &large).is_err());
    }
}