use std::sync::Arc;

use cloudflare::endpoints::ai::execute_model::{TranslationParams, TranslationResult};
use hjcommon::ai::{
    CompletionRequest, CompletionResponse, Error, Models, OpenAI, ResponseResponse,
    ResponsesRequest, WorkersAI,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct Workers {
    openai: Option<Arc<OpenAI>>,
}

fn remove_last_path(url: &str) -> &str {
    match url.rfind('/') {
        Some(pos) if pos > "https://".len() => &url[..pos],
        _ => url,
    }
}

impl Workers {
    pub fn new(account_id: &str, api_key: &str) -> Self {
        Workers {
            openai: if account_id == "" || api_key == "" {
                None
            } else {
                Some(Arc::new(
                    OpenAI::new(
                        "workers ai",
                        format!(
                            "https://api.cloudflare.com/client/v4/accounts/{}/ai/v1",
                            account_id
                        )
                        .as_str(),
                        api_key,
                        vec![],
                    )
                    .set_allow_all_models(true),
                ))
            },
        }
    }

    pub fn openai(&self) -> Result<Arc<OpenAI>, Error> {
        match &self.openai {
            Some(openai) => Ok(openai.clone()),
            None => Err(Error("account_id or api_token is empty".to_string())),
        }
    }

    pub async fn run<I: Serialize, O: DeserializeOwned>(
        &self,
        model: &str,
        input: I,
    ) -> Result<O, Error> {
        let client = self.openai()?;
        let url = remove_last_path(client.base_url.as_str());
        let body = serde_json::to_string(&input)?;

        let r = reqwest::Client::builder()
            .build()?
            .post(format!("{}/run/{}", url, model))
            .header("Authorization", self.openai()?.authorization_header())
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

impl WorkersAI for Workers {
    async fn completion(&self, req: CompletionRequest) -> Result<CompletionResponse, Error> {
        self.openai()?.completion(req).await
    }

    async fn responses(&self, req: ResponsesRequest) -> Result<ResponseResponse, Error> {
        self.openai()?.responses(req).await
    }

    fn enabled(&self) -> bool {
        self.openai.is_some()
    }

    async fn m2m100_1_2b(
        &self,
        text: &str,
        source_lang: Option<String>,
        target_lang: String,
    ) -> Result<String, Error> {
        let r: TranslateResult = self
            .run(
                Models::M2M100_1_2B.as_str(),
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

#[cfg(test)]
mod test {
    use crate::ai::Workers;
    use hjcommon::ai::{Models, WorkersAI};
    use std::fs;

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
            "gemma: {}",
            ai.translate(
                Models::Gemma3_12bIt,
                false,
                "辿るは何の意味ですか？",
                Some("日本語に翻訳してね。"),
            )
            .await
            .unwrap()
            .to_string()
        );
    }

    #[tokio::test]
    pub async fn translate() {
        let auth_json = fs::read_to_string("src/.api.json").unwrap();
        let auth = serde_json::from_str::<Auth>(&auth_json).unwrap();
        let ai = Workers::new(auth.account_id.as_str(), auth.api_token.as_str());
        println!(
            "{}",
            ai.m2m100_1_2b("辿るは何の意味ですか？", None, "zh".to_string())
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    pub async fn response() {
        let auth_json = fs::read_to_string("src/.api.json").unwrap();
        let auth = serde_json::from_str::<Auth>(&auth_json).unwrap();
        let ai = Workers::new(auth.account_id.as_str(), auth.api_token.as_str());

        println!(
            "{}",
            ai.google_search(
                hjcommon::ai::Models::GPTOss20B,
                false,
                "辿るは何の意味ですか？"
            )
            .await
            .unwrap()
            .to_string()
        );
    }
}
