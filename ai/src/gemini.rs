use futures_util::Stream;
use gemini::{Client, Content, GenerateContentRequest, Part};

use crate::{Completion, CompletionResponse, Error, Message};

#[derive(Clone, Default)]
pub struct Gemini {
    pub api_key: String,
    pub model: String,
}

impl Gemini {
    pub async fn create_completion_stream(
        self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + Send, Error> {
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
                    thought: None,
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

        // Use owned version if I added it, or ensure req is not borrowed.
        // I added `stream_generate_content_owned`.
        let stream = client
            .stream_generate_content_owned(req)
            .await
            .map_err(|e| Error::Api(e.to_string()))?;

        Ok(futures_util::stream::unfold(
            stream,
            |mut stream| async move {
                use futures_util::StreamExt;
                match stream.next().await {
                    Some(Ok(resp)) => {
                        let mut content = String::new();
                        let mut thinking = None;

                        if let Some(candidate) = resp.candidates.first() {
                            for part in &candidate.content.parts {
                                if let Some(text) = &part.text {
                                    if part.thought.unwrap_or(false) {
                                        if thinking.is_none() {
                                            thinking = Some(String::new());
                                        }
                                        if let Some(t) = &mut thinking {
                                            t.push_str(text);
                                        }
                                    } else {
                                        content.push_str(text);
                                    }
                                }
                            }
                        }

                        if !content.is_empty() || thinking.is_some() {
                            return Some((
                                Ok(CompletionResponse { content, thinking }),
                                stream,
                            ));
                        }

                        Some((
                            Ok(CompletionResponse {
                                content: "".to_string(),
                                thinking: None,
                            }),
                            stream,
                        ))
                    }
                    Some(Err(e)) => Some((Err(Error::Api(e.to_string())), stream)),
                    None => None,
                }
            },
        ))
    }
}

impl Completion for Gemini {
    async fn completion(&self, messages: Vec<Message>) -> Result<CompletionResponse, Error> {
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
                    thought: None,
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
            .map_err(|e| Error::Api(e.to_string()))?;

        if let Some(candidate) = resp.candidates.first() {
            let mut content = String::new();
            let mut thinking = None;

            for part in &candidate.content.parts {
                if let Some(text) = &part.text {
                    if part.thought.unwrap_or(false) {
                        if thinking.is_none() {
                            thinking = Some(String::new());
                        }
                        if let Some(t) = &mut thinking {
                            t.push_str(text);
                        }
                    } else {
                        content.push_str(text);
                    }
                }
            }

            return Ok(CompletionResponse { content, thinking });
        }

        Err(Error::Api("No content found".to_string()))
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + Send, Error> {
        self.clone().create_completion_stream(messages).await
    }
}
