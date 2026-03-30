use std::collections::HashSet;

use crate::{Completion, CompletionResponse, Error, Message};
use futures_util::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Claude {
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

impl Default for Claude {
    fn default() -> Self {
        Self {
            name: Default::default(),
            base_url: "https://api.anthropic.com/v1".to_string(),
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
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentBlock {
    Text { text: String },
    Thinking { thinking: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum StreamEvent {
    MessageStart {
        message: MessageStartData,
    },
    ContentBlockStart {
        index: usize,
        content_block: ContentBlock,
    },
    ContentBlockDelta {
        index: usize,
        delta: ContentBlockDelta,
    },
    ContentBlockStop {
        index: usize,
    },
    MessageDelta {
        delta: MessageDeltaData,
    },
    MessageStop,
    Ping,
    Error {
        error: ClaudeError,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageStartData {
    pub id: String,
    pub model: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentBlockDelta {
    TextDelta { text: String },
    ThinkingDelta { thinking: String },
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageDeltaData {
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

impl Claude {
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
            .map(|budget| ThinkingConfig {
                thinking_type: ThinkingType::Enabled,
                budget_tokens: budget as u32,
            })
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
            .post(format!("{}/messages", self.base_url))
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
            (stream, String::new()),
            |(mut stream, mut buffer)| async move {
                loop {
                    use futures_util::StreamExt;

                    if let Some(pos) = buffer.find('\n') {
                        let line = buffer.drain(..pos + 1).collect::<String>();
                        let line = line.trim();

                        if let Some(data) = line.strip_prefix("data:") {
                            let data = data.trim();
                            if data.is_empty() {
                                continue;
                            }

                            match serde_json::from_str::<StreamEvent>(data) {
                                Ok(StreamEvent::ContentBlockDelta { delta, .. }) => match delta {
                                    ContentBlockDelta::TextDelta { text } => {
                                        return Some((
                                            Ok(CompletionResponse {
                                                content: text,
                                                thinking: None,
                                            }),
                                            (stream, buffer),
                                        ));
                                    }
                                    ContentBlockDelta::ThinkingDelta { thinking } => {
                                        return Some((
                                            Ok(CompletionResponse {
                                                content: String::new(),
                                                thinking: Some(thinking),
                                            }),
                                            (stream, buffer),
                                        ));
                                    }
                                },
                                Ok(StreamEvent::Error { error }) => {
                                    return Some((
                                        Err(Error::Api(error.message)),
                                        (stream, buffer),
                                    ));
                                }
                                Ok(StreamEvent::MessageStop) => return None,
                                Ok(_) => continue,
                                Err(_) => continue,
                            }
                        }
                        continue;
                    }

                    match stream.next().await {
                        Some(Ok(chunk)) => {
                            buffer.push_str(&String::from_utf8_lossy(&chunk));
                        }
                        Some(Err(e)) => return Some((Err(Error::from(e)), (stream, buffer))),
                        None => return None,
                    }
                }
            },
        ))
    }
}

impl Completion for Claude {
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
            .post(format!("{}/messages", self.base_url))
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
            match block {
                ContentBlock::Text { text } => content.push_str(&text),
                ContentBlock::Thinking { thinking: t } => {
                    thinking = Some(t);
                }
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
