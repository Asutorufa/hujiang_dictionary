use std::sync::Arc;

use cloudflare::endpoints::ai::execute_model::{
    Message, MessageRole, MessagesParams, ResponseAndToolCallsResult, TranslationParams,
    TranslationResult,
};
use hjcommon::ai::{
    Choice, CompletionRequest, CompletionResponse, Error, Message as CommonMessage, Models,
    ResponseResponse, ResponsesRequest, WorkersAI,
};
use serde::{Serialize, de::DeserializeOwned};
use worker::{Ai, Env};

pub struct WasmAI {
    ai: Option<Arc<Ai>>,
}

fn completion_request_to_messages_params(msg: &CompletionRequest) -> MessagesParams {
    let mut messages = vec![];

    for m in msg.messages.iter() {
        messages.push(Message {
            role: match m.role.as_str() {
                "user" => MessageRole::User,
                "system" => MessageRole::System,
                "assistant" => MessageRole::Assistant,
                _ => MessageRole::User,
            },
            content: m.content.clone(),
        });
    }

    MessagesParams {
        messages: messages,
        stream: Some(false),
        ..Default::default()
    }
}

fn response_and_tool_calls_result_to_completion_response(
    result: ResponseAndToolCallsResult,
) -> CompletionResponse {
    CompletionResponse {
        choices: vec![Choice {
            message: CommonMessage {
                role: "assistant".to_string(),
                content: result.response,
                reasoning: None,
            },
        }],
    }
}

impl WasmAI {
    pub fn new(env: &Env, binding: &str) -> WasmAI {
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
        model: &str,
        msg: I,
    ) -> Result<O, Error> {
        let result: O = self
            .get_ai()?
            .run(model, msg)
            .await
            .map_err(|v| Error(v.to_string()))?;
        Ok(result)
    }
}

impl Clone for WasmAI {
    fn clone(&self) -> Self {
        Self {
            ai: self.ai.clone(),
        }
    }
}

impl WorkersAI for WasmAI {
    async fn completion(&self, req: CompletionRequest) -> Result<CompletionResponse, Error> {
        let result: ResponseAndToolCallsResult = self
            .exec(&req.model, completion_request_to_messages_params(&req))
            .await?;

        Ok(response_and_tool_calls_result_to_completion_response(
            result,
        ))
    }

    async fn responses(&self, req: ResponsesRequest) -> Result<ResponseResponse, Error> {
        self.exec(&req.model.clone(), req).await
    }

    fn enabled(&self) -> bool {
        self.ai.is_some()
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
        let result: TranslationResult = self.exec(Models::M2M100_1_2B.as_str(), msg).await?;
        Ok(result.translated_text)
    }
}
