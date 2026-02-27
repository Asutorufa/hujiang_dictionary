use std::collections::HashSet;

use crate::{Completion, CompletionResponse, Message};
use futures_util::Stream;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct OpenAI {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub models: HashSet<String>,
    pub allow_all_models: Option<bool>,
    pub model: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct APICompletionResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    pub message: APICompletionMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct APICompletionMessage {
    pub content: String,
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StreamCompletionResponse {
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StreamChoice {
    pub delta: StreamMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct StreamMessage {
    pub content: Option<String>,
    pub reasoning_content: Option<String>,
}

impl OpenAI {
    pub async fn create_completion_stream(
        self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, String>> + Send, String> {
        let req = CompletionRequest {
            model: self.model.clone(),
            messages,
            stream: Some(true),
        };

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let stream = resp.bytes_stream();

        Ok(futures_util::stream::unfold(
            (stream, String::new()),
            |(mut stream, mut buffer)| async move {
                loop {
                    use futures_util::StreamExt;

                    if let Some(pos) = buffer.find("\n\n") {
                        let message = buffer[..pos].to_string();
                        buffer.drain(..pos + 2);

                        if let Some(data) = message.strip_prefix("data: ") {
                            let data = data.trim();
                            if data == "[DONE]" {
                                return None;
                            }
                            if !data.is_empty() {
                                match serde_json::from_str::<StreamCompletionResponse>(data) {
                                    Ok(response) => {
                                        if let Some(choice) = response.choices.first() {
                                            let content =
                                                choice.delta.content.clone().unwrap_or_default();
                                            let thinking = choice.delta.reasoning_content.clone();
                                            if !content.is_empty() || thinking.is_some() {
                                                return Some((
                                                    Ok(CompletionResponse { content, thinking }),
                                                    (stream, buffer),
                                                ));
                                            }
                                        }
                                    }
                                    Err(e) => return Some((Err(e.to_string()), (stream, buffer))),
                                }
                            }
                        }
                        continue;
                    }

                    match stream.next().await {
                        Some(Ok(chunk)) => {
                            buffer.push_str(&String::from_utf8_lossy(&chunk));
                        }
                        Some(Err(e)) => return Some((Err(e.to_string()), (stream, buffer))),
                        None => {
                            if !buffer.is_empty() {
                                let message = buffer.clone();
                                buffer.clear();
                                if let Some(data) = message.strip_prefix("data: ") {
                                    let data = data.trim();
                                    if !data.is_empty() && data != "[DONE]" {
                                        match serde_json::from_str::<StreamCompletionResponse>(data)
                                        {
                                            Ok(response) => {
                                                if let Some(choice) = response.choices.first() {
                                                    let content = choice
                                                        .delta
                                                        .content
                                                        .clone()
                                                        .unwrap_or_default();
                                                    let thinking =
                                                        choice.delta.reasoning_content.clone();
                                                    if !content.is_empty() || thinking.is_some() {
                                                        return Some((
                                                            Ok(CompletionResponse {
                                                                content,
                                                                thinking,
                                                            }),
                                                            (stream, buffer),
                                                        ));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                return Some((Err(e.to_string()), (stream, buffer)));
                                            }
                                        }
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

impl Completion for OpenAI {
    async fn completion(&self, messages: Vec<Message>) -> Result<CompletionResponse, String> {
        let req = CompletionRequest {
            model: self.model.clone(),
            messages,
            stream: Some(false),
        };

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let resp_json: APICompletionResponse = resp.json().await.map_err(|e| e.to_string())?;

        if let Some(choice) = resp_json.choices.first() {
            Ok(CompletionResponse {
                content: choice.message.content.clone(),
                thinking: choice.message.reasoning_content.clone(),
            })
        } else {
            Err("No choices found".to_string())
        }
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, String>> + Send, String> {
        self.clone().create_completion_stream(messages).await
    }
}
