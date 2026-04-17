//! MiniMaxi-compatible Anthropic client for WikiMind
//! Config loaded from wikimind.json

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{info, debug, error};

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
}
