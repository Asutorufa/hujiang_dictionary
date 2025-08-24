use hjdict::{en, google, jp, kotobakku, weblio};
use serde::{Deserialize, Serialize};

use crate::{
    ai::AI,
    d1::{DBv2, Error as D1Error},
    opts::RunOpt,
};

#[derive(Deserialize)]
pub struct ListWordRequest {
    pub page_size: Option<u64>,
    pub page_number: Option<u64>,
    pub order_by: Option<String>,
}

#[derive(Deserialize)]
pub struct SaveWordRequest {
    pub word: String,
    pub explain: String,
}

#[derive(Deserialize)]
pub struct SingleWordRequest {
    pub word: String,
}

#[derive(Deserialize)]
pub struct ChangeWordPriorityRequest {
    pub word: String,
    pub priority: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WordQueryRequest {
    pub method: String,
    pub word: String,
    pub src_lang: Option<String>,
    pub dst_lang: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WordQueryResponse {
    pub result: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WordCountResponse {
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl std::error::Error for Error {}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error(s.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<D1Error> for Error {
    fn from(value: D1Error) -> Self {
        Self(value.to_string())
    }
}

impl<T1: DBv2, T2: AI> RunOpt<T1, T2> {
    pub async fn route(&self, path: &str, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        match path {
            "/word/list" => self.list_word(body).await,
            "/word/query" => self.word_query(body).await,
            "/word/count" => self.count_word().await,
            "/word/save" => self.save_word(body).await,
            "/word/delete" => self.delete_word(body).await,
            "/word/remind_count_increment" => self.increment_remind_count(body).await,
            "/word/priority" => self.change_priority(body).await,
            _ => Err(Error("not found".to_string())),
        }
    }

    pub async fn list_word(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<ListWordRequest>(&body)?;

        let order_by = req.order_by.clone().unwrap_or("word".to_string());

        let order_by = if order_by.ends_with(" desc") {
            order_by.strip_suffix(" desc").unwrap().to_string()
        } else {
            order_by
        };

        match order_by.as_str() {
            "word" | "update_time" | "priority" => {}
            _ => return Err(Error("invalid order_by".to_string())),
        }

        let words = self
            .d1
            .list_word(
                req.page_size.unwrap_or(10),
                req.page_number.unwrap_or(1),
                req.order_by.unwrap_or("word".to_string()).as_ref(),
            )
            .await?;

        Ok(serde_json::to_vec(&words)?)
    }

    pub async fn save_word(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<SaveWordRequest>(&body)?;

        self.d1.save_word(&req.word, &req.explain).await?;

        Ok(['{' as u8, '}' as u8].to_vec())
    }

    pub async fn delete_word(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<SingleWordRequest>(&body)?;

        self.d1.delete_word(&req.word).await?;

        Ok(['{' as u8, '}' as u8].to_vec())
    }

    pub async fn count_word(&self) -> Result<Vec<u8>, Error> {
        let size = self.d1.count_word().await?;

        Ok(serde_json::to_vec(&WordCountResponse { size })?)
    }

    pub async fn increment_remind_count(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<SingleWordRequest>(&body)?;
        self.d1.increment_remind_count(&req.word).await?;
        Ok(['{' as u8, '}' as u8].to_vec())
    }

    pub async fn change_priority(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<ChangeWordPriorityRequest>(&body)?;
        self.d1.change_priority(&req.word, req.priority).await?;
        Ok(['{' as u8, '}' as u8].to_vec())
    }

    fn llm_query(&self, req: WordQueryRequest) -> String {
        return if req.dst_lang.is_some() {
            format!("{}\nUser Language is: {}", req.word, req.dst_lang.unwrap())
        } else {
            req.word.clone()
        };
    }

    pub async fn word_query(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<WordQueryRequest>(&body)?;

        let result = match req.method.as_str() {
            "jc" => jp::get(req.word.as_str(), "jc")
                .await
                .unwrap()
                .iter()
                .map(|x| x.markdown())
                .collect::<Vec<_>>()
                .join("\n"),
            "cj" => jp::get(req.word.as_str(), "cj")
                .await
                .unwrap()
                .iter()
                .map(|x| x.markdown())
                .collect::<Vec<_>>()
                .join("\n"),
            "en" => en::get(req.word.as_str())
                .await
                .unwrap()
                .iter()
                .map(|x| x.markdown())
                .collect::<Vec<_>>()
                .join("\n"),
            "weblio" => weblio::get(&req.word).await.unwrap().join("\n"),
            "ktbk" => kotobakku::get(&req.word).await.unwrap().join("\n"),
            "gpt" => self
                .workers_ai
                .gpt_oss_20b(self.llm_query(req).as_ref())
                .await
                .unwrap(),
            "llama4" => self
                .workers_ai
                .llama4_scout_17b_16e_instruct(self.llm_query(req).as_ref())
                .await
                .unwrap(),
            "gemma" => self
                .workers_ai
                .gemma3_12b(self.llm_query(req).as_ref())
                .await
                .unwrap(),
            "google" | "googlev1" => {
                let target = req.dst_lang.clone().unwrap_or("en".to_string());

                let query = async |src: Option<String>, target: &str| {
                    if req.method == "googlev1" {
                        return google::translate(req.word.as_ref(), src, target).await;
                    } else {
                        return google::translatev2(req.word.as_ref(), src, target).await;
                    }
                };

                query(req.src_lang, target.as_ref())
                    .await
                    .unwrap()
                    .iter()
                    .map(|x| x.translation.as_ref())
                    .collect::<Vec<_>>()
                    .join("")
            }
            _ => {
                format!("Unknown command: {}", req.method)
            }
        };
        Ok(serde_json::to_vec(&WordQueryResponse { result })?)
    }
}
