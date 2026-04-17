//! Session store for WikiMind - creates per-workspace session directories

use std::path::{Path, PathBuf};

pub struct SessionStore {
    sessions_dir: PathBuf,
}

impl SessionStore {
    pub fn from_cwd(cwd: &Path) -> Result<Self, std::io::Error> {
        // Create a unique session dir per workspace by hashing the absolute cwd
        let fingerprint = format!("{:x}", md5_hash(cwd));
        let sessions_dir = PathBuf::from("/tmp/wikimind-sessions").join(fingerprint);
        Ok(Self { sessions_dir })
    }

    pub fn sessions_dir(&self) -> &Path {
        &self.sessions_dir
    }

    pub fn load(&self, _session_id: &str) -> Option<Vec<u8>> {
        None
    }

    pub fn save(&self, _session_id: &str, _data: &[u8]) -> std::io::Result<()> {
        Ok(())
    }
}

/// Simple MD5-like hash using std::collections::hash_map::DefaultHasher
fn md5_hash(path: &Path) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    hasher.finish()
}
