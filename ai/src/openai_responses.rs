use crate::{Completion, CompletionResponse, Error, Message};
use futures_util::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
            base_url: Default::default(),
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
    pub content: Vec<ResponsesContent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponsesContent {
    pub r#type: String,
    pub text: String,
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
            .post(format!("{}/responses", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await?;

        let resp_json: ResponsesResponse = resp.json().await?;

        let mut content = String::new();
        let mut thinking = None;

        if let Some(output) = resp_json.output.first() {
            for c in &output.content {
                if c.r#type == "reasoning" {
                    if thinking.is_none() {
                        thinking = Some(String::new());
                    }
                    if let Some(t) = &mut thinking {
                        t.push_str(&c.text);
                    }
                } else {
                    content.push_str(&c.text);
                }
            }
            Ok(CompletionResponse { content, thinking })
        } else {
            Err(Error::Api("No output found".to_string()))
        }
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + Send, Error> {
        let req = ResponsesRequest {
            model: self.model.clone(),
            input: messages,
            stream: Some(true),
        };

        let resp = self
            .client
            .post(format!("{}/responses", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await?;

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
                                            let mut content = String::new();
                                            let mut thinking = None;

                                            for c in &output.content {
                                                if c.r#type == "reasoning" {
                                                    if thinking.is_none() {
                                                        thinking = Some(String::new());
                                                    }
                                                    if let Some(t) = &mut thinking {
                                                        t.push_str(&c.text);
                                                    }
                                                } else {
                                                    content.push_str(&c.text);
                                                }
                                            }

                                            if !content.is_empty() || thinking.is_some() {
                                                return Some((
                                                    Ok(CompletionResponse { content, thinking }),
                                                    (stream, buffer),
                                                ));
                                            }
                                        }
                                    }
                                    Err(e) => return Some((Err(Error::from(e)), (stream, buffer))),
                                }
                            }
                        }
                        continue;
                    }

                    match stream.next().await {
                        Some(Ok(chunk)) => {
                            buffer.push_str(&String::from_utf8_lossy(&chunk));
                        }
                        Some(Err(e)) => return Some((Err(Error::from(e)), (stream, buffer))),
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
                                                    let mut content = String::new();
                                                    let mut thinking = None;

                                                    for c in &output.content {
                                                        if c.r#type == "reasoning" {
                                                            if thinking.is_none() {
                                                                thinking = Some(String::new());
                                                            }
                                                            if let Some(t) = &mut thinking {
                                                                t.push_str(&c.text);
                                                            }
                                                        } else {
                                                            content.push_str(&c.text);
                                                        }
                                                    }

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
                                                return Some((
                                                    Err(Error::from(e)),
                                                    (stream, buffer),
                                                ));
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
