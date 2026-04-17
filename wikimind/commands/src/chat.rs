//! Chat command with tool execution for file operations

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tracing::{info, error, debug};
use wikimind_api::providers::anthropic::SseEvent;

// Use file_ops from wikimind_runtime
use wikimind_runtime::tools::file_ops;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub usage: Option<ChatUsage>,
    pub tools_used: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Execute a tool by name with JSON input string
fn execute_tool(tool_name: &str, input_json: &str) -> Result<String, String> {
    let input: serde_json::Value = serde_json::from_str(input_json)
        .map_err(|e| format!("Invalid tool input JSON: {}", e))?;

    match tool_name {
        "Read" => {
            let path = input["path"].as_str().ok_or("Missing 'path' field")?;
            let offset = input.get("offset").and_then(|v| v.as_u64()).map(|v| v as usize);
            let limit = input.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);
            let output = file_ops::read_file(path, offset, limit)
                .map_err(|e| e.to_string())?;
            serde_json::to_string(&output).map_err(|e| e.to_string())
        }
        "Write" => {
            let path = input["path"].as_str().ok_or("Missing 'path' field")?;
            let content = input["content"].as_str().ok_or("Missing 'content' field")?;
            let output = file_ops::write_file(path, content)
                .map_err(|e| e.to_string())?;
            serde_json::to_string(&output).map_err(|e| e.to_string())
        }
        "Edit" => {
            let path = input["path"].as_str().ok_or("Missing 'path' field")?;
            let old_string = input["old_string"].as_str().ok_or("Missing 'old_string' field")?;
            let new_string = input["new_string"].as_str().ok_or("Missing 'new_string' field")?;
            let replace_all = input["replace_all"].as_bool().unwrap_or(false);
            let output = file_ops::edit_file(path, old_string, new_string, replace_all)
                .map_err(|e| e.to_string())?;
            serde_json::to_string(&output).map_err(|e| e.to_string())
        }
        "Glob" => {
            let pattern = input["pattern"].as_str().unwrap_or("**/*.md");
            let path = input["path"].as_str();
            let output = file_ops::glob_search(pattern, path)
                .map_err(|e| e.to_string())?;
            serde_json::to_string(&output).map_err(|e| e.to_string())
        }
        "Grep" => {
            let pattern = input["pattern"].as_str().ok_or("Missing 'pattern' field")?;
            let path = input["path"].as_str();
            let input_for_grep = file_ops::GrepSearchInput {
                pattern: pattern.to_string(),
                path: path.map(String::from),
                glob: input["glob"].as_str().map(String::from),
                output_mode: input["output_mode"].as_str().map(String::from),
                before: input["-B"].as_u64().map(|v| v as usize),
                after: input["-A"].as_u64().map(|v| v as usize),
                context_short: None,
                context: input["-C"].as_u64().map(|v| v as usize),
                line_numbers: Some(true),
                case_insensitive: input["-i"].as_bool(),
                file_type: input["type"].as_str().map(String::from),
                head_limit: input["head_limit"].as_u64().map(|v| v as usize),
                offset: input["offset"].as_u64().map(|v| v as usize),
                multiline: input["multiline"].as_bool(),
            };
            let output = file_ops::grep_search(&input_for_grep)
                .map_err(|e| e.to_string())?;
            serde_json::to_string(&output).map_err(|e| e.to_string())
        }
        "ListDir" => {
            let path = input["path"].as_str().ok_or("Missing 'path' field")?;
            let entries = std::fs::read_dir(path)
                .map_err(|e| e.to_string())?;
            let mut files = Vec::new();
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    // Skip hidden files
                    if name.starts_with('.') || name.starts_with('_') {
                        continue;
                    }
                    if metadata.is_dir() || name.ends_with(".md") {
                        files.push(serde_json::json!({
                            "name": name,
                            "path": entry.path().to_string_lossy(),
                            "is_dir": metadata.is_dir(),
                            "size": metadata.len()
                        }));
                    }
                }
            }
            serde_json::to_string(&files).map_err(|e| e.to_string())
        }
        _ => Err(format!("Unknown tool: {}", tool_name))
    }
}

/// List markdown files in a directory
fn list_markdown_files(dir_path: &str) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "md" {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    files
}

