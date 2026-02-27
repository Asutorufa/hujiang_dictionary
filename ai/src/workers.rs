use futures_util::Stream;

use crate::{Completion, Message, openai};

#[derive(Clone, Default)]
pub struct WorkersAI {
    pub account_id: String,
    pub api_token: String,
    pub model: String,
}

impl Completion for WorkersAI {
    async fn completion(&self, messages: Vec<Message>) -> Result<String, String> {
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
    ) -> Result<impl Stream<Item = Result<String, String>> + Send, String> {
        let client = openai::OpenAI {
            base_url: format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/ai/v1",
                self.account_id
            ),
            api_key: self.api_token.clone(),
            model: self.model.clone(),
            ..Default::default()
        };

        client.completion_stream(messages).await
    }
}
