use futures_util::Stream;
use gemini::{Content, GenerateContentRequest, Part, Client};

use crate::{Completion, Message, CompletionResponse};

#[derive(Clone, Default)]
pub struct Gemini {
    pub api_key: String,
    pub model: String,
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

        // Note: gemini::Client::stream_generate_content returns a stream that might depend on `client` or `req`.
        // The `gemini` crate implementation in `gemini/src/client.rs` shows `stream_generate_content` returns `SseStream<S>` which owns `inner` stream and `buffer`.
        // However, `reqwest::Client` (inside `gemini::Client`) handles connection.
        // If `stream_generate_content` is async and returns `Result<impl Stream...>`, and if the `impl Stream` is `'static` (i.e. doesn't borrow from local `client` or `req`), we are fine.
        // Let's assume `gemini` crate is well-behaved for now, but if `client` is dropped, the request might be cancelled if the stream relied on it?
        // Actually `reqwest::Client` is reference counted internally, so cloning it is fine.
        // But `gemini::Client` owns `reqwest::Client`.
        // The `gemini::Client::stream_generate_content` calls `self.http.post(...)` and gets a response, then `resp.bytes_stream()`.
        // The `bytes_stream()` consumes `resp`, which is good.
        // So the returned stream should be independent of `gemini::Client` lifetime.

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