/// Build system prompt with workspace context and tool descriptions
fn build_system_prompt(workspace_path: Option<String>) -> String {
    let mut prompt = String::from(
        "You are WikiMind, an AI assistant that helps users manage their personal wiki knowledge base.\n\n\
        You have access to tools that let you read, write, and edit files in the user's workspace.\n\n\
        IMPORTANT: You can directly modify files using the Write and Edit tools. Always use these tools \
        when the user asks you to create, update, or modify wiki content.\n\n\
        When the user asks you to update or create wiki content, you should:\n\
        1. Use ListDir to see what files exist\n\
        2. Use Read to see the content of existing wiki files\n\
        3. Use Write or Edit to create or modify files\n\
        4. Confirm what you did to the user\n\n\
        Available tools:\n\
        - Read(path): Read a file's contents\n\
        - Write(path, content): Create or overwrite a file\n\
        - Edit(path, old_string, new_string, replace_all?): Edit specific text in a file\n\
        - ListDir(path): List files in a directory\n\
        - Glob(pattern, path?): Search for files matching a pattern\n\
        - Grep(pattern, path?, -i?): Search for text in files\n\
    \n\
        Wiki Update Command:\n\
        - /update: Compare my-notes/ and wiki/ directories, find changed files,\n\
          show diff (before/after at line level), and sync changes to wiki notes\n"
    );

    if let Some(path) = &workspace_path {
        let files = list_markdown_files(path);
        if !files.is_empty() {
            prompt.push_str(&format!("\nCurrent workspace: {}\n", path));
            prompt.push_str("Markdown files in workspace:\n");
            for f in &files {
                prompt.push_str(&format!("  - {}\n", f));
            }
        }
    }

    prompt
}

