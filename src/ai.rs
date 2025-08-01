use async_openai::{
    Client,
    config::{Config, OpenAIConfig},
    error::OpenAIError,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs,
    },
};

pub struct Workers {
    client: Client<Box<dyn Config>>,
}

impl Workers {
    pub fn new(api_key: String, account_id: String) -> Self {
        let openai_config = OpenAIConfig::default()
            .with_api_base(format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/ai/v1",
                account_id
            ))
            .with_api_key(api_key);
        // You can use `std::sync::Arc` to wrap the config as well
        let config = Box::new(openai_config) as Box<dyn Config>;

        let client: Client<Box<dyn Config>> = Client::with_config(config);

        Workers { client }
    }

    pub async fn completion(
        &mut self,
        msgs: Vec<ChatCompletionRequestMessage>,
        model: &str,
    ) -> Result<String, OpenAIError> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(msgs)
            .build()?;

        let result = self.client.chat().create(request).await?;

        let content = result
            .choices
            .first()
            .unwrap()
            .message
            .content
            .clone()
            .unwrap();

        Ok(content)
    }

    pub async fn gemma3_12b(&mut self, prompt: String) -> Result<String, OpenAIError> {
        self.completion(vec![
        ChatCompletionRequestMessage::System(
        ChatCompletionRequestSystemMessage {
            content:ChatCompletionRequestSystemMessageContent::Text("You are the professional translator. You need translate input text by user's instruction. Don't print with markdown format.".to_string()),
            name:None,
        }),
        ChatCompletionRequestMessage::User(
        ChatCompletionRequestUserMessage {
            content:ChatCompletionRequestUserMessageContent::Text(prompt),
            name:None,
        }),
    ], "@cf/google/gemma-3-12b-it").await
    }

    pub async fn llama4_scout_17b_16e_instruct(
        &mut self,
        prompt: String,
    ) -> Result<String, OpenAIError> {
        self.completion(vec![
        ChatCompletionRequestMessage::System(
        ChatCompletionRequestSystemMessage {
            content:ChatCompletionRequestSystemMessageContent::Text("You are the professional translator. You need translate input text by user's instruction. Don't print with markdown format.".to_string()),
            name:None,
        }),
        ChatCompletionRequestMessage::User(
        ChatCompletionRequestUserMessage {
            content:ChatCompletionRequestUserMessageContent::Text(prompt),
            name:None,
        }),
    ], "@cf/meta/llama-4-scout-17b-16e-instruct").await
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::ai::Workers;

    #[derive(serde::Deserialize)]
    struct Auth {
        account_id: String,
        api_token: String,
    }

    #[tokio::test]
    pub async fn completion() {
        let auth_json = fs::read_to_string("src/.api.json").unwrap();
        let auth = serde_json::from_str::<Auth>(&auth_json).unwrap();
        let mut ai = Workers::new(auth.api_token, auth.account_id);
        println!(
            "{}",
            ai.gemma3_12b("生意気の意味は？".to_string()).await.unwrap()
        );
        println!(
            "{}",
            ai.llama4_scout_17b_16e_instruct("生意気の意味は？".to_string())
                .await
                .unwrap()
        );
    }
}
