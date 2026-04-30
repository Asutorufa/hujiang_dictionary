use crate::{Completion, CompletionResponse, Error, Message};
use futures_util::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

#[derive(Clone)]
pub struct OpenAIResponses {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub models: HashSet<String>,
    pub allow_all_models: Option<bool>,
    pub model: String,
    pub client: Client,
}

impl Default for OpenAIResponses {
    fn default() -> Self {
        Self {
            name: Default::default(),
            base_url: DEFAULT_OPENAI_BASE_URL.to_string(),
            api_key: Default::default(),
            models: Default::default(),
            allow_all_models: Default::default(),
            model: Default::default(),
            client: Client::new(),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct ResponsesRequest {
    pub model: String,
    pub input: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponsesResponse {
    pub output: Vec<ResponsesOutput>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponsesOutput {
    #[serde(default)]
    pub content: Vec<ResponsesContent>,
    #[serde(default)]
    pub summary: Vec<ResponsesContent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponsesContent {
    pub r#type: String,
    #[serde(default)]
    pub text: String,
}

enum ResponsesStreamItem {
    Chunk(CompletionResponse),
    Stop,
}

impl OpenAIResponses {
    fn responses_url(&self) -> String {
        let base_url = self.base_url.trim().trim_end_matches('/');
        let base_url = if base_url.is_empty() {
            DEFAULT_OPENAI_BASE_URL
        } else {
            base_url
        };

        if base_url.ends_with("/responses") {
            base_url.to_string()
        } else {
            format!("{}/responses", base_url)
        }
    }

    fn response_to_completion(response: ResponsesResponse) -> Result<CompletionResponse, Error> {
        let mut content = String::new();
        let mut thinking = None;

        for output in response.output {
            for c in output.content {
                match c.r#type.as_str() {
                    "output_text" | "text" => content.push_str(&c.text),
                    "reasoning" | "summary_text" => {
                        Self::append_optional(&mut thinking, &c.text);
                    }
                    _ => {}
                }
            }

            for c in output.summary {
                if c.r#type == "summary_text" || c.r#type == "reasoning" {
                    Self::append_optional(&mut thinking, &c.text);
                }
            }
        }

        Ok(CompletionResponse { content, thinking })
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

    fn parse_stream_data(data: &str) -> Result<Option<ResponsesStreamItem>, Error> {
        if data == "[DONE]" {
            return Ok(Some(ResponsesStreamItem::Stop));
        }

        let value: Value = serde_json::from_str(data)?;

        match value.get("type").and_then(Value::as_str) {
            Some("response.output_text.delta") => {
                let content = value
                    .get("delta")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();

                if content.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(ResponsesStreamItem::Chunk(CompletionResponse {
                        content,
                        thinking: None,
                    })))
                }
            }
            Some("response.reasoning_summary_text.delta") => {
                let thinking = value
                    .get("delta")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();

                if thinking.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(ResponsesStreamItem::Chunk(CompletionResponse {
                        content: String::new(),
                        thinking: Some(thinking),
                    })))
                }
            }
            Some("response.completed") => Ok(Some(ResponsesStreamItem::Stop)),
            Some("response.failed" | "response.incomplete") => {
                let error_message = value
                    .pointer("/response/error/message")
                    .and_then(Value::as_str)
                    .or_else(|| value.pointer("/response/error").and_then(Value::as_str))
                    .or_else(|| value.get("message").and_then(Value::as_str))
                    .unwrap_or("OpenAI Responses stream failed");

                Err(Error::Api(error_message.to_string()))
            }
            Some("error") => {
                let error_message = value
                    .pointer("/error/message")
                    .and_then(Value::as_str)
                    .or_else(|| value.get("message").and_then(Value::as_str))
                    .unwrap_or("OpenAI Responses stream returned an error");

                Err(Error::Api(error_message.to_string()))
            }
            _ => Ok(None),
        }
    }
}

impl Completion for OpenAIResponses {
    async fn completion(&self, messages: Vec<Message>) -> Result<CompletionResponse, Error> {
        let req = ResponsesRequest {
            model: self.model.clone(),
            input: messages,
            stream: Some(false),
        };

        let resp = self
            .client
            .post(self.responses_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let err_msg = resp.text().await.unwrap_or_default();
            return Err(Error::Api(format!("HTTP error {}: {}", status, err_msg)));
        }

        let resp_json: ResponsesResponse = resp.json().await?;

        if resp_json.output.is_empty() {
            Err(Error::Api("No output found".to_string()))
        } else {
            Self::response_to_completion(resp_json)
        }
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + 'static, Error> {
        let req = ResponsesRequest {
            model: self.model.clone(),
            input: messages,
            stream: Some(true),
        };

        let resp = self
            .client
            .post(self.responses_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
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
                                    Ok(Some(ResponsesStreamItem::Chunk(chunk))) => {
                                        return Some((Ok(chunk), (stream, buffer, event_data)));
                                    }
                                    Ok(Some(ResponsesStreamItem::Stop)) => return None,
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
                                    Ok(Some(ResponsesStreamItem::Chunk(chunk))) => {
                                        return Some((Ok(chunk), (stream, buffer, event_data)));
                                    }
                                    Ok(Some(ResponsesStreamItem::Stop)) | Ok(None) => return None,
                                    Err(e) => {
                                        return Some((Err(e), (stream, buffer, event_data)));
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
