use std::sync::Arc;

use cloudflare::endpoints::ai::execute_model::{
    Message, MessageRole, MessagesParams, ResponseAndToolCallsResult, TranslationParams,
    TranslationResult,
};
use hjcommon::ai::{AI, Models, SYSTEM_MSG};
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

    fn get_ai(&self) -> Result<Arc<Ai>, hjcommon::ai::Error> {
        match self.ai.as_ref() {
            Some(v) => Ok(v.clone()),
            None => Err(hjcommon::ai::Error("ai not found".to_string())),
        }
    }

    pub async fn completion(
        &self,
        prompt: String,
        model: &str,
    ) -> Result<String, hjcommon::ai::Error> {
        let msg = MessagesParams {
            messages: vec![
                Message {
                    role: MessageRole::System,
                    content: SYSTEM_MSG.to_string(),
                },
                Message {
                    role: MessageRole::User,
                    content: prompt,
                },
            ],
            stream: Some(false),
            ..Default::default()
        };

        let result: ResponseAndToolCallsResult = self
            .get_ai()?
            .run(model, msg)
            .await
            .map_err(|v| hjcommon::ai::Error(v.to_string()))?;

        Ok(result.response)
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
    async fn gemma3_12b(&self, prompt: String) -> Result<String, hjcommon::ai::Error> {
        self.completion(prompt, Models::Gemma3_12bIt.as_str()).await
    }

    async fn llama4_scout_17b_16e_instruct(
        &self,
        prompt: &str,
    ) -> Result<String, hjcommon::ai::Error> {
        self.completion(
            prompt.to_string(),
            Models::Llama4Scout17B16EInstruct.as_str(),
        )
        .await
    }

    async fn m2m100_1_2b(
        &self,
        text: &str,
        source_lang: Option<String>,
        target_lang: String,
    ) -> Result<String, hjcommon::ai::Error> {
        let msg = TranslationParams {
            target_lang,
            text: text.to_string(),
            source_lang,
        };
        let result: TranslationResult = self
            .get_ai()?
            .run(Models::M2M100_1_2B.as_str(), msg)
            .await
            .map_err(|v| hjcommon::ai::Error(v.to_string()))?;

        Ok(result.translated_text)
    }
}
