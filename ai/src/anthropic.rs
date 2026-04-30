use std::collections::HashSet;

use crate::{Completion, CompletionResponse, Error, Message};
use futures_util::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com";

#[derive(Clone)]
pub struct Anthropic {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub models: HashSet<String>,
    pub allow_all_models: Option<bool>,
    pub model: String,
    pub client: Client,
    pub features: Option<String>,
    pub anthropic_version: String,
}

impl Default for Anthropic {
    fn default() -> Self {
        Self {
            name: Default::default(),
            base_url: DEFAULT_ANTHROPIC_BASE_URL.to_string(),
            api_key: Default::default(),
            models: Default::default(),
            allow_all_models: Default::default(),
            model: Default::default(),
            client: Client::new(),
            features: None,
            anthropic_version: "2023-06-01".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingType {
    Enabled,
    Disabled,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThinkingConfig {
    #[serde(rename = "type")]
    pub thinking_type: ThinkingType,
    pub budget_tokens: u32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct APICompletionResponse {
    pub content: Vec<Value>,
}

enum AnthropicStreamItem {
    Chunk(CompletionResponse),
    Stop,
}

impl Anthropic {
    fn messages_url(&self) -> String {
        let base_url = self.base_url.trim().trim_end_matches('/');
        let base_url = if base_url.is_empty() {
            DEFAULT_ANTHROPIC_BASE_URL
        } else {
            base_url
        };

        if base_url.ends_with("/messages") {
            base_url.to_string()
        } else if base_url.ends_with("/v1") {
            format!("{}/messages", base_url)
        } else {
            format!("{}/v1/messages", base_url)
        }
    }

    fn extract_system_and_messages(
        &self,
        messages: Vec<Message>,
    ) -> (Option<String>, Vec<Message>) {
        let (system_messages, other_messages): (Vec<_>, Vec<_>) =
            messages.into_iter().partition(|m| m.role == "system");

        let system = if system_messages.is_empty() {
            None
        } else {
            Some(
                system_messages
                    .into_iter()
                    .map(|m| m.content)
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        };

        (system, other_messages)
    }

    fn get_thinking_config(&self) -> Option<ThinkingConfig> {
        let features = self.features.as_deref()?;
        let features_json: serde_json::Value = serde_json::from_str(features).ok()?;
        features_json
            .get("thinking")
            .and_then(|thinking| thinking.get("budget_tokens"))
            .and_then(|v| v.as_u64())
            .and_then(|budget| {
                budget.try_into().ok().map(|budget_tokens| ThinkingConfig {
                    thinking_type: ThinkingType::Enabled,
                    budget_tokens,
                })
            })
    }

    fn parse_stream_data(data: &str) -> Result<Option<AnthropicStreamItem>, Error> {
        if data == "[DONE]" {
            return Ok(Some(AnthropicStreamItem::Stop));
        }

        let value: Value = serde_json::from_str(data)?;

        match value.get("type").and_then(Value::as_str) {
            Some("content_block_delta") => {
                let Some(delta) = value.get("delta") else {
                    return Ok(None);
                };

                match delta.get("type").and_then(Value::as_str) {
                    Some("text_delta") => {
                        let content = delta
                            .get("text")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string();

                        if content.is_empty() {
                            Ok(None)
                        } else {
                            Ok(Some(AnthropicStreamItem::Chunk(CompletionResponse {
                                content,
                                thinking: None,
                            })))
                        }
                    }
                    Some("thinking_delta") => {
                        let thinking = delta
                            .get("thinking")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string();

                        if thinking.is_empty() {
                            Ok(None)
                        } else {
                            Ok(Some(AnthropicStreamItem::Chunk(CompletionResponse {
                                content: String::new(),
                                thinking: Some(thinking),
                            })))
                        }
                    }
                    Some("signature_delta" | "input_json_delta") | None => Ok(None),
                    Some(_) => Ok(None),
                }
            }
            Some("error") => {
                let error = value.get("error");
                let message = error
                    .and_then(|e| e.get("message"))
                    .and_then(Value::as_str)
                    .or_else(|| value.get("message").and_then(Value::as_str))
                    .unwrap_or("Anthropic stream returned an error");
                let error_type = error
                    .and_then(|e| e.get("type"))
                    .and_then(Value::as_str)
                    .unwrap_or("error");

                Err(Error::Api(format!("{}: {}", error_type, message)))
            }
            Some("message_stop") => Ok(Some(AnthropicStreamItem::Stop)),
            _ => Ok(None),
        }
    }

    fn append_optional(target: &mut Option<String>, value: &str) {
        if value.is_empty() {
            return;
        }

        if let Some(existing) = target {
            existing.push_str(value);
        } else {
            *target = Some(value.to_string());
        }
    }

    pub async fn create_completion_stream(
        self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + 'static, Error> {
        let (system, messages) = self.extract_system_and_messages(messages);
        let thinking = self.get_thinking_config();

        let req = CompletionRequest {
            model: self.model.clone(),
            messages,
            system,
            max_tokens: if thinking.is_some() { 16384 } else { 4096 },
            stream: Some(true),
            thinking,
        };

        let resp = self
            .client
            .post(self.messages_url())
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.anthropic_version)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&req)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let err_msg = resp.text().await.unwrap_or_default();
            return Err(Error::Api(format!("HTTP error {}: {}", status, err_msg)));
        }

        let stream = resp.bytes_stream();

        Ok(futures_util::stream::unfold(
            (stream, String::new(), String::new()),
            |(mut stream, mut buffer, mut event_data)| async move {
                loop {
                    use futures_util::StreamExt;

                    if let Some(pos) = buffer.find('\n') {
                        let line = buffer[..pos].to_string();
                        buffer.drain(..pos + 1);
                        let line = line.trim_end_matches('\r');

                        if line.is_empty() {
                            if !event_data.is_empty() {
                                let data = std::mem::take(&mut event_data);
                                match Self::parse_stream_data(data.trim()) {
                                    Ok(Some(AnthropicStreamItem::Chunk(chunk))) => {
                                        return Some((Ok(chunk), (stream, buffer, event_data)));
                                    }
                                    Ok(Some(AnthropicStreamItem::Stop)) => return None,
                                    Ok(None) => {}
                                    Err(e) => {
                                        return Some((Err(e), (stream, buffer, event_data)));
                                    }
                                }
                            }
                        } else if let Some(data) = line.strip_prefix("data:") {
                            let data = data.trim_start();
                            if !event_data.is_empty() {
                                event_data.push('\n');
                            }
                            event_data.push_str(data);
                        }
                        continue;
                    }

                    match stream.next().await {
                        Some(Ok(chunk)) => {
                            buffer.push_str(&String::from_utf8_lossy(&chunk));
                        }
                        Some(Err(e)) => {
                            return Some((Err(Error::from(e)), (stream, buffer, event_data)));
                        }
                        None => {
                            if !buffer.is_empty() {
                                let line = std::mem::take(&mut buffer);
                                let line = line.trim_end_matches('\r');
                                if let Some(data) = line.strip_prefix("data:") {
                                    let data = data.trim_start();
                                    if !event_data.is_empty() {
                                        event_data.push('\n');
                                    }
                                    event_data.push_str(data);
                                }
                            }

                            if !event_data.is_empty() {
                                let data = std::mem::take(&mut event_data);
                                match Self::parse_stream_data(data.trim()) {
                                    Ok(Some(AnthropicStreamItem::Chunk(chunk))) => {
                                        return Some((Ok(chunk), (stream, buffer, event_data)));
                                    }
                                    Ok(Some(AnthropicStreamItem::Stop)) | Ok(None) => return None,
                                    Err(e) => {
                                        return Some((Err(e), (stream, buffer, event_data)));
                                    }
                                }
                            }

                            return None;
                        }
                    }
                }
            },
        ))
    }
}

impl Completion for Anthropic {
    async fn completion(&self, messages: Vec<Message>) -> Result<CompletionResponse, Error> {
        let (system, messages) = self.extract_system_and_messages(messages);
        let thinking = self.get_thinking_config();

        let req = CompletionRequest {
            model: self.model.clone(),
            messages,
            system,
            max_tokens: if thinking.is_some() { 16384 } else { 4096 },
            stream: Some(false),
            thinking,
        };

        let resp = self
            .client
            .post(self.messages_url())
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.anthropic_version)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let err_msg = resp.text().await.unwrap_or_default();
            return Err(Error::Api(format!("HTTP error {}: {}", status, err_msg)));
        }

        let resp_json: APICompletionResponse = resp.json().await?;

        let mut content = String::new();
        let mut thinking = None;

        for block in resp_json.content {
            match block.get("type").and_then(Value::as_str) {
                Some("text") => {
                    if let Some(text) = block.get("text").and_then(Value::as_str) {
                        content.push_str(text);
                    }
                }
                Some("thinking") => {
                    if let Some(value) = block.get("thinking").and_then(Value::as_str) {
                        Self::append_optional(&mut thinking, value);
                    }
                }
                _ => {}
            }
        }

        Ok(CompletionResponse { content, thinking })
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + 'static, Error> {
        self.clone().create_completion_stream(messages).await
    }
}
