//! MiniMaxi-compatible Anthropic client for WikiMind
//! Config loaded from wikimind.json

use bytes::Bytes;
use futures::stream::{self, Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::pin::Pin;
use tracing::{info, debug, error};

/// SSE event types from Anthropic/MiniMaxi streaming API
#[derive(Debug, Clone)]
pub enum SseEvent {
    MessageStart { id: String, role: String },
    ContentBlockStart { index: u32, block_type: String, id: Option<String>, name: Option<String> },
    ContentBlockDelta { index: u32, delta_type: String, delta: String },
    ContentBlockStop { index: u32 },
    MessageDelta { stop_reason: Option<String> },
    MessageStop,
    Error { error: String },
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicResponse {
    pub content: String,
    pub usage: AnthropicUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct MiniMaxiResponse {
    content: Vec<ContentBlock>,
    usage: ResponseUsage,
}

#[derive(Debug, Clone, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ResponseUsage {
    #[serde(rename = "input_tokens")]
    input_tokens: u32,
    #[serde(rename = "output_tokens")]
    output_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct ApiConfig {
    base_url: String,
    auth_token: String,
    model: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    api: ApiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api: ApiConfig {
                base_url: "https://api.minimaxi.com/anthropic".to_string(),
                auth_token: String::new(),
                model: "MiniMax-M2.7-highspeed".to_string(),
            },
        }
    }
}

impl Config {
    fn load() -> Self {
        let locations: Vec<PathBuf> = [
            PathBuf::from("wikimind.json"),
            PathBuf::from(".config/wikimind.json"),
        ].into_iter().chain(dirs_config().map(|p| p.join("wikimind.json"))).collect();

        for loc in &locations {
            if let Ok(content) = fs::read_to_string(loc) {
                if let Ok(config) = serde_json::from_str(&content) {
                    info!(path = %loc.display(), "API 配置加载成功");
                    return config;
                }
            }
        }

        info!("未找到 wikimind.json，使用默认配置");
        Self::default()
    }
}

fn dirs_config() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(PathBuf::from).map(|p| p.join("WikiMind"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").ok()
            .map(PathBuf::from)
            .map(|p| p.join(".config").join("wikimind"))
    }
}

/// Create a new AnthropicClient using wikimind.json config
pub struct AnthropicClient {
    client: Client,
    api_key: String,
    base_url: String,
    pub model: String,
}

impl AnthropicClient {
    pub fn new() -> Self {
        let config = Config::load();

        Self {
            client: Client::new(),
            api_key: config.api.auth_token,
            base_url: config.api.base_url,
            model: config.api.model,
        }
    }

    pub async fn chat(&mut self, messages: Vec<AnthropicMessage>) -> Result<AnthropicResponse, String> {
        let url = format!("{}/v1/messages", self.base_url);

        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": messages.iter().map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content
                })
            }).collect::<Vec<_>>(),
            "stream": false
        });

        info!(method = "POST", url = %url, "发送 API 请求");
        debug!(body = %serde_json::to_string_pretty(&body).unwrap_or_default(), "请求 Body");

        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!(err = %e, "API 请求失败");
                e.to_string()
            })?;

        let status = response.status();
        let body_text = response.text().await.map_err(|e| {
            error!(err = %e, "读取响应体失败");
            e.to_string()
        })?;

        info!(status = %status, "收到 API 响应");
        debug!(body = %body_text, "响应 Body");

        if !status.is_success() {
            error!(status = %status, body = %body_text, "API 返回错误");
            return Err(format!("API error {}: {}", status, body_text));
        }

        let mini_response: MiniMaxiResponse = serde_json::from_str(&body_text)
            .map_err(|e| {
                error!(err = %e, "解析响应 JSON 失败");
                format!("Failed to parse response: {}", e)
            })?;

        let text = mini_response.content
            .iter()
            .filter_map(|b| b.text.clone())
            .collect::<Vec<_>>()
            .join("");

        Ok(AnthropicResponse {
            content: text,
            usage: AnthropicUsage {
                input_tokens: mini_response.usage.input_tokens,
                output_tokens: mini_response.usage.output_tokens,
            },
        })
    }

    /// Send a raw request and return (status, body_text)
    pub async fn send_request(&self, body: &serde_json::Value) -> Result<(reqwest::StatusCode, String), String> {
        let url = format!("{}/v1/messages", self.base_url);

        info!(method = "POST", url = %url, "发送 API 请求");
        debug!(body = %serde_json::to_string_pretty(body).unwrap_or_default(), "请求 Body");

        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| {
                error!(err = %e, "API 请求失败");
                e.to_string()
            })?;

        let status = response.status();
        let body_text = response.text().await.map_err(|e| {
            error!(err = %e, "读取响应体失败");
            e.to_string()
        })?;

        info!(status = %status, "收到 API 响应");
        debug!(body = %body_text, "响应 Body");

        Ok((status, body_text))
    }

    /// Send a streaming request and return a stream of SSE events
    pub async fn send_streaming_request(
        &self,
        body: &serde_json::Value,
    ) -> Result<Pin<Box<dyn Stream<Item = SseEvent> + Send>>, String> {
        let url = format!("{}/v1/messages", self.base_url);

        info!(method = "POST", url = %url, "发送流式 API 请求");
        debug!(body = %serde_json::to_string_pretty(body).unwrap_or_default(), "请求 Body");

        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| {
                error!(err = %e, "流式 API 请求失败");
                e.to_string()
            })?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            error!(status = %status, body = %body_text, "流式 API 返回错误");
            return Err(format!("API error {}: {}", status, body_text));
        }

        info!(status = %status, "收到流式 API 响应");

        // Create a boxed stream from the response body
        let stream = response.bytes_stream().flat_map(|chunk_result| {
            let chunk = chunk_result.expect("Failed to read chunk");
            let events = parse_sse_chunk(&chunk);
            stream::iter(events)
        }).boxed();

        Ok(stream)
    }
}

