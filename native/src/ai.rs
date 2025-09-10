use hjcommon::ai::{
    CompletionRequest, CompletionResponse, Error, Models, ResponseResponse, ResponsesRequest,
    TranslateRequest, TranslateResult, WorkersAI,
};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct Workers {
    account_id: String,
    api_token: String,
}

impl Workers {
    pub fn new(account_id: &str, api_key: &str) -> Self {
        Workers {
            account_id: account_id.to_string(),
            api_token: api_key.to_string(),
        }
    }

    pub fn is_valid(&self) -> Result<(), Error> {
        if self.account_id.is_empty() || self.api_token.is_empty() {
            return Err(Error("account_id or api_token is empty".to_string()));
        }
        Ok(())
    }

    pub async fn exec<I: Serialize, O: DeserializeOwned>(
        &self,
        path: &str,
        input: I,
    ) -> Result<O, Error> {
        self.is_valid()?;

        let body = serde_json::to_string(&input).unwrap();

        let r = reqwest::Client::builder()
            .build()?
            .post(format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/ai/{}",
                self.account_id, path
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .body(body)
            .send()
            .await?;

        if r.status() != 200 {
            return Err(Error(r.text().await?));
        }

        Ok(r.json::<O>().await?)
    }
}

impl WorkersAI for Workers {
    async fn completion(&self, req: CompletionRequest) -> Result<CompletionResponse, Error> {
        self.exec("v1/chat/completions", req).await
    }

    async fn responses(&self, req: ResponsesRequest) -> Result<ResponseResponse, Error> {
        self.exec("v1/responses", req).await
    }

    async fn m2m100_1_2b(
        &self,
        text: &str,
        source_lang: Option<String>,
        target_lang: String,
    ) -> Result<String, Error> {
        let r: TranslateResult = self
            .exec(
                format!("run/{}", Models::M2M100_1_2B.as_str()).as_str(),
                TranslateRequest::new(text.as_ref(), target_lang.as_ref(), source_lang),
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
