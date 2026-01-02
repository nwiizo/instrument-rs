//! Backup file management for safe modifications
//!
//! This module provides utilities for creating and managing backup files
//! before modifying source code.

use crate::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Create a backup of the file with .bak extension
///
/// Returns the path to the backup file.
pub fn create_backup(path: &Path) -> Result<PathBuf> {
    let backup_path = path.with_extension("rs.bak");
    fs::copy(path, &backup_path)?;
    Ok(backup_path)
}

/// Restore a file from its backup
///
/// Copies the backup file back to the original path.
pub fn restore_backup(original: &Path, backup: &Path) -> Result<()> {
    fs::copy(backup, original)?;
    Ok(())
}

/// Remove a backup file
pub fn remove_backup(backup: &Path) -> Result<()> {
    if backup.exists() {
        fs::remove_file(backup)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_create_backup() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");

        // Create a test file
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        // Create backup
        let backup_path = create_backup(&file_path).unwrap();

        assert!(backup_path.exists());
        assert_eq!(backup_path.extension().unwrap(), "bak");

        // Verify content matches
        let original = fs::read_to_string(&file_path).unwrap();
        let backup = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(original, backup);
    }

    #[test]
    fn test_restore_backup() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        let backup_path = dir.path().join("test.rs.bak");

        // Create original
        fs::write(&file_path, "original content").unwrap();

        // Create backup
        fs::write(&backup_path, "backup content").unwrap();

        // Modify original
        fs::write(&file_path, "modified content").unwrap();

        // Restore
        restore_backup(&file_path, &backup_path).unwrap();

        // Verify restored
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "backup content");
    }

    #[test]
    fn test_remove_backup() {
        let dir = tempdir().unwrap();
        let backup_path = dir.path().join("test.rs.bak");

        // Create backup
        fs::write(&backup_path, "content").unwrap();
        assert!(backup_path.exists());

        // Remove
        remove_backup(&backup_path).unwrap();
        assert!(!backup_path.exists());
    }

    #[test]
    fn test_remove_nonexistent_backup() {
        let dir = tempdir().unwrap();
        let backup_path = dir.path().join("nonexistent.rs.bak");

        // Should not error
        assert!(remove_backup(&backup_path).is_ok());
    }
}
