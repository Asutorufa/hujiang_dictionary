use futures_util::Stream;
use std::pin::Pin;

use crate::{
    Completion, CompletionResponse, Error, Message, gemini, openai, openai_responses, workers,
};

#[derive(Clone)]
pub enum Provider {
    OpenAI(openai::OpenAI),
    WorkersAI(workers::WorkersAI),
    Gemini(gemini::Gemini),
    OpenAIResponses(openai_responses::OpenAIResponses),
}

fn box_stream<S>(stream: S) -> Pin<Box<dyn Stream<Item = Result<CompletionResponse, Error>> + Send>>
where
    S: Stream<Item = Result<CompletionResponse, Error>> + Send + 'static,
{
    Box::pin(stream)
}

impl Completion for Provider {
    async fn completion(&self, messages: Vec<Message>) -> Result<CompletionResponse, Error> {
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
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + Send + 'static, Error> {
        match self {
            Provider::OpenAI(provider) => {
                Ok(box_stream(provider.completion_stream(messages).await?))
            }
            Provider::WorkersAI(provider) => {
                Ok(box_stream(provider.completion_stream(messages).await?))
            }
            Provider::Gemini(provider) => {
                Ok(box_stream(provider.completion_stream(messages).await?))
            }
            Provider::OpenAIResponses(provider) => {
                Ok(box_stream(provider.completion_stream(messages).await?))
            }
        }
    }
}
