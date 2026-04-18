//! Update wiki command with multi-agent flow

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::thread;
use tauri::{AppHandle, Emitter};
use tracing::{info, debug, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSection {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WikiOutline {
    pub sections: Vec<WikiSection>,
}

/// Wiki styles that can be requested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WikiStyle {
    #[serde(rename = "文艺风格")]
    Literary,
    #[serde(rename = "技术风格")]
    Technical,
    #[serde(rename = "简洁风格")]
    Simple,
    #[serde(rename = "详细风格")]
    Detailed,
}

impl Default for WikiStyle {
    fn default() -> Self {
        WikiStyle::Technical
    }
}

impl WikiStyle {
    fn as_str(&self) -> &'static str {
        match self {
            WikiStyle::Literary => "文艺风格",
            WikiStyle::Technical => "技术风格",
            WikiStyle::Simple => "简洁风格",
            WikiStyle::Detailed => "详细风格",
        }
    }
}

/// Parse outline from AI response (simplified for MVP)
fn parse_outline(outline_text: &str) -> Vec<WikiSection> {
    // For MVP, split by lines and create simple sections
    // Real implementation would parse structured output
    outline_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
        .map(|(i, line)| WikiSection {
            title: format!("Section {}", i + 1),
            content: line.trim().to_string(),
        })
        .collect()
}

/// Generate outline using coordinator agent
fn coordinator_generate_outline(docs: &[String], style: &WikiStyle) -> String {
    // Stub: return a simple outline
    format!(
        "# Wiki Outline ({})\n\nBased on {} documents provided.",
        style.as_str(),
        docs.len()
    )
}

/// Worker agent generates content for a section
fn worker_generate_section(section_title: &str, docs: &[String], style: &WikiStyle) -> String {
    // Stub: return simple content
    format!(
        "Content for '{}' in {} style. Based on {} docs.",
        section_title,
        style.as_str(),
        docs.len()
    )
}

/// Merge sections into final wiki
fn merger_merge_sections(sections: &[WikiSection]) -> Vec<WikiSection> {
    sections.to_vec()
}

