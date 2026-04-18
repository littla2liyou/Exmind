use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

#[tauri::command]
pub fn list_dir(path: String) -> Result<Vec<FileInfo>, String> {
    let dir = PathBuf::from(&path);
    if !dir.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !dir.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    let mut files = Vec::new();
    match std::fs::read_dir(&dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let metadata = entry.metadata().ok();
                files.push(FileInfo {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: entry.path().to_string_lossy().to_string(),
                    is_dir: entry.path().is_dir(),
                    size: metadata.map(|m| m.len()).unwrap_or(0),
                });
            }
        }
        Err(e) => return Err(format!("Failed to read directory: {}", e)),
    }

    Ok(files)
}

#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    wikimind_runtime::file_ops::read_file(&path, None, None)
        .map(|output| output.file.content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    wikimind_runtime::file_ops::write_file(&path, &content)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_dir(path: String) -> Result<(), String> {
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_path(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Ok(());
    }
    if p.is_dir() {
        std::fs::remove_dir_all(&p).map_err(|e| e.to_string())
    } else {
        std::fs::remove_file(&p).map_err(|e| e.to_string())
    }
}
