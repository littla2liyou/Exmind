use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub api_key: Option<String>,
    pub root_path: Option<String>,
    pub theme: Option<String>,
}

impl AppConfig {
    /// Loads config from `<root_path>/.exmind/config.json`
    pub fn load<P: AsRef<Path>>(root_path: P) -> Result<Self> {
        let path = Self::config_path(root_path);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: AppConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Saves config to `<root_path>/.exmind/config.json`
    pub fn save<P: AsRef<Path>>(&self, root_path: P) -> Result<()> {
        let path = Self::config_path(root_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn config_path<P: AsRef<Path>>(root_path: P) -> PathBuf {
        root_path.as_ref().join(".exmind").join("config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::AppConfig;
    use anyhow::Result;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestRootGuard {
        path: PathBuf,
    }

    impl TestRootGuard {
        fn new(path: PathBuf) -> Self {
            Self { path }
        }

        fn path(&self) -> &PathBuf {
            &self.path
        }
    }

    impl Drop for TestRootGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn test_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_nanos();
        std::env::temp_dir().join(format!("exmind-config-test-{nanos}"))
    }

    #[test]
    fn load_returns_default_when_missing() -> Result<()> {
        let root = TestRootGuard::new(test_root());
        let config = AppConfig::load(root.path())?;
        assert_eq!(config.api_key, None);
        assert_eq!(config.root_path, None);
        assert_eq!(config.theme, None);
        Ok(())
    }

    #[test]
    fn save_and_load_use_root_local_config_path() -> Result<()> {
        let root = TestRootGuard::new(test_root());
        let config = AppConfig {
            api_key: Some("test-key".to_string()),
            root_path: Some(root.path().to_string_lossy().to_string()),
            theme: Some("dark".to_string()),
        };
        config.save(root.path())?;

        let config_file = root.path().join(".exmind").join("config.json");
        assert!(config_file.exists());

        let loaded = AppConfig::load(root.path())?;
        assert_eq!(loaded.api_key.as_deref(), Some("test-key"));
        assert_eq!(loaded.theme.as_deref(), Some("dark"));
        assert_eq!(
            loaded.root_path.as_deref(),
            Some(root.path().to_string_lossy().as_ref())
        );
        Ok(())
    }
}
