use std::path::PathBuf;
use wikimind_runtime::core::wiki::{self, WikiPageMeta};
use serde::Serialize;

#[derive(Serialize)]
pub struct WikiPageResponse {
    pub meta: WikiPageMeta,
    pub content: String,
}

#[tauri::command]
pub async fn get_wiki_page(wiki_root: String, file_path: String) -> Result<WikiPageResponse, String> {
    let root = PathBuf::from(wiki_root);
    let relative_path = PathBuf::from(file_path);

    if relative_path.is_absolute() {
        return Err("file_path must be relative to wiki_root".to_string());
    }

    let canonical_root = root.canonicalize().map_err(|e| e.to_string())?;
    let resolved_path = canonical_root.join(&relative_path);
    let canonical_page_path = resolved_path.canonicalize().map_err(|e| e.to_string())?;

    if !canonical_page_path.starts_with(&canonical_root) {
        return Err("file_path must remain within wiki_root".to_string());
    }

    let content = std::fs::read_to_string(&canonical_page_path).map_err(|e| e.to_string())?;
    let page = wiki::parse_page(&content).map_err(|e| e.to_string())?;
    
    Ok(WikiPageResponse {
        meta: page.meta,
        content: page.content,
    })
}

#[tauri::command]
pub async fn update_wiki_page(
    wiki_root: String,
    file_path: String,
    meta: WikiPageMeta,
    content: String,
) -> Result<(), String> {
    let root = PathBuf::from(wiki_root);
    let path = PathBuf::from(file_path);
    
    wiki::update_page(&root, &path, &meta, &content).map_err(|e| e.to_string())?;
    Ok(())
}
