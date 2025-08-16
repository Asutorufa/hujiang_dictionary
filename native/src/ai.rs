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
use hjcommon::ai::{AI, Error, Models, SYSTEM_MSG};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Workers {
    client: Client<OpenAIConfig>,
    account_id: String,
    api_token: String,
}

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

    pub fn is_valid(&self) -> Result<(), OpenAIError> {
        if self.account_id.is_empty() || self.api_token.is_empty() {
            return Err(OpenAIError::InvalidArgument(
                "account_id or api_token is empty".to_string(),
            ));
        }

        Ok(())
    }

    pub async fn completion(
        &self,
        msgs: Vec<ChatCompletionRequestMessage>,
        model: &str,
    ) -> Result<String, OpenAIError> {
        self.is_valid()?;

        let model = model.to_string();
        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(msgs)
            .build()?;

        let result = self.client.chat().create(request).await?;

        let content = result
            .choices
            .first()
            .ok_or(OpenAIError::InvalidArgument("choice is empty".to_string()))?
            .message
            .content
            .clone()
            .ok_or(OpenAIError::InvalidArgument("content is empty".to_string()))?;

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
                Models::Gemma3_12bIt.as_str(),
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
            Models::Llama4Scout17B16EInstruct.as_str(),
        )
        .await
        .map_err(|e| Error::from(e.to_string()))
    }

    async fn m2m100_1_2b(
        &self,
        text: String,
        source_lang: Option<String>,
        target_lang: String,
    ) -> Result<String, Error> {
        self.is_valid().map_err(|e| Error(e.to_string()))?;

        #[derive(Debug, Serialize, Deserialize)]
        struct Request {
            text: String,
            target_lang: String,

            #[serde(skip_serializing_if = "Option::is_none")]
            source_lang: Option<String>,
        }

        let body = serde_json::to_string(&Request {
            source_lang,
            target_lang,
            text,
        })
        .unwrap();

        let r = reqwest::Client::builder()
            .build()?
            .post(format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/ai/run/{}",
                self.account_id,
                Models::M2M100_1_2B.as_str(),
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
    use hjcommon::ai::AI;

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
            ai.m2m100_1_2b("辿るは何の意味ですか？".to_string(), None, "zh".to_string())
                .await
                .unwrap()
        );
    }
}
