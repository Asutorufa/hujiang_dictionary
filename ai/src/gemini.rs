use futures_util::Stream;
use gemini::{Client, Content, GenerateContentRequest, Part};

use crate::{Completion, CompletionResponse, Error, Message};

#[derive(Clone)]
pub struct Gemini {
    client: Client,
    pub models: Vec<String>,
    pub gemini_search: bool,
}

impl Gemini {
    pub fn new(api_key: String, model: String, models: Vec<String>) -> Self {
        Self {
            client: Client::new(api_key, model),
            models,
            gemini_search: false,
        }
    }

    pub fn new_vertex_ai(
        project_id: String,
        location: String,
        model: String,
        token: String,
        models: Vec<String>,
    ) -> Self {
        Self {
            client: Client::new_vertex_ai(project_id, location, model, token),
            models,
            gemini_search: false,
        }
    }

    pub fn set_model(&mut self, model: &str) {
        self.client.set_model(model);
    }

    pub async fn create_completion_stream(
        self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + 'static, Error> {
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

        let mut tools = None;
        if self.gemini_search {
            tools = Some(vec![gemini::Tool {
                google_search: Some(gemini::GoogleSearch::default()),
            }]);
        }

        let req = GenerateContentRequest {
            contents,
            tools,
            safety_settings: None,
            system_instruction: None,
            generation_config: Some(gemini::GenerationConfig {
                stop_sequences: None,
                response_mime_type: None,
                candidate_count: None,
                max_output_tokens: None,
                temperature: None,
                top_p: None,
                top_k: None,
                thinking_config: Some(gemini::ThinkingConfig {
                    include_thoughts: Some(true),
                    thinking_level: None,
                }),
            }),
        };

        // Use owned version or standard version if client handles it.
        // `gemini::Client::stream_generate_content_owned` consumes `self`.
        // So we can use `self.client` which we own.
        let stream = self
            .client
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
                            return Some((Ok(CompletionResponse { content, thinking }), stream));
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

        let mut tools = None;
        if self.gemini_search {
            tools = Some(vec![gemini::Tool {
                google_search: Some(gemini::GoogleSearch::default()),
            }]);
        }

        let req = GenerateContentRequest {
            contents,
            tools,
            safety_settings: None,
            system_instruction: None,
            generation_config: Some(gemini::GenerationConfig {
                stop_sequences: None,
                response_mime_type: None,
                candidate_count: None,
                max_output_tokens: None,
                temperature: None,
                top_p: None,
                top_k: None,
                thinking_config: Some(gemini::ThinkingConfig {
                    include_thoughts: Some(true),
                    thinking_level: None,
                }),
            }),
        };

        let resp = self
            .client
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
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + 'static, Error> {
        self.clone().create_completion_stream(messages).await
    }
}
