use futures_util::Stream;

use crate::{Completion, CompletionResponse, Error, Message, openai};

#[derive(Clone, Default)]
pub struct WorkersAI {
    pub account_id: String,
    pub api_token: String,
    pub model: String,
}

impl Completion for WorkersAI {
    async fn completion(&self, messages: Vec<Message>) -> Result<CompletionResponse, Error> {
        let client = openai::OpenAI {
            base_url: format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/ai/v1",
                self.account_id
            ),
            api_key: self.api_token.clone(),
            model: self.model.clone(),
            ..Default::default()
        };

        client.completion(messages).await
    }

    async fn completion_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<CompletionResponse, Error>> + Send, Error> {
        let client = openai::OpenAI {
            base_url: format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/ai/v1",
                self.account_id
            ),
            api_key: self.api_token.clone(),
            model: self.model.clone(),
            ..Default::default()
        };

        // client must live as long as the stream.
        // We use create_completion_stream which takes ownership of client.
        client.create_completion_stream(messages).await
    }
}
