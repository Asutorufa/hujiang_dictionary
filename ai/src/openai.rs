use std::collections::HashSet;

use crate::{Completion, CompletionResponse, Error, Message};
use futures_util::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

#[derive(Clone)]
pub struct OpenAI {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub models: HashSet<String>,
    pub allow_all_models: Option<bool>,
    pub model: String,
    pub client: Client,
    pub features: Option<String>,
}

impl Default for OpenAI {
    fn default() -> Self {
        Self {
            name: Default::default(),
            base_url: Default::default(),
            api_key: Default::default(),
            models: Default::default(),
            allow_all_models: Default::default(),
            model: Default::default(),
            client: Client::new(),
            features: None,
        }
    }
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

#[derive(Debug, Deserialize)]
struct APIModelsResponse {
    #[serde(default)]
    pub data: Vec<APIModel>,
}

#[derive(Debug, Deserialize)]
struct APIModel {
    pub id: String,
}

pub(crate) fn models_url(base_url: &str) -> String {
    let base_url = base_url.trim().trim_end_matches('/');
    let base_url = if base_url.is_empty() {
        DEFAULT_OPENAI_BASE_URL
    } else {
        base_url
    };
    let base_url = base_url.strip_suffix("/models").unwrap_or(base_url);
    format!("{base_url}/models")
}

pub(crate) async fn fetch_models(
    client: &Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<String>, Error> {
    let mut request = client.get(models_url(base_url));
    if !api_key.trim().is_empty() {
        request = request.bearer_auth(api_key);
    }

    let response = request.send().await?;
    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(Error::Api(format!("HTTP error {}: {}", status, error_body)));
    }

    let response: APIModelsResponse = response.json().await?;
    Ok(response
        .data
        .into_iter()
        .map(|model| model.id.trim().to_string())
        .filter(|model| !model.is_empty())
        .collect())
}

impl OpenAI {
    fn completions_base_url(&self) -> String {
        let base_url = self.base_url.trim().trim_end_matches('/');
        if base_url.is_empty() {
            DEFAULT_OPENAI_BASE_URL.to_string()
        } else {
            base_url.to_string()
        }
    }

    pub async fn list_models(&self) -> Result<Vec<String>, Error> {
        fetch_models(&self.client, &self.base_url, &self.api_key).await
    }

    pub async fn create_completion_stream(
        self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + 'static, Error> {
        let req = CompletionRequest {
            model: self.model.clone(),
            messages,
            stream: Some(true),
        };

        let resp = self
            .client
            .post(format!("{}/chat/completions", self.completions_base_url()))
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

        let stream = crate::http::bytes_stream(resp);

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
                            if data == "[DONE]" {
                                return None;
                            }
                            if let Some(choice) = Some(data)
                                .filter(|d| !d.is_empty())
                                .and_then(|d| {
                                    serde_json::from_str::<StreamCompletionResponse>(d).ok()
                                })
                                .and_then(|r| r.choices.into_iter().next())
                            {
                                let content = choice.delta.content.clone().unwrap_or_default();
                                let thinking = choice.delta.reasoning_content.clone();
                                if !content.is_empty() || thinking.is_some() {
                                    return Some((
                                        Ok(CompletionResponse { content, thinking }),
                                        (stream, buffer),
                                    ));
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
                                let line = buffer.clone();
                                buffer.clear();
                                if let Some(data) = line.strip_prefix("data:") {
                                    let data = data.trim();
                                    if let Some(choice) = Some(data)
                                        .filter(|d| !d.is_empty() && *d != "[DONE]")
                                        .and_then(|d| {
                                            serde_json::from_str::<StreamCompletionResponse>(d).ok()
                                        })
                                        .and_then(|r| r.choices.into_iter().next())
                                    {
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
    async fn completion(&self, messages: Vec<Message>) -> Result<CompletionResponse, Error> {
        let req = CompletionRequest {
            model: self.model.clone(),
            messages,
            stream: Some(false),
        };

        let resp = self
            .client
            .post(format!("{}/chat/completions", self.completions_base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await?;

        let resp_json: APICompletionResponse = resp.json().await?;

        if let Some(choice) = resp_json.choices.first() {
            Ok(CompletionResponse {
                content: choice.message.content.clone(),
                thinking: choice.message.reasoning_content.clone(),
            })
        } else {
            Err(Error::Api("No choices found".to_string()))
        }
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + 'static, Error> {
        self.clone().create_completion_stream(messages).await
    }
}
