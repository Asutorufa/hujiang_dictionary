use std::sync::Arc;

use cloudflare::endpoints::ai::execute_model::{
    Message, MessageRole, MessagesParams, ResponseAndToolCallsResult, TranslationParams,
    TranslationResult,
};
use hjcommon::ai::{AI, Error, Models, ResponseResponse, ResponsesRequest, SYSTEM_MSG};
use serde::{Serialize, de::DeserializeOwned};
use worker::{Ai, Env};

pub struct WasmAI {
    ai: Option<Arc<Ai>>,
}

impl WasmAI {
    pub fn new(env: Arc<Env>, binding: &str) -> WasmAI {
        Self {
            ai: match env.ai(binding) {
                Ok(v) => Some(Arc::new(v)),
                Err(_) => None,
            },
        }
    }

    fn get_ai(&self) -> Result<Arc<Ai>, Error> {
        match self.ai.as_ref() {
            Some(v) => Ok(v.clone()),
            None => Err(Error("ai not found".to_string())),
        }
    }

    pub async fn exec<I: Serialize, O: DeserializeOwned>(
        &self,
        model: Models,
        msg: I,
    ) -> Result<O, Error> {
        let result: O = self
            .get_ai()?
            .run(model.as_str(), msg)
            .await
            .map_err(|v| Error(v.to_string()))?;
        Ok(result)
    }

    pub async fn completion(
        &self,
        prompt: &str,
        instruction: Option<&str>,
        model: Models,
    ) -> Result<String, Error> {
        let mut mgs = vec![
            Message {
                role: MessageRole::System,
                content: SYSTEM_MSG.to_string(),
            },
            Message {
                role: MessageRole::User,
                content: prompt.to_string(),
            },
        ];

        if let Some(instruction) = instruction {
            mgs.push(Message {
                role: MessageRole::System,
                content: instruction.to_string(),
            })
        }

        let msg = MessagesParams {
            messages: mgs,
            stream: Some(false),
            ..Default::default()
        };

        let result: ResponseAndToolCallsResult = self.exec(model, msg).await?;

        Ok(result.response)
    }

    pub async fn response(
        &self,
        prompt: &str,
        instruction: Option<&str>,
        model: Models,
    ) -> Result<Vec<(String, String)>, Error> {
        let resp: ResponseResponse = self
            .exec(
                model.clone(),
                ResponsesRequest::new(model, prompt, instruction),
            )
            .await?;
        Ok(resp.content())
    }
}

impl Clone for WasmAI {
    fn clone(&self) -> Self {
        Self {
            ai: self.ai.clone(),
        }
    }
}

impl AI for WasmAI {
    async fn gemma3_12b(&self, prompt: &str, instruction: Option<&str>) -> Result<String, Error> {
        self.completion(prompt, instruction, Models::Gemma3_12bIt)
            .await
    }

    async fn llama4_scout_17b_16e_instruct(
        &self,
        prompt: &str,
        instruction: Option<&str>,
    ) -> Result<String, Error> {
        self.completion(prompt, instruction, Models::Llama4Scout17B16EInstruct)
            .await
    }

    async fn gpt_oss_20b(&self, prompt: &str, instruction: Option<&str>) -> Result<String, Error> {
        let content = self
            .response(prompt, instruction, Models::GPTOss20B)
            .await?;

        let text = content
            .iter()
            .map(|(a, b)| format!("{}:\n{}", a, b))
            .collect::<Vec<String>>()
            .join("\n\n");

        Ok(text)
    }

    async fn m2m100_1_2b(
        &self,
        text: &str,
        source_lang: Option<String>,
        target_lang: String,
    ) -> Result<String, Error> {
        let msg = TranslationParams {
            target_lang,
            text: text.to_string(),
            source_lang,
        };
        let result: TranslationResult = self.exec(Models::M2M100_1_2B, msg).await?;
        Ok(result.translated_text)
    }
}
