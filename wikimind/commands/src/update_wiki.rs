//! Update wiki command with multi-agent flow

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tracing::{info, debug, error};
use wikimind_api::providers::anthropic::{AnthropicClient, AnthropicMessage};

/// Represents a single line diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_number: usize,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub diff_type: String, // "unchanged", "added", "removed"
}

/// Represents diff for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub file_path: String,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub lines: Vec<DiffLine>,
    pub change_type: String, // "added", "deleted", "modified", "unchanged"
}

/// Complete diff result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub files: Vec<FileDiff>,
    pub summary: String,
}

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

/// Parse outline from AI response
/// Expects a structured format like "1. Title - Description\n2. Title2 - Description\n..."
fn parse_outline(outline_text: &str) -> Vec<WikiSection> {
    outline_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            // Try to parse "1. Title - Description" or "1. Title" format
            let line = line.trim();
            if let Some(dot_pos) = line.find('.') {
                let after_dot = &line[dot_pos + 1..].trim();
                if let Some(dash_pos) = after_dot.find(" - ") {
                    let title = after_dot[..dash_pos].trim().to_string();
                    let description = after_dot[dash_pos + 3..].trim().to_string();
                    if !title.is_empty() {
                        return Some(WikiSection { title, content: description });
                    }
                } else if let Some(hash_pos) = after_dot.find('#') {
                    let title = after_dot[..hash_pos].trim().to_string();
                    let description = after_dot[hash_pos..].trim().to_string();
                    if !title.is_empty() {
                        return Some(WikiSection { title, content: description });
                    }
                } else if !after_dot.is_empty() {
                    return Some(WikiSection {
                        title: after_dot.to_string(),
                        content: String::new(),
                    });
                }
            }
            // Fallback: use whole line as title
            if !line.is_empty() {
                Some(WikiSection {
                    title: line.to_string(),
                    content: String::new(),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Generate outline using coordinator agent (calls AI)
async fn coordinator_generate_outline(client: &mut AnthropicClient, docs: &[String], style: &WikiStyle) -> Result<String, String> {
    info!(docs_count = docs.len(), style = %style.as_str(), "Coordinator 生成大纲中...");

    // Build document summary for the prompt
    let doc_summaries: Vec<String> = docs.iter()
        .enumerate()
        .map(|(i, doc)| {
            // Truncate each document to first 500 chars for the outline prompt
            let preview = if doc.len() > 500 { &doc[..500] } else { doc };
            format!("[文档 {}]\n{}\n...", i + 1, preview)
        })
        .collect();

    let prompt = format!(
        "你是一个知识库整理专家。请分析以下 {} 个 markdown 文档，生成一个 Wiki 知识库的章节目录结构。\n\n\
        要求：\n\
        - 生成 5-10 个主要章节\n\
        - 每个章节有标题和简短描述\n\
        - 考虑章节之间的逻辑关系\n\
        - 风格：{}\n\
        - 只输出章节列表，每行格式：序号. 标题 - 描述\n\
        - 不要输出其他内容\n\n\
        文档预览：\n{}\n",
        docs.len(),
        style.as_str(),
        doc_summaries.join("\n\n---\n\n")
    );

    let messages = vec![AnthropicMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let response = client.chat(messages).await?;
    info!(response_len = response.content.len(), "Coordinator 大纲生成完成");

    Ok(response.content)
}

/// Worker agent generates content for a section (calls AI)
async fn worker_generate_section(
    client: &mut AnthropicClient,
    section_title: &str,
    section_brief: &str,
    docs: &[String],
    style: &WikiStyle,
) -> Result<String, String> {
    info!(section = %section_title, "Worker 生成章节内容中...");

    let prompt = format!(
        "你是一个 Wiki 知识库写作专家。请根据以下参考文档，为章节「{}」撰写完整的 Wiki 内容。\n\n\
        章节简介：{}\n\n\
        要求：\n\
        - 内容详实，包含核心知识点\n\
        - 使用 [[章节名]] 格式引用其他相关章节（如果需要）\n\
        - 风格：{}\n\
        - 输出完整的 Markdown 格式内容\n\n\
        参考文档：\n{}\n",
        section_title,
        section_brief,
        style.as_str(),
        docs.join("\n\n---\n\n")
    );

    let messages = vec![AnthropicMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let response = client.chat(messages).await?;
    info!(section = %section_title, response_len = response.content.len(), "Worker 章节生成完成");

    Ok(response.content)
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
    diff_mode: Option<bool>,
) -> Result<WikiOutline, String> {
    let style = match style.as_deref() {
        Some("文艺风格") => WikiStyle::Literary,
        Some("简洁风格") => WikiStyle::Simple,
        Some("详细风格") => WikiStyle::Detailed,
        _ => WikiStyle::Technical,
    };

    // DIFF MODE: Compare my-notes/ and wiki/
    if diff_mode.unwrap_or(false) {
        if let Some(ws) = &workspace_path {
            let my_notes_dir = PathBuf::from(ws).join("my-notes");
            let wiki_dir = PathBuf::from(ws).join("wiki");

            info!(my_notes = %my_notes_dir.display(), wiki = %wiki_dir.display(), "执行 diff mode");

            let diff_result = compute_dir_diff(&my_notes_dir, &wiki_dir);

            // Emit diff result for frontend to display
            let _ = app.emit("wiki-diff-result", serde_json::json!({
                "files": &diff_result.files,
                "summary": &diff_result.summary
            }));

            let sections = generate_diff_summary_sections(&diff_result);
            return Ok(WikiOutline { sections });
        }
    }

    info!(docs_count = docs.len(), style = %style.as_str(), "=== 开始 update_wiki ===");

    // Create AI client for this request
    let mut client = AnthropicClient::new();

    // Phase 1: Coordinator reads all docs and generates outline
    info!(phase = 1, status = "reading", "Phase 1: Coordinator 生成大纲");
    let _ = app.emit("wiki-progress", serde_json::json!({
        "phase": 1,
        "status": "reading",
        "docs_count": docs.len()
    }));

    let outline_text = coordinator_generate_outline(&mut client, &docs, &style).await?;
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

    // Spawn async tasks for each section using tokio
    let style_clone = style.clone();
    let docs_clone = docs.clone();
    let mut handles = Vec::new();

    for (i, section) in sections.iter().enumerate() {
        let title = section.title.clone();
        let brief = section.content.clone();
        let docs = docs_clone.clone();
        let style = style_clone.clone();

        handles.push(tokio::spawn(async move {
            let mut client = AnthropicClient::new();
            let content = worker_generate_section(&mut client, &title, &brief, &docs, &style).await;
            (i, title, content)
        }));
    }

    // Collect results and emit progress
    let mut results = Vec::new();
    for (i, handle) in handles.into_iter().enumerate() {
        let (idx, title, content_result) = handle.await.map_err(|e| {
            error!(err = ?e, "Worker task join 失败");
            format!("Task join error: {:?}", e)
        })?;

        let content = content_result.map_err(|e| {
            error!(err = %e, "Worker 生成失败");
            e
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

    // Save wiki content to {workspace_path}/notes/{workspace_name}-wiki-notes/ directory
    if let Some(ws) = workspace_path {
        let ws_name = PathBuf::from(&ws)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "workspace".to_string());

        let wiki_dir = PathBuf::from(&ws)
            .join("notes")
            .join(format!("{}-wiki-notes", ws_name));

        if std::fs::create_dir_all(&wiki_dir).is_ok() {
            for section in &final_sections {
                let filename = format!("{}.md", section.title.replace(" ", "_"));
                let filepath = wiki_dir.join(&filename);
                let content = format!("# {}\n\n{}", section.title, section.content);
                if let Err(e) = std::fs::write(&filepath, content) {
                    error!(err = %e, path = %filepath.display(), "保存 wiki 文件失败");
                } else {
                    info!(path = %filepath.display(), "Wiki 文件已保存");
                }
            }
        }
    }

    Ok(WikiOutline { sections: final_sections })
}

/// Read all markdown files from a directory
fn read_markdown_files(dir: &PathBuf) -> HashMap<String, String> {
    let mut files = HashMap::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "md" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Some(name) = path.file_name() {
                                files.insert(name.to_string_lossy().to_string(), content);
                            }
                        }
                    }
                }
            }
        }
    }
    files
}

/// Compute line-by-line diff between two texts
fn compute_line_diff(old_text: &str, new_text: &str) -> Vec<DiffLine> {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();
    let mut diff_lines = Vec::new();

    let max_old = old_lines.len();
    let max_new = new_lines.len();
    let mut i = 0;
    let mut j = 0;

    while i < max_old || j < max_new {
        if i >= max_old {
            diff_lines.push(DiffLine {
                line_number: j + 1,
                old_content: None,
                new_content: Some(new_lines[j].to_string()),
                diff_type: "added".to_string(),
            });
            j += 1;
        } else if j >= max_new {
            diff_lines.push(DiffLine {
                line_number: i + 1,
                old_content: Some(old_lines[i].to_string()),
                new_content: None,
                diff_type: "removed".to_string(),
            });
            i += 1;
        } else if old_lines[i] == new_lines[j] {
            diff_lines.push(DiffLine {
                line_number: i + 1,
                old_content: Some(old_lines[i].to_string()),
                new_content: Some(new_lines[j].to_string()),
                diff_type: "unchanged".to_string(),
            });
            i += 1;
            j += 1;
        } else {
            diff_lines.push(DiffLine {
                line_number: i + 1,
                old_content: Some(old_lines[i].to_string()),
                new_content: None,
                diff_type: "removed".to_string(),
            });
            diff_lines.push(DiffLine {
                line_number: j + 1,
                old_content: None,
                new_content: Some(new_lines[j].to_string()),
                diff_type: "added".to_string(),
            });
            i += 1;
            j += 1;
        }
    }

    diff_lines
}

/// Compare my-notes/ and wiki/ directories and return differences
fn compute_dir_diff(my_notes_dir: &PathBuf, wiki_dir: &PathBuf) -> DiffResult {
    let mut files: Vec<FileDiff> = Vec::new();

    let my_notes_files = read_markdown_files(my_notes_dir);
    let wiki_files = read_markdown_files(wiki_dir);

    // Find added/modified files
    for (name, my_content) in &my_notes_files {
        if let Some(wiki_content) = wiki_files.get(name) {
            if my_content != wiki_content {
                let lines = compute_line_diff(wiki_content, my_content);
                files.push(FileDiff {
                    file_path: name.clone(),
                    old_path: Some(format!("{}/{}", wiki_dir.display(), name)),
                    new_path: Some(format!("{}/{}", my_notes_dir.display(), name)),
                    lines,
                    change_type: "modified".to_string(),
                });
            }
        } else {
            files.push(FileDiff {
                file_path: name.clone(),
                old_path: None,
                new_path: Some(format!("{}/{}", my_notes_dir.display(), name)),
                lines: vec![],
                change_type: "added".to_string(),
            });
        }
    }

    // Find deleted files
    for (name, _wiki_content) in &wiki_files {
        if !my_notes_files.contains_key(name) {
            files.push(FileDiff {
                file_path: name.clone(),
                old_path: Some(format!("{}/{}", wiki_dir.display(), name)),
                new_path: None,
                lines: vec![],
                change_type: "deleted".to_string(),
            });
        }
    }

    let summary = format!("{} files changed", files.len());
    DiffResult { files, summary }
}

/// Generate wiki sections summarizing the diff
fn generate_diff_summary_sections(diff_result: &DiffResult) -> Vec<WikiSection> {
    let mut sections = Vec::new();

    sections.push(WikiSection {
        title: "变更概览".to_string(),
        content: diff_result.summary.clone(),
    });

    for file_diff in &diff_result.files {
        let change_desc = match file_diff.change_type.as_str() {
            "added" => format!("新增文件: {}", file_diff.file_path),
            "deleted" => format!("删除文件: {}", file_diff.file_path),
            "modified" => {
                let added = file_diff.lines.iter().filter(|l| l.diff_type == "added").count();
                let removed = file_diff.lines.iter().filter(|l| l.diff_type == "removed").count();
                format!("修改文件: {} ({}行新增, {}行删除)", file_diff.file_path, added, removed)
            },
            _ => format!("文件: {}", file_diff.file_path),
        };

        sections.push(WikiSection {
            title: file_diff.file_path.clone(),
            content: change_desc,
        });

        // For modified files, include the actual diff
        if file_diff.change_type == "modified" && !file_diff.lines.is_empty() {
            let mut diff_content = String::new();
            for line in &file_diff.lines {
                match line.diff_type.as_str() {
                    "added" => diff_content.push_str(&format!("+ {}\n", line.new_content.as_ref().unwrap())),
                    "removed" => diff_content.push_str(&format!("- {}\n", line.old_content.as_ref().unwrap())),
                    _ => {}
                }
            }
            sections.push(WikiSection {
                title: format!("{} (diff)", file_diff.file_path),
                content: diff_content,
            });
        }
    }

    sections
}
