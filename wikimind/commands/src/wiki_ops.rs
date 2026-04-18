use std::path::PathBuf;
use tauri::AppHandle;
use wikimind_runtime::core::wiki::{self, WikiPageMeta, WikiPage};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct WikiPageResponse {
    pub meta: WikiPageMeta,
    pub content: String,
}

#[tauri::command]
pub async fn get_wiki_page(wiki_root: String, file_path: String) -> Result<WikiPageResponse, String> {
    let content = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
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
