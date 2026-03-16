//! Atomic Save Utilities
//!
//! Provides safe write-through save operations: write to .tmp, sync, rename.
//! Protects against data corruption from crashes/power loss mid-write.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Atomic save: write to temp file, sync, then rename to final.
/// Creates .bak backup of existing file before overwriting.
pub fn atomic_save(path: impl AsRef<Path>, data: &[u8]) -> io::Result<()> {
    let path = path.as_ref();
    let tmp_path = path.with_extension("tmp");
    let bak_path = path.with_extension("bak");

    // Step 1: Create backup of existing file if present
    if path.exists() {
        if bak_path.exists() {
            fs::remove_file(&bak_path)?;
        }
        fs::rename(path, &bak_path)?;
    }

    // Step 2: Write to temp file
    {
        let mut file = File::create(&tmp_path)?;
        file.write_all(data)?;
        file.sync_all()?; // Ensure data is flushed to disk
    }

    // Step 3: Atomic rename (on most filesystems this is atomic)
    fs::rename(&tmp_path, path)?;

    // Step 4: Sync parent directory to ensure rename is persisted
    if let Some(parent) = path.parent() {
        if let Ok(dir) = File::open(parent) {
            let _ = dir.sync_all();
        }
    }

    Ok(())
}

/// Atomic save without backup (faster, for frequent writes like chunks).
/// Still uses tmp → sync → rename for crash safety.
pub fn atomic_save_fast(path: impl AsRef<Path>, data: &[u8]) -> io::Result<()> {
    let path = path.as_ref();
    let tmp_path = path.with_extension("tmp");

    // Write to temp file
    {
        let mut file = File::create(&tmp_path)?;
        file.write_all(data)?;
        file.sync_all()?;
    }

    // Atomic rename
    fs::rename(&tmp_path, path)?;

    // Sync parent
    if let Some(parent) = path.parent() {
        if let Ok(dir) = File::open(parent) {
            let _ = dir.sync_all();
        }
    }

    Ok(())
}

/// Check if a save file is complete (not truncated).
/// Returns true if file exists and has non-zero size.
pub fn is_save_valid(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    match fs::metadata(path) {
        Ok(meta) => meta.len() > 0,
        Err(_) => false,
    }
}

/// Attempt to recover from a corrupted/truncated save using .bak backup.
pub fn recover_from_backup(path: impl AsRef<Path>) -> io::Result<bool> {
    let path = path.as_ref();
    let bak_path = path.with_extension("bak");

    if !path.exists() && bak_path.exists() {
        // Original missing but backup exists - restore from backup
        fs::rename(&bak_path, path)?;
        tracing::info!("recovered save from backup: {}", path.display());
        return Ok(true);
    }

    // Check if current file is truncated (less than expected min size)
    if let Ok(meta) = fs::metadata(path) {
        if meta.len() < 100 && bak_path.exists() {
            // Likely truncated, restore from backup
            let _ = fs::remove_file(path);
            fs::rename(&bak_path, path)?;
            tracing::info!("recovered save from backup (truncated): {}", path.display());
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_atomic_save() {
        let test_path = PathBuf::from("test_atomic_save.tmp");
        let test_data = b"test_data_12345";

        // Clean up first
        let _ = fs::remove_file(&test_path);
        let _ = fs::remove_file(test_path.with_extension("bak"));

        // Write
        atomic_save(&test_path, test_data).expect("atomic_save failed");

        // Verify
        let loaded = fs::read(&test_path).expect("read failed");
        assert_eq!(loaded, test_data);

        // Note: backup does NOT exist on first write (nothing to back up)

        // Cleanup
        let _ = fs::remove_file(&test_path);
    }

    #[test]
    fn test_atomic_save_overwrite() {
        let test_path = PathBuf::from("test_atomic_overwrite.tmp");
        let data_v1 = b"version_1";
        let data_v2 = b"version_2";

        // Clean up
        let _ = fs::remove_file(&test_path);
        let _ = fs::remove_file(test_path.with_extension("bak"));

        // Write v1 (no backup yet)
        atomic_save(&test_path, data_v1).expect("write v1 failed");
        let loaded_v1 = fs::read(&test_path).expect("read v1 failed");
        assert_eq!(loaded_v1, data_v1);

        // Write v2 (now backup should exist from v1)
        atomic_save(&test_path, data_v2).expect("write v2 failed");
        let loaded_v2 = fs::read(&test_path).expect("read v2 failed");
        assert_eq!(loaded_v2, data_v2);

        // Backup should exist after second write
        let bak_path = test_path.with_extension("bak");
        assert!(bak_path.exists(), "backup should exist after overwrite");
        
        // Verify backup contains v1
        let backup_content = fs::read(&bak_path).expect("read backup failed");
        assert_eq!(backup_content, data_v1, "backup should contain previous version");

        // Cleanup
        let _ = fs::remove_file(&test_path);
        let _ = fs::remove_file(test_path.with_extension("bak"));
    }

    #[test]
    fn test_atomic_save_fast() {
        let test_path = PathBuf::from("test_atomic_fast.tmp");
        let test_data = b"fast_test_data";

        // Clean up
        let _ = fs::remove_file(&test_path);

        // Write
        atomic_save_fast(&test_path, test_data).expect("atomic_save_fast failed");

        // Verify
        let loaded = fs::read(&test_path).expect("read failed");
        assert_eq!(loaded, test_data);

        // Cleanup
        let _ = fs::remove_file(&test_path);
    }

    #[test]
    fn test_is_save_valid() {
        let test_path = PathBuf::from("test_valid.tmp");

        // Non-existent
        assert!(!is_save_valid(&test_path));

        // Empty file
        {
            let mut f = File::create(&test_path).expect("create failed");
            f.write_all(b"").expect("write failed");
        }
        assert!(!is_save_valid(&test_path)); // Zero size should be invalid

        // Valid file
        {
            let mut f = File::create(&test_path).expect("create failed");
            f.write_all(b"data").expect("write failed");
        }
        assert!(is_save_valid(&test_path));

        // Cleanup
        let _ = fs::remove_file(&test_path);
    }
}