/// Define available tools for the API
fn get_tools() -> serde_json::Value {
    serde_json::json!([
        {
            "name": "Read",
            "description": "Read the contents of a file from the filesystem.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "The path to the file to read" }
                },
                "required": ["path"]
            }
        },
        {
            "name": "Write",
            "description": "Create a new file or overwrite an existing file with new content.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "The path where the file should be written" },
                    "content": { "type": "string", "description": "The content to write to the file" }
                },
                "required": ["path", "content"]
            }
        },
        {
            "name": "Edit",
            "description": "Make a targeted edit to an existing file by replacing exact text.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "The path to the file to edit" },
                    "old_string": { "type": "string", "description": "The exact text to find in the file" },
                    "new_string": { "type": "string", "description": "The replacement text" },
                    "replace_all": { "type": "boolean", "description": "Replace all occurrences (default: false)" }
                },
                "required": ["path", "old_string", "new_string"]
            }
        },
        {
            "name": "ListDir",
            "description": "List files and directories in a folder.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "The path to the directory to list" }
                },
                "required": ["path"]
            }
        },
        {
            "name": "Glob",
            "description": "Search for files matching a glob pattern.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "The glob pattern (e.g., **/*.md)" },
                    "path": { "type": "string", "description": "The base directory to search in" }
                }
            }
        },
        {
            "name": "Grep",
            "description": "Search for text within files using regex.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "The regex pattern to search for" },
                    "path": { "type": "string", "description": "The directory or file to search in" },
                    "-i": { "type": "boolean", "description": "Case insensitive search" }
                },
                "required": ["pattern"]
            }
        }
    ])
}

/// Chat with tool support - main entry point (SSE streaming)
#[tauri::command]
pub async fn chat(
    app: AppHandle,
    messages: Vec<ChatMessage>,
    workspace_path: Option<String>,
) -> Result<ChatResponse, String> {
    info!(role = "chat", message_len = messages.len(), "收到 chat 请求");

    let system_prompt = build_system_prompt(workspace_path.clone());

    // Build messages with system prompt first
    let mut all_messages: Vec<serde_json::Value> = vec![
        serde_json::json!({
            "role": "user",
            "content": system_prompt
        })
    ];

    // Add conversation messages
    for msg in &messages {
        all_messages.push(serde_json::json!({
            "role": msg.role,
            "content": msg.content
        }));
    }

    let client = wikimind_api::providers::anthropic::AnthropicClient::new();
    let tools = get_tools();
    let mut tools_used = Vec::new();
    let mut max_iterations = 128;
    let mut final_text = String::new();

    loop {
        if max_iterations == 0 {
            return Err("Too many tool calls, possible infinite loop".to_string());
        }
        max_iterations -= 1;

        let body = serde_json::json!({
            "model": client.model,
            "max_tokens": 4096,
            "messages": all_messages,
            "stream": true,
            "tools": tools
        });

        let mut stream = client.send_streaming_request(&body).await?;

        // Accumulate state from SSE stream
        let mut text_parts: Vec<String> = Vec::new();
        let mut assistant_blocks: Vec<serde_json::Value> = Vec::new();
        let mut current_tool_input = String::new();
        let mut current_tool_id: Option<String> = None;
        let mut current_tool_name: Option<String> = None;
        let mut in_tool_use = false;
        let mut stop_reason: Option<String> = None;

        while let Some(event) = stream.next().await {
            match event {
                SseEvent::ContentBlockDelta { index: _, delta_type, delta } => {
                    if delta_type == "text" || delta_type == "text_delta" {
                        // Emit token to frontend immediately
                        let _ = app.emit("chat-token", serde_json::json!({ "token": delta }));
                        text_parts.push(delta);
                    } else if delta_type == "input_json_delta" {
                        // This is input_json_delta for tool_use
                        if in_tool_use {
                            current_tool_input.push_str(&delta);
                        }
                    }
                }
                SseEvent::ContentBlockStart { index: _, block_type, id, name } => {
                    if block_type == "tool_use" {
                        in_tool_use = true;
                        current_tool_input.clear();
                        current_tool_id = id;
                        current_tool_name = name;
                    }
                }
                SseEvent::ContentBlockStop { index: _ } => {
                    if in_tool_use {
                        in_tool_use = false;
                        // Flush accumulated text before tool_use
                        if !text_parts.is_empty() {
                            let combined = text_parts.join("");
                            final_text.push_str(&combined);
                            assistant_blocks.push(serde_json::json!({
                                "type": "text",
                                "text": combined
                            }));
                            text_parts.clear();
                        }
                        // Parse accumulated tool info
                        if let Ok(input_json) = serde_json::from_str::<serde_json::Value>(&current_tool_input) {
                            // Prefer saved id/name, fall back to input_json
                            let tool_id = current_tool_id.take()
                                .or_else(|| input_json.get("id").and_then(|v| v.as_str()).map(String::from))
                                .unwrap_or_default();
                            let tool_name = current_tool_name.take()
                                .or_else(|| input_json.get("name").and_then(|v| v.as_str()).map(String::from))
                                .unwrap_or_default();
                            // Extract input from input_json, excluding id and name
                            let input = input_json.get("input")
                                .cloned()
                                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                            assistant_blocks.push(serde_json::json!({
                                "type": "tool_use",
                                "id": tool_id,
                                "name": tool_name,
                                "input": input
                            }));
                        }
                    }
                }
                SseEvent::MessageDelta { stop_reason: sr } => {
                    stop_reason = sr;
                }
                SseEvent::MessageStop => {
                    // Flush any remaining text
                    if !text_parts.is_empty() {
                        let combined = text_parts.join("");
                        final_text.push_str(&combined);
                        assistant_blocks.push(serde_json::json!({
                            "type": "text",
                            "text": combined
                        }));
                        text_parts.clear();
                    }
                    break;
                }
                SseEvent::Error { error } => {
                    error!(err = %error, "SSE 流式错误");
                    return Err(format!("SSE error: {}", error));
                }
                _ => {}
            }
        }

        // Add assistant message to all_messages (包含 tool_use blocks)
        if !assistant_blocks.is_empty() {
            all_messages.push(serde_json::json!({
                "role": "assistant",
                "content": assistant_blocks
            }));
        }

        // Determine stop_reason
        let reason = stop_reason.unwrap_or_default();

        info!(stop_reason = %reason, "API stop_reason");

        // Log first 20 chars of accumulated text
        let preview = final_text.chars().take(20).collect::<String>();
        info!(text_preview = %preview, "本轮响应前20字");

        if reason != "tool_use" {
            // No more tool calls, we're done
            // Usage info from streaming is not easily extractable, use 0
            return Ok(ChatResponse {
                content: final_text,
                usage: None,
                tools_used: if tools_used.is_empty() { None } else { Some(tools_used) },
            });
        }

        // Process tool use results - extract from assistant_blocks
        for block in &assistant_blocks {
            if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                let tool_name = block.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let tool_input = block.get("input")
                    .map(|i| serde_json::to_string(i).unwrap_or_default())
                    .unwrap_or_default();
                let tool_id = block.get("id")
                    .and_then(|id| id.as_str())
                    .unwrap_or("");

                debug!(tool_name = %tool_name, tool_id = %tool_id, "执行工具调用");

                // Execute the tool
                let result = match execute_tool(tool_name, &tool_input) {
                    Ok(r) => r,
                    Err(e) => serde_json::json!({ "error": e }).to_string(),
                };
                tools_used.push(tool_name.to_string());

                // Add tool result to messages - Claude 风格 (claw-code 格式)
                all_messages.push(serde_json::json!({
                    "role": "user",
                    "content": [{
                        "type": "tool_result",
                        "tool_use_id": tool_id,
                        "content": [{
                            "type": "text",
                            "text": result
                        }]
                    }]
                }));
            }
        }

        // Signal end of this turn
        let _ = app.emit("chat-turn-end", serde_json::json!({}));
    }
}
