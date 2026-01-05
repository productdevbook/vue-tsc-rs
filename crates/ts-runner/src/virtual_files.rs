//! Virtual file system for generated TypeScript files.

use crate::{TsError, TsResult};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Virtual file system for managing generated TypeScript files.
#[derive(Debug)]
pub struct VirtualFileSystem {
    /// Root directory for virtual files.
    root: PathBuf,
    /// Map of original file to virtual file.
    files: HashMap<PathBuf, VirtualFile>,
}

/// A virtual file entry.
#[derive(Debug, Clone)]
pub struct VirtualFile {
    /// Original file path.
    pub original: PathBuf,
    /// Virtual file path.
    pub virtual_path: PathBuf,
    /// File extension.
    pub extension: String,
}

impl VirtualFileSystem {
    /// Create a new virtual file system.
    pub fn new(root: PathBuf) -> Self {
        // Ensure root directory exists
        let _ = fs::create_dir_all(&root);

        Self {
            root,
            files: HashMap::new(),
        }
    }

    /// Get the root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Generate a virtual path for an original file.
    pub fn virtual_path(&self, original: &Path, extension: &str) -> PathBuf {
        // Create a unique path based on the original file
        let file_name = original
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".to_string());

        let parent_hash = self.hash_path(original.parent().unwrap_or(Path::new("")));

        let virtual_name = format!("{}.{}.{}", file_name, parent_hash, extension);
        self.root.join(virtual_name)
    }

    /// Write a virtual file.
    pub fn write(&self, path: &Path, content: &str) -> TsResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                TsError::process(format!(
                    "Failed to create directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        fs::write(path, content).map_err(|e| {
            TsError::process(format!("Failed to write {}: {}", path.display(), e))
        })
    }

    /// Read a virtual file.
    pub fn read(&self, path: &Path) -> TsResult<String> {
        fs::read_to_string(path).map_err(|e| {
            TsError::process(format!("Failed to read {}: {}", path.display(), e))
        })
    }

    /// Check if a virtual file exists.
    pub fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    /// Remove a virtual file.
    pub fn remove(&self, path: &Path) -> TsResult<()> {
        if path.exists() {
            fs::remove_file(path).map_err(|e| {
                TsError::process(format!("Failed to remove {}: {}", path.display(), e))
            })?;
        }
        Ok(())
    }

    /// Clean up all virtual files.
    pub fn cleanup(&self) -> TsResult<()> {
        if self.root.exists() {
            fs::remove_dir_all(&self.root).map_err(|e| {
                TsError::process(format!(
                    "Failed to cleanup {}: {}",
                    self.root.display(),
                    e
                ))
            })?;
        }
        Ok(())
    }

    /// Generate a short hash for a path.
    fn hash_path(&self, path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        format!("{:x}", hash & 0xFFFFFF) // 6 hex chars
    }

    /// Register a virtual file mapping.
    pub fn register(&mut self, original: PathBuf, virtual_path: PathBuf, extension: String) {
        self.files.insert(
            original.clone(),
            VirtualFile {
                original,
                virtual_path,
                extension,
            },
        );
    }

    /// Get the virtual file for an original file.
    pub fn get_virtual(&self, original: &Path) -> Option<&VirtualFile> {
        self.files.get(original)
    }

    /// Get the original file for a virtual file.
    pub fn get_original(&self, virtual_path: &Path) -> Option<&PathBuf> {
        self.files
            .values()
            .find(|f| f.virtual_path == virtual_path)
            .map(|f| &f.original)
    }

    /// List all virtual files.
    pub fn list(&self) -> Vec<&VirtualFile> {
        self.files.values().collect()
    }
}

impl Drop for VirtualFileSystem {
    fn drop(&mut self) {
        // Optionally clean up on drop
        // let _ = self.cleanup();
    }
}

/// Helper to generate a tsconfig for virtual files.
pub fn generate_virtual_tsconfig(
    vfs: &VirtualFileSystem,
    base_tsconfig: Option<&Path>,
) -> TsResult<String> {
    let mut config = serde_json::json!({
        "compilerOptions": {
            "noEmit": true,
            "skipLibCheck": true,
            "strict": true
        },
        "include": [
            format!("{}/**/*.ts", vfs.root().display()),
            format!("{}/**/*.tsx", vfs.root().display())
        ]
    });

    if let Some(base) = base_tsconfig {
        config["extends"] = serde_json::Value::String(base.to_string_lossy().to_string());
    }

    serde_json::to_string_pretty(&config).map_err(|e| {
        TsError::process(format!("Failed to generate tsconfig: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_virtual_path() {
        let vfs = VirtualFileSystem::new(env::temp_dir().join("vue-tsc-rs-test"));
        let original = Path::new("/home/user/project/src/App.vue");
        let virtual_path = vfs.virtual_path(original, "ts");
        assert!(virtual_path.to_string_lossy().contains("App"));
        assert!(virtual_path.to_string_lossy().ends_with(".ts"));
    }

    #[test]
    fn test_write_read() {
        let vfs = VirtualFileSystem::new(env::temp_dir().join("vue-tsc-rs-test-rw"));
        let path = vfs.root().join("test.ts");
        vfs.write(&path, "const x = 1;").unwrap();
        let content = vfs.read(&path).unwrap();
        assert_eq!(content, "const x = 1;");
        vfs.remove(&path).unwrap();
    }
}
