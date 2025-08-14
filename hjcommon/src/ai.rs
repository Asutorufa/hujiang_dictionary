use async_openai::{
    Client,
    config::OpenAIConfig,
    error::OpenAIError,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs,
    },
};
use hjdef::ai::{AI, Error};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Workers {
    client: Client<OpenAIConfig>,
    account_id: String,
    api_token: String,
}

static SYSTEM_MSG: &str = r#"
You are a professional translator.
Translate the input text according to the user's instructions and return the result in the user’s original language (unless the user requests otherwise).
The total output must not exceed 4096 characters, including spaces and line breaks.
If the translated content is approaching the limit, prioritize preserving core meaning and compress the expression when necessary. Paraphrase or summarize if required.
Do not output in Markdown format.
Strictly follow the character limit to prevent truncation.
"#;

impl Workers {
    pub fn new(account_id: &str, api_key: &str) -> Self {
        let openai_config = OpenAIConfig::default()
            .with_api_base(format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/ai/v1",
                account_id
            ))
            .with_api_key(api_key);

        let client: Client<OpenAIConfig> = Client::with_config(openai_config);

        Workers {
            client,
            account_id: account_id.to_string(),
            api_token: api_key.to_string(),
        }
    }

    pub async fn completion(
        &self,
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
}

impl AI for Workers {
    async fn gemma3_12b(&self, prompt: String) -> Result<String, Error> {
        match self
            .completion(
                vec![
                    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                        content: ChatCompletionRequestSystemMessageContent::Text(
                            SYSTEM_MSG.to_string(),
                        ),
                        name: None,
                    }),
                    ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Text(prompt),
                        name: None,
                    }),
                ],
                "@cf/google/gemma-3-12b-it",
            )
            .await
        {
            Ok(content) => Ok(content.clone()),
            Err(e) => Err(Error::from(e.to_string())),
        }
    }

    async fn llama4_scout_17b_16e_instruct(&self, prompt: String) -> Result<String, Error> {
        self.completion(
            vec![
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(
                        SYSTEM_MSG.to_string(),
                    ),
                    name: None,
                }),
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(prompt),
                    name: None,
                }),
            ],
            "@cf/meta/llama-4-scout-17b-16e-instruct",
        )
        .await
        .map_err(|e| Error::from(e.to_string()))
    }

    async fn m2m100_1_2b(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String, Error> {
        #[derive(Debug, Serialize, Deserialize)]
        struct Request {
            text: String,
            source_lang: String,
            target_lang: String,
        }

        let body = serde_json::to_string(&Request {
            source_lang: if source_lang.is_empty() {
                "english".to_string()
            } else {
                source_lang.to_string()
            },
            target_lang: target_lang.to_string(),
            text: text.to_string(),
        })
        .unwrap();

        let r = reqwest::Client::builder()
            .build()?
            .post(format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/ai/run/@cf/meta/m2m100-1.2b",
                self.account_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .body(body)
            .send()
            .await?;

        if r.status() != 200 {
            return Ok(r.text().await?);
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct Output {
            translated_text: String,
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct Result {
            result: Output,
        }

        Ok(r.json::<Result>().await?.result.translated_text)
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::ai::Workers;
    use hjdef::ai::AI;

    #[derive(serde::Deserialize)]
    struct Auth {
        account_id: String,
        api_token: String,
    }

    #[tokio::test]
    pub async fn completion() {
        let auth_json = fs::read_to_string("src/.api.json").unwrap();
        let auth = serde_json::from_str::<Auth>(&auth_json).unwrap();
        let ai = Workers::new(auth.account_id.as_str(), auth.api_token.as_str());
        println!(
            "{}",
            ai.gemma3_12b("辿るは何の意味ですか？".to_string())
                .await
                .unwrap()
        );
        println!(
            "{}",
            ai.llama4_scout_17b_16e_instruct("生意気の意味は？".to_string())
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    pub async fn translate() {
        let auth_json = fs::read_to_string("src/.api.json").unwrap();
        let auth = serde_json::from_str::<Auth>(&auth_json).unwrap();
        let ai = Workers::new(auth.account_id.as_str(), auth.api_token.as_str());
        println!(
            "{}",
            ai.m2m100_1_2b("辿るは何の意味ですか？", "ja", "zh")
                .await
                .unwrap()
        );
    }
}
