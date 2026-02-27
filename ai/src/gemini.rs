use futures_util::Stream;
use gemini::{Content, GenerateContentRequest, Part, Client};

use crate::{Completion, Message, CompletionResponse};

#[derive(Clone, Default)]
pub struct Gemini {
    pub api_key: String,
    pub model: String,
}

impl Gemini {
    pub async fn create_completion_stream(
        self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, String>> + Send, String> {
        let client = Client::new(self.api_key.clone(), self.model.clone());

        let contents = messages
            .into_iter()
            .map(|m| Content {
                role: if m.role == "assistant" {
                    "model".to_string()
                } else {
                    "user".to_string()
                },
                parts: vec![Part {
                    text: Some(m.content),
                    inline_data: None,
                }],
            })
            .collect();

        let req = GenerateContentRequest {
            contents,
            tools: None,
            safety_settings: None,
            system_instruction: None,
            generation_config: None,
        };

        let stream = client
            .stream_generate_content(&req)
            .await
            .map_err(|e| e.to_string())?;

        Ok(futures_util::stream::unfold(
            stream,
            |mut stream| async move {
                use futures_util::StreamExt;
                match stream.next().await {
                    Some(Ok(resp)) => {
                        if let Some(candidate) = resp.candidates.first() {
                            if let Some(part) = candidate.content.parts.first() {
                                if let Some(text) = &part.text {
                                    return Some((Ok(CompletionResponse {
                                        content: text.clone(),
                                        thinking: None,
                                    }), stream));
                                }
                            }
                        }
                        Some((Ok(CompletionResponse {
                            content: "".to_string(),
                            thinking: None,
                        }), stream))
                    }
                    Some(Err(e)) => Some((Err(e.to_string()), stream)),
                    None => None,
                }
            },
        ))
    }
}

impl Completion for Gemini {
    async fn completion(&self, messages: Vec<Message>) -> Result<CompletionResponse, String> {
        let client = Client::new(self.api_key.clone(), self.model.clone());

        let contents = messages
            .into_iter()
            .map(|m| Content {
                role: if m.role == "assistant" {
                    "model".to_string()
                } else {
                    "user".to_string()
                },
                parts: vec![Part {
                    text: Some(m.content),
                    inline_data: None,
                }],
            })
            .collect();

        let req = GenerateContentRequest {
            contents,
            tools: None,
            safety_settings: None,
            system_instruction: None,
            generation_config: None,
        };

        let resp = client
            .generate_content(&req)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(candidate) = resp.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                if let Some(text) = &part.text {
                    return Ok(CompletionResponse {
                        content: text.clone(),
                        thinking: None,
                    });
                }
            }
        }

        Err("No content found".to_string())
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, String>> + Send, String> {
        self.clone().create_completion_stream(messages).await
    }
}
