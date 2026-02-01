use chrono::{Duration, Utc};
use hjdict::{en, google, jp, kotobanku, kr, weblio};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::str;

use crate::{
    ai::{self, Models, WorkersAI},
    d1::{DB, Error as D1Error, SaveWord},
    opts::RunOpt,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct ListWordRequest {
    pub page_size: Option<u64>,
    pub page_number: Option<u64>,
    pub order_by: Option<String>,
    pub r#type: Option<i64>,
}

#[derive(Deserialize)]
pub struct SaveWordRequest {
    pub origin: Option<String>,
    pub word: String,
    pub explain: String,
    pub r#type: Option<i64>,
    pub example: Option<String>,
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
    pub custom_llm: Option<CustomLLM>,
    pub instruction: Option<String>,
    pub google_search: Option<bool>,
    pub src_lang: Option<String>,
    pub dst_lang: Option<String>,
}

impl WordQueryRequest {
    pub fn instruction(&self) -> Option<String> {
        if self.dst_lang.is_none() && self.instruction.is_none() {
            return None;
        }

        let mut ret = "".to_string();

        if let Some(i) = &self.instruction {
            if !ret.is_empty() {
                ret.push_str("\n");
            }
            ret.push_str(i.as_str());
        }

        if let Some(l) = &self.dst_lang {
            if !l.is_empty() {
                if !ret.is_empty() {
                    ret.push_str("\n");
                }

                ret.push_str("\nTarget Language: ");
                ret.push_str(l);
            }
        }

        if ret.is_empty() {
            return None;
        }

        Some(ret)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WordQueryResponse {
    pub result: String,
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WordCountResponse {
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CustomLLMResponse {
    pub name: String,
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomLLM {
    pub name: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

impl From<ai::Error> for Error {
    fn from(value: ai::Error) -> Self {
        Self(value.to_string())
    }
}

impl<T1: DB, T2: WorkersAI> RunOpt<T1, T2> {
    pub fn create_token(&self) -> Result<String, String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(self.auth_token_expiration))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: self.auth_username.clone(),
            exp: expiration,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.auth_secret.as_bytes()),
        )
        .map_err(|e| e.to_string())
    }

    pub fn check_auth(&self, auth_header: Option<&str>) -> Result<(), String> {
        if self.auth_username.is_empty() || self.auth_password.is_empty() {
            return Ok(());
        }

        let token = match auth_header {
            Some(h) if h.starts_with("Bearer ") => &h[7..],
            Some(_) => return Err("Invalid Authorization header format".to_string()),
            None => return Err("Missing Authorization header".to_string()),
        };

        let validation = Validation::default();
        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.auth_secret.as_bytes()),
            &validation,
        ) {
            Ok(_) => Ok(()),
            Err(_) => Err("Invalid or expired token".to_string()),
        }
    }

    pub fn login(&self, req: LoginRequest) -> Result<LoginResponse, (String, u16)> {
        if self.auth_username.is_empty() || self.auth_password.is_empty() {
            return Err(("Authentication not configured".to_string(), 500));
        }

        if req.username == self.auth_username && req.password == self.auth_password {
            match self.create_token() {
                Ok(token) => Ok(LoginResponse { token }),
                Err(e) => Err((e, 500)),
            }
        } else {
            Err(("Invalid credentials".to_string(), 401))
        }
    }

    pub async fn route(&self, path: &str, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        match path {
            "/word/list" => self.list_word(body).await,
            "/word/query" => self.word_query(body).await,
            "/word/count" => self.count_word(body).await,
            "/word/save" => self.save_word(body).await,
            "/word/delete" => self.delete_word(body).await,
            "/word/remind_count_increment" => self.increment_remind_count(body).await,
            "/word/priority" => self.change_priority(body).await,
            "/word/ai_custom" => self.custom_llms().await,
            _ => Err(Error("not found".to_string())),
        }
    }

    pub async fn list_word(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<ListWordRequest>(&body)?;

        let order_by = req.order_by.clone().unwrap_or("word".to_string());

        let order_by = if let Some(r) = order_by.strip_suffix(" desc") {
            r.to_string()
        } else {
            order_by
        };

        match order_by.as_str() {
            "word" | "update_time" | "priority" | "reminder_time" | "anki_count" | "add_time" => {}
            _ => return Err(Error("invalid order_by".to_string())),
        }

        let words = self
            .d1
            .list_word(
                req.page_size.unwrap_or(10),
                req.page_number.unwrap_or(1),
                req.order_by.unwrap_or("word".to_string()).as_ref(),
                req.r#type.unwrap_or(0),
            )
            .await?;

        Ok(serde_json::to_vec(&words)?)
    }

    pub async fn save_word(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<SaveWordRequest>(&body)?;

        self.d1
            .save_word(SaveWord {
                origin_word: req.origin.as_deref(),
                word: &req.word,
                explain: &req.explain,
                r#type: req.r#type.unwrap_or(0),
                example: &req.example.unwrap_or("".to_string()),
            })
            .await?;

        Ok(['{' as u8, '}' as u8].to_vec())
    }

    pub async fn delete_word(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<SingleWordRequest>(&body)?;

        self.d1.delete_word(&req.word).await?;

        Ok(['{' as u8, '}' as u8].to_vec())
    }

    pub async fn count_word(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let r#type = match serde_json::from_slice::<ListWordRequest>(&body) {
            Ok(req) => req.r#type.unwrap_or(0),
            Err(_) => 0,
        };

        let size = self.d1.count_word(r#type).await?;

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

    pub async fn custom_llms(&self) -> Result<Vec<u8>, Error> {
        let mut models = vec![];

        if self.workers_ai.enabled() {
            models.push(CustomLLMResponse {
                name: "workers-ai".to_string(),
                models: self.workers_ai.models(),
            });
        }

        for (name, l) in self.custom_llms.iter() {
            models.push(CustomLLMResponse {
                name: name.clone(),
                models: l.models(),
            });
        }
        Ok(serde_json::to_vec(&models)?)
    }

    pub async fn word_query(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<WordQueryRequest>(&body)?;

        let response = match req.method.as_str() {
            "custom_llm" => {
                let (name, model) = match req.custom_llm.as_ref() {
                    Some(llm) => (&llm.name, &llm.model),
                    None => return Err(Error("custom llm is empty".to_string())),
                };

                match name.as_str() {
                    "workers-ai" => Some(
                        self.workers_ai
                            .explain(
                                req.google_search.unwrap_or(false),
                                Models::from_str(model)
                                    .ok_or(Error("model not supported".to_string()))?,
                                false,
                                &req.word,
                                req.instruction().as_deref(),
                            )
                            .await?,
                    ),
                    _ => Some(
                        self.custom_llms
                            .get(name)
                            .ok_or(Error("custom llm not found".to_string()))?
                            .explain(
                                req.google_search.unwrap_or(false),
                                ai::TranslateRequest {
                                    model: model,
                                    chars_limit: false,
                                    query: &req.word,
                                    dst_lang: req.dst_lang.as_deref(),
                                    ..Default::default()
                                },
                            )
                            .await?,
                    ),
                }
            }
            _ => None,
        };

        if let Some(response) = response {
            return Ok(serde_json::to_vec(&WordQueryResponse {
                result: response.content,
                reasoning: response.reasoning,
            })?);
        }

        let result = match req.method.as_str() {
            "jc" => match jp::get(req.word.as_str(), "jc").await {
                Ok(v) => v
                    .iter()
                    .map(|x| x.markdown())
                    .collect::<Vec<_>>()
                    .join("\n"),
                Err(e) => return Err(Error(e.to_string())),
            },
            "cj" => match jp::get(req.word.as_str(), "cj").await {
                Ok(v) => v
                    .iter()
                    .map(|x| x.markdown())
                    .collect::<Vec<_>>()
                    .join("\n"),
                Err(e) => return Err(Error(e.to_string())),
            },
            "kr" => match kr::get(req.word.as_str()).await {
                Ok(v) => v
                    .iter()
                    .map(|x| x.markdown())
                    .collect::<Vec<_>>()
                    .join("\n"),
                Err(e) => return Err(Error(e.to_string())),
            },
            "en" => match en::get(req.word.as_str()).await {
                Ok(v) => v
                    .iter()
                    .map(|x| x.markdown())
                    .collect::<Vec<_>>()
                    .join("\n"),
                Err(e) => return Err(Error(e.to_string())),
            },
            "weblio" => match weblio::get(&req.word).await {
                Ok(v) => v.join("\n"),
                Err(e) => return Err(Error(e.to_string())),
            },
            "ktbk" => match kotobanku::get(&req.word).await {
                Ok(v) => v.join("\n"),
                Err(e) => return Err(Error(e.to_string())),
            },
            "m2m100_1_2b" => {
                self.workers_ai
                    .m2m100_1_2b(
                        &req.word,
                        req.src_lang,
                        req.dst_lang.unwrap_or("en".to_string()),
                    )
                    .await?
            }
            "google" | "googlev1" => {
                let target = req.dst_lang.clone().unwrap_or("en".to_string());

                let query = async |src: Option<String>, target: &str| {
                    if req.method == "googlev1" {
                        return google::translate(req.word.as_ref(), src, target).await;
                    } else {
                        return google::translatev2(req.word.as_ref(), src, target).await;
                    }
                };

                match query(req.src_lang, target.as_ref()).await {
                    Ok(v) => v
                        .iter()
                        .map(|x| x.translation.as_ref())
                        .collect::<Vec<_>>()
                        .join(""),
                    Err(e) => return Err(Error(e.to_string())),
                }
            }
            _ => {
                format!("Unknown command: {}", req.method)
            }
        };

        Ok(serde_json::to_vec(&WordQueryResponse {
            result,
            reasoning: None,
        })?)
    }
}