/// Multi-agent wiki update command
#[tauri::command]
pub async fn update_wiki(
    app: AppHandle,
    docs: Vec<String>,
    style: Option<String>,
    workspace_path: Option<String>,
    api_key: Option<String>,
    root_dir: Option<String>,
) -> Result<WikiOutline, String> {
    if let Some(key) = api_key {
        std::env::set_var("ANTHROPIC_API_KEY", key);
    }
    let style = match style.as_deref() {
        Some("文艺风格") => WikiStyle::Literary,
        Some("简洁风格") => WikiStyle::Simple,
        Some("详细风格") => WikiStyle::Detailed,
        _ => WikiStyle::Technical,
    };

    info!(docs_count = docs.len(), style = %style.as_str(), "=== 开始 update_wiki ===");

    // Phase 1: Coordinator reads all docs and generates outline
    info!(phase = 1, status = "reading", "Phase 1: Coordinator 生成大纲");
    let _ = app.emit("wiki-progress", serde_json::json!({
        "phase": 1,
        "status": "reading",
        "docs_count": docs.len()
    }));

    let outline_text = coordinator_generate_outline(&docs, &style);
    let sections = parse_outline(&outline_text);

    info!(phase = 1, status = "outline_ready", sections = sections.len(), "Phase 1 完成");
    let _ = app.emit("wiki-progress", serde_json::json!({
        "phase": 1,
        "status": "outline_ready",
        "sections_count": sections.len()
    }));

    // Phase 2: Parallel worker agents generate content for each section
    info!(phase = 2, status = "processing", "Phase 2: Worker agents 并行生成内容");
    let _ = app.emit("wiki-progress", serde_json::json!({
        "phase": 2,
        "status": "processing",
        "progress": 0.0
    }));

    // Spawn worker threads for each section
    let style_clone = style.clone();
    let docs_clone = docs.clone();
    let section_results: Vec<_> = sections.iter().enumerate().map(|(i, section)| {
        let title = section.title.clone();
        let docs = docs_clone.clone();
        let style = style_clone.clone();

        thread::spawn(move || {
            let title_clone = title.clone();
            (i, title, worker_generate_section(&title_clone, &docs, &style))
        })
    }).collect();

    // Collect results and emit progress
    let mut results = Vec::new();
    for (i, handle) in section_results.into_iter().enumerate() {
        let (idx, title, content) = handle.join().map_err(|e| {
            error!(err = ?e, "Worker thread join 失败");
            format!("Thread join error: {:?}", e)
        })?;
        results.push((idx, WikiSection { title, content }));

        let progress = (i + 1) as f32 / results.len() as f32;
        debug!(phase = 2, worker = i, progress = progress, "Worker progress");
        let _ = app.emit("wiki-progress", serde_json::json!({
            "phase": 2,
            "status": "processing",
            "progress": progress
        }));
    }

    // Sort by original index and extract sections
    results.sort_by_key(|(i, _)| *i);
    let sections: Vec<WikiSection> = results.into_iter().map(|(_, s)| s).collect();

    // Phase 3: Merge sections
    info!(phase = 3, status = "merging", "Phase 3: Merger 汇总");
    let _ = app.emit("wiki-progress", serde_json::json!({
        "phase": 3,
        "status": "merging"
    }));

    let final_sections = merger_merge_sections(&sections);

    info!(phase = 3, status = "complete", sections = final_sections.len(), "=== update_wiki 完成 ===");
    let _ = app.emit("wiki-progress", serde_json::json!({
        "phase": 3,
        "status": "complete",
        "sections_count": final_sections.len()
    }));

    let _ = app.emit("wiki-complete", serde_json::json!({
        "sections_count": final_sections.len()
    }));

    // Determine the actual wiki save directory based on root_dir or workspace
    let wiki_dir = if let Some(root) = root_dir {
        let mut path = PathBuf::from(root);
        path.push("agent-wiki");
        path
    } else if let Some(ws) = workspace_path {
        let ws_name = PathBuf::from(&ws)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "workspace".to_string());
        PathBuf::from("notes").join(format!("{}-wiki-notes", ws_name))
    } else {
        PathBuf::from("agent-wiki")
    };

    info!(target_dir = %wiki_dir.display(), "准备保存 Wiki 大纲到目录");

    if let Err(e) = std::fs::create_dir_all(&wiki_dir) {
        error!(dir = %wiki_dir.display(), err = %e, "创建 Wiki 目录失败");
    } else {
        for (idx, section) in final_sections.iter().enumerate() {
            // Sanitize filename carefully
            let filename = format!("{}.md", section.title
                .replace(" ", "_")
                .replace("/", "-")
                .replace("\\", "-")
                .replace(":", "-")
                .replace("*", "-")
                .replace("?", "-")
                .replace("\"", "-")
                .replace("<", "-")
                .replace(">", "-")
                .replace("|", "-")
            );
            
            let filepath = wiki_dir.join(&filename);
            
            // 基于 PRD 定义，生成 YAML Frontmatter 元数据
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let uid = format!("{}-{}", timestamp, idx);
            
            let frontmatter = format!(
r#"---
uid: "{}"
title: "{}"
type: "topic"
created_by: "ai"
source:
  kind: "workspace"
---"#,
                uid, section.title
            );
            
            let content = format!("{}\n\n# {}\n\n{}", frontmatter, section.title, section.content);
            
            match std::fs::write(&filepath, content) {
                Ok(_) => info!(path = %filepath.display(), "Wiki 文件已成功写入硬盘"),
                Err(e) => error!(err = %e, path = %filepath.display(), "保存 wiki 文件物理写入失败"),
            }
        }
    }

    Ok(WikiOutline { sections: final_sections })
}
