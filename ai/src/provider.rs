use futures_util::Stream;
use std::pin::Pin;

use crate::{
    Completion, CompletionResponse, Error, Message, claude, gemini, openai, openai_responses,
    workers,
};

#[derive(Clone)]
pub enum Provider {
    OpenAI(openai::OpenAI),
    WorkersAI(workers::WorkersAI),
    Gemini(gemini::Gemini),
    OpenAIResponses(openai_responses::OpenAIResponses),
    Claude(claude::Claude),
}

fn box_stream<S>(stream: S) -> Pin<Box<dyn Stream<Item = Result<CompletionResponse, Error>>>>
where
    S: Stream<Item = Result<CompletionResponse, Error>> + 'static,
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
            Provider::Claude(provider) => provider.completion(messages).await,
        }
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + 'static, Error> {
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
            Provider::Claude(provider) => {
                Ok(box_stream(provider.completion_stream(messages).await?))
            }
        }
    }
}

impl Provider {
    pub fn models(&self) -> Vec<String> {
        match self {
            Provider::OpenAI(provider) => provider.models.iter().cloned().collect(),
            Provider::WorkersAI(provider) => provider.models.clone(),
            Provider::Gemini(provider) => provider.models.clone(),
            Provider::OpenAIResponses(_) => vec![],
            Provider::Claude(provider) => provider.models.iter().cloned().collect(),
        }
    }

    pub fn set_model(&mut self, model: &str) {
        match self {
            Provider::OpenAI(provider) => provider.model = model.to_string(),
            Provider::WorkersAI(provider) => provider.model = model.to_string(),
            Provider::Gemini(provider) => provider.set_model(model),
            Provider::OpenAIResponses(provider) => provider.model = model.to_string(),
            Provider::Claude(provider) => provider.model = model.to_string(),
        }
    }

    pub fn set_gemini_search(&mut self, enabled: bool) {
        if let Provider::Gemini(provider) = self {
            provider.gemini_search = enabled;
        }
    }

    pub fn features(&self) -> &str {
        match self {
            Provider::OpenAI(provider) => provider.features.as_deref().unwrap_or("{}"),
            Provider::Gemini(provider) => provider.features.as_deref().unwrap_or("{}"),
            Provider::WorkersAI(_) => "{}",
            Provider::OpenAIResponses(_) => "{}",
            Provider::Claude(provider) => provider.features.as_deref().unwrap_or("{}"),
        }
    }

    pub fn set_features(&mut self, features: Option<String>) {
        match self {
            Provider::OpenAI(provider) => provider.features = features,
            Provider::Gemini(provider) => provider.features = features,
            Provider::WorkersAI(_) => {}
            Provider::OpenAIResponses(_) => {}
            Provider::Claude(provider) => provider.features = features,
        }
    }
}