/// Parse SSE chunk into events
fn parse_sse_chunk(chunk: &Bytes) -> Vec<SseEvent> {
    let text = String::from_utf8_lossy(chunk);
    let mut events = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line == "data: [DONE]" {
            continue;
        }

        if let Some(json_str) = line.strip_prefix("data: ") {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                let event = parse_sse_event(&json);
                events.push(event);
            }
        }
    }

    events
}

/// Parse a single SSE data line into an SseEvent
fn parse_sse_event(json: &serde_json::Value) -> SseEvent {
    let event_type = json.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match event_type {
        "message_start" => {
            let id = json.get("message")
                .and_then(|m| m.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let role = json.get("message")
                .and_then(|m| m.get("role"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            SseEvent::MessageStart { id, role }
        }
        "content_block_start" => {
            let index = json.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let block_type = json.get("content_block")
                .and_then(|cb| cb.get("type"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            let id = json.get("id").and_then(|v| v.as_str()).map(String::from);
            let name = json.get("name")
                .or_else(|| json.get("content_block").and_then(|cb| cb.get("name")))
                .and_then(|v| v.as_str())
                .map(String::from);
            SseEvent::ContentBlockStart { index, block_type, id, name }
        }
        "content_block_delta" => {
            let index = json.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let delta_type = json.get("delta")
                .and_then(|d| d.get("type"))
                .and_then(|t| t.as_str())
                .unwrap_or("text")
                .to_string();
            let delta = if delta_type == "text" || delta_type == "text_delta" {
                json.get("delta")
                    .and_then(|d| d.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string()
            } else {
                json.get("delta")
                    .and_then(|d| d.get("input_json"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            SseEvent::ContentBlockDelta { index, delta_type, delta }
        }
        "content_block_stop" => {
            let index = json.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            SseEvent::ContentBlockStop { index }
        }
        "message_delta" => {
            let stop_reason = json.get("delta")
                .and_then(|d| d.get("stop_reason"))
                .and_then(|v| v.as_str())
                .map(String::from);
            SseEvent::MessageDelta { stop_reason }
        }
        "message_stop" => SseEvent::MessageStop,
        "error" => {
            let error_msg = json.get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            SseEvent::Error { error: error_msg }
        }
        _ => SseEvent::Unknown,
    }
}
