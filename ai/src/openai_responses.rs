use futures_util::Stream;
use std::collections::HashSet;
use crate::{Completion, Message, CompletionResponse};
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Clone, Default)]
pub struct OpenAIResponses {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub models: HashSet<String>,
    pub allow_all_models: Option<bool>,
    pub model: String,
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
    pub content: Vec<ResponsesContent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponsesContent {
    pub r#type: String,
    pub text: String,
}

impl Completion for OpenAIResponses {
    async fn completion(&self, messages: Vec<Message>) -> Result<CompletionResponse, String> {
        let req = ResponsesRequest {
            model: self.model.clone(),
            input: messages,
            stream: Some(false),
        };

        let client = Client::new();
        let resp = client
            .post(format!("{}/responses", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let resp_json: ResponsesResponse = resp.json().await.map_err(|e| e.to_string())?;

        if let Some(output) = resp_json.output.first() {
            if let Some(content) = output.content.first() {
                Ok(CompletionResponse {
                    content: content.text.clone(),
                    thinking: None,
                })
            } else {
                Err("No content found".to_string())
            }
        } else {
            Err("No output found".to_string())
        }
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, String>> + Send, String> {
        let req = ResponsesRequest {
            model: self.model.clone(),
            input: messages,
            stream: Some(true),
        };

        let client = Client::new();
        let resp = client
            .post(format!("{}/responses", self.base_url))
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
                                match serde_json::from_str::<ResponsesResponse>(data) {
                                    Ok(response) => {
                                        if let Some(output) = response.output.first() {
                                            if let Some(content) = output.content.first() {
                                                return Some((Ok(CompletionResponse {
                                                    content: content.text.clone(),
                                                    thinking: None,
                                                }), (stream, buffer)));
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
                                        match serde_json::from_str::<ResponsesResponse>(data) {
                                            Ok(response) => {
                                                if let Some(output) = response.output.first() {
                                                    if let Some(content) = output.content.first() {
                                                        return Some((Ok(CompletionResponse {
                                                            content: content.text.clone(),
                                                            thinking: None,
                                                        }), (stream, buffer)));
                                                    }
                                                }
                                            }
                                            Err(e) => return Some((Err(e.to_string()), (stream, buffer))),
                                        }
                                    }
                                }
                            }
                            return None;
                        },
                    }
                }
            },
        ))
    }
}
