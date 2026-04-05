use futures_util::Stream;
#[cfg(feature = "worker")]
use futures_util::StreamExt;

#[cfg(feature = "worker")]
use std::sync::Arc;

use crate::{Completion, CompletionResponse, Error, Message};
use std::pin::Pin;

#[cfg(feature = "worker")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "worker")]
use std::sync::OnceLock;

#[cfg(feature = "worker")]
static GLOBAL_AI: OnceLock<Arc<worker::Ai>> = OnceLock::new();

#[cfg(feature = "worker")]
pub fn set_global_ai(ai: worker::Ai) {
    let _ = GLOBAL_AI.set(Arc::new(ai));
}

#[cfg(feature = "worker")]
pub fn get_global_ai() -> Option<Arc<worker::Ai>> {
    GLOBAL_AI.get().cloned()
}

#[derive(Clone)]
pub struct WorkersAI {
    pub model: String,
    pub models: Vec<String>,
    pub reasoning_effort: Option<String>,
    #[cfg(feature = "worker")]
    pub binding: Option<Arc<worker::Ai>>,
}

impl Default for WorkersAI {
    fn default() -> Self {
        Self {
            model: String::new(),
            models: Vec::new(),
            reasoning_effort: Some("low".to_string()),
            #[cfg(feature = "worker")]
            binding: None,
        }
    }
}

#[cfg(feature = "worker")]
#[derive(Serialize)]
struct AiRequest {
    messages: Vec<Message>,
    stream: bool,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_template_kwargs: Option<ChatTemplateKwargs>,
}

#[cfg(feature = "worker")]
#[derive(Serialize)]
struct ChatTemplateKwargs {
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_thinking: Option<bool>,
}

#[cfg(feature = "worker")]
#[derive(Deserialize)]
struct AiResponse {
    response: Option<String>,
    #[serde(default)]
    choices: Vec<Choice>,
    thought: Option<String>,
    reasoning: Option<String>,
}

#[cfg(feature = "worker")]
#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[cfg(feature = "worker")]
#[derive(Deserialize)]
struct ChoiceMessage {
    content: String,
    reasoning_content: Option<String>,
    reasoning: Option<String>,
}

impl Completion for WorkersAI {
    async fn completion(&self, _messages: Vec<Message>) -> Result<CompletionResponse, Error> {
        #[cfg(feature = "worker")]
        {
            let binding = self.binding.clone().or_else(get_global_ai);
            if let Some(ai) = binding {
                let req = AiRequest {
                    messages: _messages,
                    stream: false,
                    max_tokens: 4096,
                    reasoning_effort: self.reasoning_effort.clone(),
                    chat_template_kwargs: Some(ChatTemplateKwargs {
                        enable_thinking: Some(true),
                    }),
                };
                let res: AiResponse = ai
                    .run(&self.model, req)
                    .await
                    .map_err(|e| Error::Internal(e.to_string()))?;

                let mut thinking = res.thought.or(res.reasoning);
                let mut content = res.response.unwrap_or_default();

                if let Some(choice) = res.choices.first() {
                    content = choice.message.content.clone();
                    if thinking.is_none() {
                        thinking = choice
                            .message
                            .reasoning_content
                            .clone()
                            .or(choice.message.reasoning.clone());
                    }
                }

                return Ok(CompletionResponse { content, thinking });
            }
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
        {
            let binding = self.binding.clone().or_else(get_global_ai);
            if let Some(ai) = binding {
                let req = AiRequest {
                    messages,
                    stream: true,
                    max_tokens: 4096,
                    reasoning_effort: self.reasoning_effort.clone(),
                    chat_template_kwargs: Some(ChatTemplateKwargs {
                        enable_thinking: Some(true),
                    }),
                };
                let stream = ai
                    .run_bytes(&self.model, req)
                    .await
                    .map_err(|e| Error::Internal(e.to_string()))?;

                let parsed = crate::sse::parse_stream(stream.map(|res| {
                    res.map(bytes::Bytes::from)
                        .map_err(|e| std::io::Error::other(e.to_string()))
                }));

                return Ok(Box::pin(parsed)
                    as Pin<Box<dyn Stream<Item = Result<CompletionResponse, Error>>>>);
            }
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
