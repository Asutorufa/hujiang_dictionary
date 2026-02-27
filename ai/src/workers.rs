use futures_util::Stream;

use crate::{Completion, CompletionResponse, Message, openai};

#[derive(Clone, Default)]
pub struct WorkersAI {
    pub account_id: String,
    pub api_token: String,
    pub model: String,
}

impl Completion for WorkersAI {
    async fn completion(&self, messages: Vec<Message>) -> Result<CompletionResponse, String> {
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
    ) -> Result<impl Stream<Item = Result<CompletionResponse, String>> + Send, String> {
        let client = openai::OpenAI {
            base_url: format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/ai/v1",
                self.account_id
            ),
            api_key: self.api_token.clone(),
            model: self.model.clone(),
            ..Default::default()
        };

        // We need to return a stream that owns the client data, because `client` is dropped at end of function.
        // `OpenAI::completion_stream` returns `impl Stream` that borrows `self`.
        // To fix this, we can wrap the stream in a way that moves ownership, OR better:
        // Create a dedicated `OpenAIClient` struct that implements `Completion` and owns its data, which `OpenAI` struct already does.
        // The problem is `completion_stream` takes `&self`.

        // We can manually implement the stream here or use a helper that takes ownership.
        // Since `OpenAI` logic is in `openai.rs`, let's make a static method or a method that takes ownership?
        // No, trait defines `&self`.

        // Solution: Create a stream that owns the `OpenAI` client.
        // But `client.completion_stream` returns a stream that borrows `client`.
        // If we can't change `OpenAI` to not borrow `self` (it needs base_url etc), we must ensure `client` lives as long as stream.
        // But `client` is local.

        // Let's modify `OpenAI::completion_stream` to return a stream that owns the necessary data?
        // `OpenAI` struct fields are Strings.
        // If we move `client` into the async block of `unfold`, it might work?
        // But `client.completion_stream` is async and returns `impl Stream`.

        // Hack/Fix:
        // Create the stream using `reqwest` directly here, duplicating code? No.
        // Move the `OpenAI` client creation into a helper function in `openai.rs` that returns a Stream and takes ownership of config.

        openai::OpenAI::create_completion_stream(client, messages).await
    }
}
