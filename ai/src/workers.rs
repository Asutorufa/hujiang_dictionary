use futures_util::Stream;

#[cfg(feature = "worker")]
use std::sync::Arc;

use crate::{Completion, CompletionResponse, Error, Message};

#[cfg(not(feature = "worker"))]
use crate::openai;

#[cfg(feature = "worker")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct WorkersAI {
    pub account_id: String,
    pub api_token: String,
    pub model: String,
    #[cfg(feature = "worker")]
    pub binding: Option<Arc<worker::Ai>>,
}

#[cfg(feature = "worker")]
#[derive(Serialize)]
struct AiRequest {
    messages: Vec<Message>,
    stream: bool,
}

#[cfg(feature = "worker")]
#[derive(Deserialize)]
struct AiResponse {
    response: String,
}

#[cfg(feature = "worker")]
#[derive(Deserialize)]
struct StreamResponse {
    response: Option<String>,
}

impl Completion for WorkersAI {
    async fn completion(&self, messages: Vec<Message>) -> Result<CompletionResponse, Error> {
        #[cfg(feature = "worker")]
        if let Some(ai) = &self.binding {
            let req = AiRequest {
                messages,
                stream: false,
            };
            let res: AiResponse = ai
                .run(&self.model, req)
                .await
                .map_err(|e| Error::Internal(e.to_string()))?;

            return Ok(CompletionResponse {
                content: res.response,
                thinking: None,
            });
        }

        #[cfg(not(feature = "worker"))]
        {
            let client = openai::OpenAI {
                base_url: format!(
                    "https://api.cloudflare.com/client/v4/accounts/{}/ai/v1",
                    self.account_id
                ),
                api_key: self.api_token.clone(),
                model: self.model.clone(),
                ..Default::default()
            };

            return client.completion(messages).await;
        }

        #[cfg(feature = "worker")]
        Err(Error::Internal("No binding available and REST fallback disabled for worker feature".to_string()))
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + Send, Error> {
        #[cfg(feature = "worker")]
        if let Some(ai) = &self.binding {
            let req = AiRequest {
                messages,
                stream: true,
            };

            let stream_result: worker::Stream = ai
                .run(&self.model, req)
                .await
                .map_err(|e| Error::Internal(e.to_string()))?;

            let byte_stream = stream_result.stream();

            // Map worker::Error to something parse_stream accepts (or update parse_stream to be generic)
            // Updated parse_stream to be generic over Error.
            use futures_util::StreamExt;
            let mapped_stream = byte_stream.map(|item| {
                item.map(bytes::Bytes::from)
            });

            // Ensure Unpin. worker::Stream's stream likely is Unpin or we box/pin it.
            // map returns Map which is Unpin if inner is Unpin.
            // If compilation fails on Unpin, we box it.

            return Ok(crate::sse::parse_stream(Box::pin(mapped_stream)));
        }

        #[cfg(not(feature = "worker"))]
        {
            let client = openai::OpenAI {
                base_url: format!(
                    "https://api.cloudflare.com/client/v4/accounts/{}/ai/v1",
                    self.account_id
                ),
                api_key: self.api_token.clone(),
                model: self.model.clone(),
                ..Default::default()
            };

            return client.create_completion_stream(messages).await;
        }

        #[cfg(feature = "worker")]
        Err(Error::Internal("No binding available and REST fallback disabled for worker feature".to_string()))
    }
}
