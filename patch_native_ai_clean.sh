#!/bin/bash
cat << 'INNER_EOF' > native/src/ai.rs
use std::sync::Arc;

use cloudflare::endpoints::ai::execute_model::{TranslationParams, TranslationResult};
use hjcommon::ai::{Error, Translator};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct Workers {
    pub account_id: String,
    pub api_key: String,
}

impl Workers {
    pub fn new(account_id: &str, api_key: &str) -> Self {
        Workers {
            account_id: account_id.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn run<I: Serialize, O: DeserializeOwned>(
        &self,
        model: &str,
        input: I,
    ) -> Result<O, Error> {
        if self.account_id.is_empty() || self.api_key.is_empty() {
            return Err(Error("account_id or api_token is empty".to_string()));
        }

        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/ai/run/{}",
            self.account_id, model
        );
        let body = serde_json::to_string(&input)?;

        let r = reqwest::Client::builder()
            .build()?
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .body(body)
            .send()
            .await?;

        let status = r.status();

        if !status.is_success() {
            return Err(Error(r.text().await?));
        }

        Ok(r.json::<O>().await?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateResult {
    pub result: TranslationResult,
}

impl Translator for Workers {
    async fn m2m100_1_2b(
        &self,
        text: &str,
        source_lang: Option<String>,
        target_lang: String,
    ) -> Result<String, Error> {
        let r: TranslateResult = self
            .run(
                "@cf/meta/m2m100-1.2b",
                TranslationParams {
                    target_lang,
                    text: text.to_string(),
                    source_lang,
                },
            )
            .await?;
        Ok(r.result.translated_text)
    }
}
INNER_EOF
