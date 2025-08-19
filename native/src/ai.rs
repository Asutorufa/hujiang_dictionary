use hjcommon::ai::{
    AI, CompletionRequest, CompletionResponse, Error, Models, ResponseResponse, ResponsesRequest,
    TranslateRequest, TranslateResult,
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

    pub async fn completion(&self, prompt: &str, model: Models) -> Result<String, Error> {
        let r: CompletionResponse = self
            .exec("v1/chat/completions", CompletionRequest::new(model, prompt))
            .await?;
        Ok(r.choices
            .first()
            .ok_or(Error("choice is empty".to_string()))?
            .message
            .content
            .clone())
    }

    pub async fn response(
        &self,
        prompt: &str,
        model: Models,
    ) -> Result<Vec<(String, String)>, Error> {
        let r: ResponseResponse = self
            .exec("v1/responses", ResponsesRequest::new(model, prompt))
            .await?;
        Ok(r.content())
    }
}

impl AI for Workers {
    async fn gemma3_12b(&self, prompt: &str) -> Result<String, Error> {
        self.completion(prompt, Models::Gemma3_12bIt).await
    }

    async fn llama4_scout_17b_16e_instruct(&self, prompt: &str) -> Result<String, Error> {
        self.completion(prompt, Models::Llama4Scout17B16EInstruct)
            .await
    }

    async fn gpt_oss_20b(&self, prompt: &str) -> Result<String, Error> {
        let content = self
            .response(prompt, Models::GPTOss20B)
            .await
            .map_err(|e| Error::from(e.to_string()))?;

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
            "gemma: {}",
            ai.gemma3_12b("辿るは何の意味ですか？").await.unwrap()
        );
        println!(
            "gpt-oss-20b: {}",
            ai.gpt_oss_20b("辿るは何の意味ですか？").await.unwrap()
        );
        println!(
            "llama4_scout_17b_16e_instruct: {}",
            ai.llama4_scout_17b_16e_instruct("生意気の意味は？")
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
            ai.m2m100_1_2b("辿るは何の意味ですか？", None, "zh".to_string())
                .await
                .unwrap()
        );
    }
}
