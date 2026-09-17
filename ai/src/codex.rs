use crate::{Completion, CompletionResponse, Error, Message};
use futures_util::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

const DEFAULT_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api";
const DEFAULT_CODEX_MODEL: &str = "gpt-5.4";
const CODEX_CLIENT_VERSION: &str = "99.99.99";
const CODEX_USER_AGENT: &str = "hj-rust";

#[derive(Clone)]
pub struct OpenAICodex {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub account_id: String,
    pub models: HashSet<String>,
    pub allow_all_models: Option<bool>,
    pub model: String,
    pub client: Client,
}

impl Default for OpenAICodex {
    fn default() -> Self {
        Self {
            name: Default::default(),
            base_url: DEFAULT_CODEX_BASE_URL.to_string(),
            api_key: Default::default(),
            account_id: Default::default(),
            models: Default::default(),
            allow_all_models: Default::default(),
            model: DEFAULT_CODEX_MODEL.to_string(),
            client: Client::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct CodexRequest {
    model: String,
    store: bool,
    stream: bool,
    instructions: String,
    input: Vec<CodexInputMessage>,
    text: CodexTextOptions,
    include: [&'static str; 1],
    parallel_tool_calls: bool,
}

#[derive(Debug, Serialize)]
struct CodexInputMessage {
    role: String,
    content: Vec<CodexInputContent>,
}

#[derive(Debug, Serialize)]
struct CodexInputContent {
    r#type: &'static str,
    text: String,
}

#[derive(Debug, Serialize)]
struct CodexTextOptions {
    verbosity: &'static str,
}

#[derive(Debug, Deserialize)]
struct CodexModelsResponse {
    #[serde(default)]
    models: Vec<CodexModel>,
}

#[derive(Debug, Deserialize)]
struct CodexModel {
    slug: String,
    #[serde(default)]
    supported_in_api: bool,
}

enum CodexStreamItem {
    Chunk(CompletionResponse),
    Stop,
}

impl OpenAICodex {
    fn models_url(&self) -> String {
        let base_url = self.base_url.trim().trim_end_matches('/');
        let base_url = if base_url.is_empty() {
            DEFAULT_CODEX_BASE_URL
        } else {
            base_url
        };

        if base_url.ends_with("/codex/models") {
            base_url.to_string()
        } else if base_url.ends_with("/codex") {
            format!("{base_url}/models")
        } else {
            format!("{base_url}/codex/models")
        }
    }

    pub async fn list_models(&self) -> Result<Vec<String>, Error> {
        if self.api_key.trim().is_empty() {
            return Err(Error::Api("Codex OAuth is not connected".to_string()));
        }

        let mut request = self
            .client
            .get(self.models_url())
            .query(&[("client_version", CODEX_CLIENT_VERSION)])
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
            .header("User-Agent", CODEX_USER_AGENT)
            .header("originator", "hj-rust");
        if !self.account_id.trim().is_empty() {
            request = request.header("ChatGPT-Account-Id", &self.account_id);
        }

        let response = request.send().await?;
        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(Error::Api(codex_http_error_message(status, &error_body)));
        }

        let response: CodexModelsResponse = response.json().await?;
        Ok(response
            .models
            .into_iter()
            .filter(|model| model.supported_in_api)
            .map(|model| model.slug.trim().to_string())
            .filter(|model| !model.is_empty())
            .collect())
    }

    fn responses_url(&self) -> String {
        let base_url = self.base_url.trim().trim_end_matches('/');
        let base_url = if base_url.is_empty() {
            DEFAULT_CODEX_BASE_URL
        } else {
            base_url
        };

        if base_url.ends_with("/codex/responses") {
            base_url.to_string()
        } else if base_url.ends_with("/codex") {
            format!("{base_url}/responses")
        } else {
            format!("{base_url}/codex/responses")
        }
    }

    fn request(&self, messages: Vec<Message>) -> CodexRequest {
        let mut instructions = Vec::new();
        let mut input = Vec::new();

        for message in messages {
            if message.role == "system" {
                if !message.content.is_empty() {
                    instructions.push(message.content);
                }
                continue;
            }

            let content_type = if message.role == "assistant" {
                "output_text"
            } else {
                "input_text"
            };
            input.push(CodexInputMessage {
                role: message.role,
                content: vec![CodexInputContent {
                    r#type: content_type,
                    text: message.content,
                }],
            });
        }

        CodexRequest {
            model: self.model.clone(),
            store: false,
            stream: true,
            instructions: if instructions.is_empty() {
                "You are a helpful assistant.".to_string()
            } else {
                instructions.join("\n\n")
            },
            input,
            text: CodexTextOptions { verbosity: "low" },
            include: ["reasoning.encrypted_content"],
            parallel_tool_calls: true,
        }
    }

    fn request_builder(&self, request: &CodexRequest) -> reqwest::RequestBuilder {
        let mut builder = self
            .client
            .post(self.responses_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("OpenAI-Beta", "responses=experimental")
            .header("Accept", "text/event-stream")
            .header("Content-Type", "application/json")
            .header("User-Agent", CODEX_USER_AGENT)
            .header("originator", "hj-rust")
            .json(request);

        if !self.account_id.trim().is_empty() {
            builder = builder.header("ChatGPT-Account-Id", &self.account_id);
        }

        builder
    }

    fn parse_stream_data(data: &str) -> Result<Option<CodexStreamItem>, Error> {
        if data == "[DONE]" {
            return Ok(Some(CodexStreamItem::Stop));
        }

        let value: Value = serde_json::from_str(data)?;
        let event_type = value.get("type").and_then(Value::as_str);

        match event_type {
            Some("response.output_text.delta") => {
                let content = value
                    .get("delta")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                if content.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(CodexStreamItem::Chunk(CompletionResponse {
                        content,
                        thinking: None,
                    })))
                }
            }
            Some(
                "response.reasoning.delta"
                | "response.reasoning_summary.delta"
                | "response.reasoning_summary_text.delta"
                | "response.reasoning_text.delta",
            ) => {
                let thinking = value
                    .get("delta")
                    .and_then(Value::as_str)
                    .or_else(|| value.get("text").and_then(Value::as_str))
                    .unwrap_or_default()
                    .to_string();
                if thinking.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(CodexStreamItem::Chunk(CompletionResponse {
                        content: String::new(),
                        thinking: Some(thinking),
                    })))
                }
            }
            Some("response.completed" | "response.done") => Ok(Some(CodexStreamItem::Stop)),
            Some("response.failed" | "response.incomplete") => Err(Error::Api(
                response_error_message(&value, "Codex response failed"),
            )),
            Some("error") => Err(Error::Api(response_error_message(
                &value,
                "Codex stream returned an error",
            ))),
            _ => Ok(None),
        }
    }

