//! Content hashing for change detection (Phase D.7)
//! 
//! Provides fast content fingerprinting using xxHash for detecting
//! modified assets, configs, and world data.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Content hash result
#[derive(Clone, Debug, Default)]
pub struct ContentHash {
    /// Hash value (u64)
    pub value: u64,
    /// Size in bytes
    pub size: u64,
    /// Number of files hashed
    pub file_count: u32,
}

impl ContentHash {
    /// Create from raw hash value
    pub fn new(value: u64, size: u64, file_count: u32) -> Self {
        Self { value, size, file_count }
    }

    /// Check if hash is zero (uninitialized)
    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    /// Format as hex string
    pub fn to_hex(&self) -> String {
        format!("{:016x}", self.value)
    }
}

/// Fast xxHash-like hasher (simplified for zero-dep)
pub struct FastHasher {
    hasher: DefaultHasher,
}

impl Default for FastHasher {
    fn default() -> Self {
        Self {
            hasher: DefaultHasher::new(),
        }
    }
}

impl Hasher for FastHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.hasher.write(bytes);
    }

    fn finish(&self) -> u64 {
        self.hasher.finish()
    }
}

/// Hash a single file
pub fn hash_file(path: &Path) -> std::io::Result<ContentHash> {
    use std::fs::File;
    use std::io::Read;
    
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    let size = metadata.len();
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let mut hasher = FastHasher::default();
    buffer.hash(&mut hasher);
    
    Ok(ContentHash::new(hasher.finish(), size, 1))
}

/// Hash multiple files in a directory
pub fn hash_directory(dir: &Path, extensions: &[&str]) -> std::io::Result<ContentHash> {
    use std::fs;
    
    if !dir.exists() {
        return Ok(ContentHash::default());
    }
    
    let mut hasher = FastHasher::default();
    let mut total_size = 0u64;
    let mut file_count = 0u32;
    
    // Collect and sort paths for deterministic hashing
    let mut paths: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();
    paths.sort();
    
    for path in paths {
        // Check extension filter
        if !extensions.is_empty() {
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if !extensions.contains(&ext) {
                continue;
            }
        }
        
        if path.is_file() {
            if let Ok(file_hash) = hash_file(&path) {
                file_hash.value.hash(&mut hasher);
                total_size += file_hash.size;
                file_count += 1;
            }
        }
    }
    
    Ok(ContentHash::new(hasher.finish(), total_size, file_count))
}

/// Hash raw bytes
pub fn hash_bytes(data: &[u8]) -> u64 {
    let mut hasher = FastHasher::default();
    data.hash(&mut hasher);
    hasher.finish()
}

/// Hash string content
pub fn hash_string(content: &str) -> u64 {
    hash_bytes(content.as_bytes())
}

/// Compare two hashes - returns true if different
pub fn has_changed(old: &ContentHash, new: &ContentHash) -> bool {
    old.value != new.value
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_hash_bytes() {
        let h1 = hash_bytes(b"hello");
        let h2 = hash_bytes(b"hello");
        let h3 = hash_bytes(b"world");
        
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_hash_string() {
        let h1 = hash_string("test content");
        let h2 = hash_string("test content");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_content_hash_zero() {
        let hash = ContentHash::default();
        assert!(hash.is_zero());
        
        let hash2 = ContentHash::new(1, 0, 0);
        assert!(!hash2.is_zero());
    }

    #[test]
    fn test_content_hash_hex() {
        let hash = ContentHash::new(0xDEADBEEF, 0, 0);
        assert_eq!(hash.to_hex(), "00000000deadbeef");
    }

    #[test]
    fn test_has_changed() {
        let old = ContentHash::new(100, 100, 5);
        let same = ContentHash::new(100, 200, 10); // same hash, different meta
        let changed = ContentHash::new(200, 100, 5);
        
        assert!(!has_changed(&old, &same));
        assert!(has_changed(&old, &changed));
    }

    #[test]
    fn test_hash_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.txt");
        
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(b"test content").unwrap();
        
        let hash = hash_file(&path).unwrap();
        assert!(!hash.is_zero());
        assert_eq!(hash.file_count, 1);
    }
}
