#!/bin/bash
cat << 'INNER_EOF' > worker/src/ai.rs
use std::sync::Arc;

use cloudflare::endpoints::ai::execute_model::{TranslationParams, TranslationResult};
use hjcommon::ai::{Error, Translator};
use serde::{Serialize, de::DeserializeOwned};
use worker::{Ai, Env};

pub struct WasmAI {
    pub ai: Option<Arc<Ai>>,
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

impl Translator for WasmAI {
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
        let result: TranslationResult = self.exec("@cf/meta/m2m100-1.2b", msg).await?;
        Ok(result.translated_text)
    }
}
INNER_EOF