    async fn stream_response(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + 'static, Error> {
        if self.api_key.trim().is_empty() {
            return Err(Error::Api("Codex OAuth is not connected".to_string()));
        }

        let request = self.request(messages);
        let response = self.request_builder(&request).send().await?;
        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(Error::Api(codex_http_error_message(status, &error_body)));
        }

        let stream = crate::http::bytes_stream(response);
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
                                    Ok(Some(CodexStreamItem::Chunk(chunk))) => {
                                        return Some((Ok(chunk), (stream, buffer, event_data)));
                                    }
                                    Ok(Some(CodexStreamItem::Stop)) => return None,
                                    Ok(None) => {}
                                    Err(error) => {
                                        return Some((Err(error), (stream, buffer, event_data)));
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
                        Some(Ok(chunk)) => buffer.push_str(&String::from_utf8_lossy(&chunk)),
                        Some(Err(error)) => {
                            return Some((Err(Error::from(error)), (stream, buffer, event_data)));
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
                                    Ok(Some(CodexStreamItem::Chunk(chunk))) => {
                                        return Some((Ok(chunk), (stream, buffer, event_data)));
                                    }
                                    Ok(Some(CodexStreamItem::Stop)) | Ok(None) => return None,
                                    Err(error) => {
                                        return Some((Err(error), (stream, buffer, event_data)));
                                    }
                                }
                            }

                            if !buffer.is_empty() {
                                return Some((
                                    Err(Error::Internal("unterminated SSE event".to_string())),
                                    (stream, buffer, event_data),
                                ));
                            }
                            return None;
                        }
                    }
                }
            },
        ))
    }
}

fn codex_http_error_message(status: reqwest::StatusCode, body: &str) -> String {
    if status == reqwest::StatusCode::FORBIDDEN && body.contains("Unable to load site") {
        return "Codex request was blocked by Cloudflare (HTTP 403) from the Workers egress path. The OAuth token is present, but this shared Worker IP is not accepted by chatgpt.com. Run Codex through a non-Workers relay or local deployment.".to_string();
    }

    let body = body.trim();
    let mut chars = body.chars();
    let shortened: String = chars.by_ref().take(2_000).collect();
    let body = if chars.next().is_some() {
        format!("{shortened}…")
    } else {
        shortened
    };
    format!("HTTP error {status}: {body}")
}

fn response_error_message(value: &Value, fallback: &str) -> String {
    value
        .pointer("/response/error/message")
        .and_then(Value::as_str)
        .or_else(|| value.pointer("/error/message").and_then(Value::as_str))
        .or_else(|| value.pointer("/response/error").and_then(Value::as_str))
        .or_else(|| value.get("message").and_then(Value::as_str))
        .unwrap_or(fallback)
        .to_string()
}

impl Completion for OpenAICodex {
    async fn completion(&self, messages: Vec<Message>) -> Result<CompletionResponse, Error> {
        use futures_util::StreamExt;

        let mut response = CompletionResponse::default();
        let stream = self.stream_response(messages).await?;
        futures_util::pin_mut!(stream);
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            response.content.push_str(&chunk.content);
            if let Some(thinking) = chunk.thinking {
                if let Some(existing) = response.thinking.as_mut() {
                    existing.push_str(&thinking);
                } else {
                    response.thinking = Some(thinking);
                }
            }
        }
        Ok(response)
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + 'static, Error> {
        self.stream_response(messages).await
    }
}
