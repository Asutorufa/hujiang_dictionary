use futures_util::Stream;

#[cfg(feature = "worker")]
use std::sync::Arc;

use crate::{Completion, CompletionResponse, Error, Message};

#[cfg(feature = "worker")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct WorkersAI {
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

        Err(Error::Internal("WorkersAI only supported with worker feature and binding".to_string()))
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

            use futures_util::StreamExt;
            let mapped_stream = byte_stream.map(|item| {
                item.map(bytes::Bytes::from)
            });

            return Ok(crate::sse::parse_stream(Box::pin(mapped_stream)));
        }

        #[cfg(feature = "worker")]
        return Err(Error::Internal("WorkersAI only supported with worker feature and binding".to_string()));

        #[cfg(not(feature = "worker"))]
        {
            // Dummy usage to suppress unused variable warning if messages is used only in feature
            let _ = messages;
            Err::<futures_util::stream::Empty<_>, _>(Error::Internal("WorkersAI only supported with worker feature and binding".to_string()))
        }
    }
}
