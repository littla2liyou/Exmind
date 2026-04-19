use std::path::Path;
use std::fs;
use anyhow::{Result, Context};

/// Initializes the ExMind workspace directories.
pub fn init_workspace<P: AsRef<Path>>(root_path: P) -> Result<()> {
    let root = root_path.as_ref();

    let dirs_to_create = vec![
        root.join("workspace"),
        root.join("my-wiki").join(".exmind-mywiki").join("versions"),
        root.join("agent-wiki").join(".exmind-agentwiki").join("versions"),
        root.join(".exmind"),
    ];

    for dir in dirs_to_create {
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    }

    Ok(())
}
