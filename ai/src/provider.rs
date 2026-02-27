use futures_util::Stream;
use std::pin::Pin;

use crate::{Completion, CompletionResponse, Message, gemini, openai, openai_responses, workers};

#[derive(Clone)]
pub enum Provider {
    OpenAI(openai::OpenAI),
    WorkersAI(workers::WorkersAI),
    Gemini(gemini::Gemini),
    OpenAIResponses(openai_responses::OpenAIResponses),
}

impl Completion for Provider {
    async fn completion(&self, messages: Vec<Message>) -> Result<CompletionResponse, String> {
        match self {
            Provider::OpenAI(provider) => provider.completion(messages).await,
            Provider::WorkersAI(provider) => provider.completion(messages).await,
            Provider::Gemini(provider) => provider.completion(messages).await,
            Provider::OpenAIResponses(provider) => provider.completion(messages).await,
        }
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, String>> + Send, String> {
        match self {
            Provider::OpenAI(provider) => {
                let stream = provider.completion_stream(messages).await?;
                Ok(Box::pin(stream)
                    as Pin<
                        Box<dyn Stream<Item = Result<CompletionResponse, String>> + Send>,
                    >)
            }
            Provider::WorkersAI(provider) => {
                let stream = provider.completion_stream(messages).await?;
                Ok(Box::pin(stream)
                    as Pin<
                        Box<dyn Stream<Item = Result<CompletionResponse, String>> + Send>,
                    >)
            }
            Provider::Gemini(provider) => {
                let stream = provider.completion_stream(messages).await?;
                Ok(Box::pin(stream)
                    as Pin<
                        Box<dyn Stream<Item = Result<CompletionResponse, String>> + Send>,
                    >)
            }
            Provider::OpenAIResponses(provider) => {
                let stream = provider.completion_stream(messages).await?;
                Ok(Box::pin(stream)
                    as Pin<
                        Box<dyn Stream<Item = Result<CompletionResponse, String>> + Send>,
                    >)
            }
        }
    }
}
