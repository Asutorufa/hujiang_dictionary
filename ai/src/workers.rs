use futures_util::Stream;
#[cfg(feature = "worker")]
use futures_util::StreamExt;

#[cfg(feature = "worker")]
use std::sync::Arc;

use crate::{Completion, CompletionResponse, Error, Message};
use std::pin::Pin;

#[cfg(feature = "worker")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct WorkersAI {
    pub model: String,
    pub models: Vec<String>,
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

impl Completion for WorkersAI {
    async fn completion(&self, _messages: Vec<Message>) -> Result<CompletionResponse, Error> {
        #[cfg(feature = "worker")]
        if let Some(ai) = &self.binding {
            let req = AiRequest {
                messages: _messages,
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

        Err(Error::Internal(
            "WorkersAI only supported with worker feature and binding".to_string(),
        ))
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + 'static, Error> {
        #[cfg(feature = "worker")]
        if let Some(ai) = &self.binding {
            let req = AiRequest {
                messages,
                stream: true,
            };
            let stream = ai
                .run_bytes(&self.model, req)
                .await
                .map_err(|e| Error::Internal(e.to_string()))?;

            let parsed = crate::sse::parse_stream(stream.map(|res| {
                res.map(bytes::Bytes::from)
                    .map_err(|e| std::io::Error::other(e.to_string()))
            }));

            return Ok(
                Box::pin(parsed) as Pin<Box<dyn Stream<Item = Result<CompletionResponse, Error>>>>
            );
        }

        #[cfg(not(feature = "worker"))]
        let _ = messages;

        let err_stream = futures_util::stream::empty();
        if false {
            return Ok(Box::pin(err_stream)
                as Pin<Box<dyn Stream<Item = Result<CompletionResponse, Error>>>>);
        }

        Err(Error::Internal(
            "WorkersAI only supported with worker feature and binding".to_string(),
        ))
    }
}
