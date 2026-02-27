use futures_util::Stream;
use std::pin::Pin;

use crate::{Completion, Message, openai, workers, gemini};

#[derive(Clone)]
pub enum Provider {
    OpenAI(openai::OpenAI),
    WorkersAI(workers::WorkersAI),
    Gemini(gemini::Gemini),
}

impl Completion for Provider {
    async fn completion(&self, messages: Vec<Message>) -> Result<String, String> {
        match self {
            Provider::OpenAI(provider) => provider.completion(messages).await,
            Provider::WorkersAI(provider) => provider.completion(messages).await,
            Provider::Gemini(provider) => provider.completion(messages).await,
        }
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<String, String>> + Send, String> {
        match self {
            Provider::OpenAI(provider) => {
                let stream = provider.completion_stream(messages).await?;
                Ok(Box::pin(stream) as Pin<Box<dyn Stream<Item = Result<String, String>> + Send>>)
            }
            Provider::WorkersAI(provider) => {
                let stream = provider.completion_stream(messages).await?;
                Ok(Box::pin(stream) as Pin<Box<dyn Stream<Item = Result<String, String>> + Send>>)
            }
            Provider::Gemini(provider) => {
                let stream = provider.completion_stream(messages).await?;
                Ok(Box::pin(stream) as Pin<Box<dyn Stream<Item = Result<String, String>> + Send>>)
            }
        }
    }
}
