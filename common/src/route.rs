use chrono::{Duration, Utc};
use d1_orm::DatabaseExecutor;
use frankenstein::updates::Update;
use hjdict::{en, google, jp, kotobanku, kr, weblio};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use log::*;
use serde::{Deserialize, Serialize};
use std::str;
use std::sync::Arc;
use subtle::ConstantTimeEq;

use crate::{
    ai::{self, Translator},
    d1::{Count, Error as D1Error, Queries, Word, list_word_query},
    opts::RunOpt,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
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
                ret.push('\n');
            }
            ret.push_str(i.as_str());
        }

        if let Some(l) = &self.dst_lang
            && !l.is_empty()
        {
            if !ret.is_empty() {
                ret.push('\n');
            }

            ret.push_str("\nTarget Language: ");
            ret.push_str(l);
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
pub enum Error {
    NotFound,
    Internal(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotFound => write!(f, "not found"),
            Error::Internal(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Internal(s.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Internal(value.to_string())
    }
}

impl From<D1Error> for Error {
    fn from(value: D1Error) -> Self {
        Self::Internal(value.to_string())
    }
}

impl From<ai::Error> for Error {
    fn from(value: ai::Error) -> Self {
        Self::Internal(value.to_string())
    }
}

pub struct UnifiedResponse {
    pub status: u16,
    pub body: UnifiedBody,
    pub headers: Vec<(String, String)>,
}

impl UnifiedResponse {
    pub fn ok(body: Vec<u8>) -> Self {
        Self {
            status: 200,
            body: UnifiedBody::Bytes(body),
            headers: vec![],
        }
    }

    pub fn json(body: Vec<u8>) -> Self {
        Self {
            status: 200,
            body: UnifiedBody::Bytes(body),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        }
    }

    pub fn error(status: u16, msg: String) -> Self {
        Self {
            status,
            body: UnifiedBody::Bytes(msg.into_bytes()),
            headers: vec![],
        }
    }
}

impl<T1: DatabaseExecutor, T2: Translator> RunOpt<T1, T2> {
    pub fn create_token(&self) -> Result<String, String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(self.auth_token_expiration))
            .expect("valid timestamp")
            .timestamp() as u64;

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

        let username_match = req.username.as_bytes().ct_eq(self.auth_username.as_bytes());
        let password_match = req.password.as_bytes().ct_eq(self.auth_password.as_bytes());

        if (username_match & password_match).into() {
            match self.create_token() {
                Ok(token) => Ok(LoginResponse { token }),
                Err(e) => Err((e, 500)),
            }
        } else {
            Err(("Invalid credentials".to_string(), 401))
        }
    }

    pub async fn serve(
        self: Arc<Self>,
        method: &str,
        path: &str,
        auth_header: Option<&str>,
        body: Vec<u8>,
        domain: &str,
    ) -> Result<UnifiedResponse, Error> {
        match path {
            "/login" => {
                if method != "POST" {
                    // Return 405 Method Not Allowed
                    return Ok(UnifiedResponse::error(
                        405,
                        "Method not allowed".to_string(),
                    ));
                }
                let req = serde_json::from_slice::<LoginRequest>(&body)?;
                match self.login(req) {
                    Ok(resp) => Ok(UnifiedResponse::json(serde_json::to_vec(&resp)?)),
                    Err((msg, status)) => Ok(UnifiedResponse::error(status, msg)),
                }
            }
            "/tgbot/register" => {
                if let Err(e) = self.check_auth(auth_header) {
                    return Ok(UnifiedResponse::error(401, e));
                }
                let url = format!("https://{}/tgbot", domain);
                let config = self
                    .get_config()
                    .await
                    .map_err(|e| Error::Internal(e.to_string()))?;
                if let Some(bot) = &config.bot {
                    crate::tg::set_webhook(bot, url.as_ref(), config.maintainer_id)
                        .await
                        .map_err(|e| Error::Internal(e.to_string()))?;
                    Ok(UnifiedResponse::ok(
                        format!("register telegram bot to {} successful", url).into_bytes(),
                    ))
                } else {
                    Ok(UnifiedResponse::error(
                        500,
                        "telegram bot token not configured".to_string(),
                    ))
                }
            }
            "/d1/create_table" => {
                if let Err(e) = self.check_auth(auth_header) {
                    return Ok(UnifiedResponse::error(401, e));
                }
                d1_orm::migrate(
                    &self.d1,
                    crate::d1::migrations(),
                    None,
                    Some(|s: &str| info!("{}", s)),
                )
                .await
                .map_err(|e| Error::Internal(e.to_string()))?;
                Ok(UnifiedResponse::ok(
                    "create table [words] successful".to_string().into_bytes(),
                ))
            }
            "/tgbot" => {
                if method != "POST" {
                    return Ok(UnifiedResponse::error(
                        405,
                        "Method not allowed".to_string(),
                    ));
                }
                let update = serde_json::from_slice::<Update>(&body)?;
                match crate::tg::handle(self, update).await {
                    Ok(_) => Ok(UnifiedResponse::ok(
                        "Update was handled by bot.".to_string().into_bytes(),
                    )),
                    Err(e) => Ok(UnifiedResponse::ok(
                        format!("Update was not handled by bot: {}", e).into_bytes(),
                    )),
                }
            }
            _ => {
                if path.starts_with("/word/")
                    || path.starts_with("/llm/")
                    || path.starts_with("/config/")
                {
                    if let Err(e) = self.check_auth(auth_header) {
                        return Ok(UnifiedResponse::error(401, e));
                    }
                    if path == "/word/query_stream" {
                        return self.word_query_stream(body).await;
                    }
                    let resp = self.route(path, body).await?;
                    Ok(UnifiedResponse::json(resp))
                } else {
                    Err(Error::NotFound)
                }
            }
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
            "/llm/list" => self.list_llm_providers().await,
            "/llm/save" => self.save_llm_provider(body).await,
            "/llm/delete" => self.delete_llm_provider(body).await,
            "/config/list" => self.list_configurations().await,
            "/config/save" => self.save_configuration(body).await,
            _ => Err(Error::NotFound),
        }
    }

    pub async fn list_word(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<ListWordRequest>(&body)?;

        let order_by = req.order_by.as_deref().unwrap_or("word");

        let mut is_desc = false;
        let order_by = if let Some(r) = order_by.strip_suffix(" desc") {
            is_desc = true;
            r
        } else {
            order_by
        };

        match order_by {
            "word" | "update_time" | "priority" | "reminder_time" | "anki_count" | "add_time" => {}
            _ => return Err(Error::Internal("invalid order_by".to_string())),
        }

        let query = list_word_query(
            req.page_size.unwrap_or(10),
            req.page_number.unwrap_or(1),
            order_by,
            is_desc,
            req.r#type.unwrap_or(0),
        );

        let words: Vec<Word> = self.d1.query_all(query).await?;

        Ok(serde_json::to_vec(&words)?)
    }

    pub async fn save_word(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<SaveWordRequest>(&body)?;

        if let Some(origin) = req.origin.as_deref()
            && origin != req.word
        {
            self.d1
                .execute(Queries::RenameWord {
                    new_word: &req.word,
                    explain: &req.explain,
                    word_type: req.r#type.unwrap_or(0),
                    example: req.example.as_deref().unwrap_or(""),
                    old_word: origin,
                })
                .await?;

            return Ok([b'{', b'}'].to_vec());
        }

        self.d1
            .execute(Queries::SaveWord {
                word: &req.word,
                explain: &req.explain,
                word_type: req.r#type.unwrap_or(0),
                example: req.example.as_deref().unwrap_or(""),
            })
            .await?;

        Ok([b'{', b'}'].to_vec())
    }

    pub async fn delete_word(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<SingleWordRequest>(&body)?;

        self.d1
            .execute(Queries::DeleteWord { word: &req.word })
            .await?;

        Ok([b'{', b'}'].to_vec())
    }

    pub async fn count_word(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let r#type = match serde_json::from_slice::<ListWordRequest>(&body) {
            Ok(req) => req.r#type.unwrap_or(0),
            Err(_) => 0,
        };

        let count: Option<Count> = self
            .d1
            .query_first(Queries::CountWord { word_type: r#type })
            .await?;
        let size = count.map(|c| c.size).unwrap_or(0);

        Ok(serde_json::to_vec(&WordCountResponse { size })?)
    }

    pub async fn increment_remind_count(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<SingleWordRequest>(&body)?;
        self.d1
            .execute(Queries::IncrementRemindCount { word: &req.word })
            .await?;
        Ok([b'{', b'}'].to_vec())
    }

    pub async fn list_configurations(&self) -> Result<Vec<u8>, Error> {
        let configurations: Vec<crate::d1::Configuration> = self
            .d1
            .query_all(crate::d1::Queries::ListConfigurations)
            .await?;

        Ok(serde_json::to_vec(&configurations)?)
    }

    pub async fn save_configuration(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<crate::d1::Configuration>(&body)?;
        self.d1
            .execute(crate::d1::Queries::SaveConfiguration {
                key: &req.key,
                value: &req.value,
            })
            .await?;

        let mut cache = self
            .config_cache
            .write()
            .map_err(|e| Error::Internal(e.to_string()))?;
        cache.last_updated = 0; // Invalidate cache

        Ok([b'{', b'}'].to_vec())
    }

    pub async fn change_priority(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<ChangeWordPriorityRequest>(&body)?;
        self.d1
            .execute(Queries::ChangePriority {
                priority: req.priority,
                word: &req.word,
            })
            .await?;
        Ok([b'{', b'}'].to_vec())
    }

    pub async fn list_llm_providers(&self) -> Result<Vec<u8>, Error> {
        let providers: Vec<crate::d1::LlmProvider> = self
            .d1
            .query_all(crate::d1::Queries::ListLlmProviders)
            .await?;
        Ok(serde_json::to_vec(&providers)?)
    }

    pub async fn save_llm_provider(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<crate::d1::LlmProvider>(&body)?;
        self.d1
            .execute(crate::d1::Queries::SaveLlmProvider {
                name: &req.name,
                base_url: &req.base_url,
                api_key: &req.api_key,
                provider: &req.provider,
                models: &req.models,
                features: &req.features,
            })
            .await?;
        Ok([b'{', b'}'].to_vec())
    }

    pub async fn delete_llm_provider(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        #[derive(Deserialize)]
        struct DeleteReq {
            name: String,
        }
        let req = serde_json::from_slice::<DeleteReq>(&body)?;
        self.d1
            .execute(crate::d1::Queries::DeleteLlmProvider { name: &req.name })
            .await?;
        Ok([b'{', b'}'].to_vec())
    }

    pub async fn get_custom_llm_providers(
        &self,
    ) -> Result<std::collections::HashMap<String, hj_ai::provider::Provider>, Error> {
        let providers: Vec<crate::d1::LlmProvider> = self
            .d1
            .query_all(crate::d1::Queries::ListLlmProviders)
            .await?;
        let mut custom_llms = std::collections::HashMap::new();
        for p in providers {
            custom_llms.insert(p.name.clone(), Self::map_llm_provider_to_config(p));
        }
        Ok(custom_llms)
    }

    pub async fn get_llm_provider_by_name(
        &self,
        name: &str,
    ) -> Result<hj_ai::provider::Provider, Error> {
        let p: Option<crate::d1::LlmProvider> = self
            .d1
            .query_first(crate::d1::Queries::GetLlmProviderByName { name })
            .await?;
        let p = p.ok_or_else(|| Error::Internal("custom llm not found".to_string()))?;
        Ok(Self::map_llm_provider_to_config(p))
    }

    fn map_llm_provider_to_config(p: crate::d1::LlmProvider) -> hj_ai::provider::Provider {
        let config = crate::ai::ConfigProvider {
            name: p.name.clone(),
            base_url: if p.base_url.is_empty() {
                None
            } else {
                Some(p.base_url)
            },
            api_key: if p.api_key.is_empty() {
                None
            } else {
                Some(p.api_key)
            },
            provider: match p.provider.as_str() {
                "gemini" => Some(crate::ai::ProviderType::Gemini),
                "vertexai" => Some(crate::ai::ProviderType::VertexAI),
                "workersai" => Some(crate::ai::ProviderType::WorkersAI),
                _ => Some(crate::ai::ProviderType::OpenAI),
            },
            models: p
                .models
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect(),
            features: if p.features.is_empty() {
                None
            } else {
                serde_json::from_str(&p.features).ok()
            },
        };
        hj_ai::provider::Provider::from(config)
    }

    pub async fn custom_llms(&self) -> Result<Vec<u8>, Error> {
        let mut models = vec![];

        if let Some(workers_ai) = &self.workers_ai {
            models.push(CustomLLMResponse {
                name: "workers-ai".to_string(),
                models: workers_ai.models(),
            });
        }

        let custom_llms = self.get_custom_llm_providers().await?;

        for (name, l) in custom_llms.iter() {
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
                    None => return Err(Error::Internal("custom llm is empty".to_string())),
                };

                match name.as_str() {
                    "workers-ai" => {
                        let mut provider = self.workers_ai.as_ref().unwrap().clone();
                        if let hj_ai::provider::Provider::WorkersAI(ref mut w) = provider {
                            w.model = model.to_string();
                        }
                        Some(
                            crate::ai::explain(
                                &provider,
                                req.google_search.unwrap_or(false),
                                crate::ai::TranslateRequest {
                                    model,
                                    chars_limit: false,
                                    query: &req.word,
                                    instruction: req.instruction().as_deref(),
                                    dst_lang: req.dst_lang.as_deref(),
                                },
                            )
                            .await?,
                        )
                    }
                    _ => {
                        let mut provider = self.get_llm_provider_by_name(name).await?;
                        provider.set_model(model);
                        Some(
                            crate::ai::explain(
                                &provider,
                                req.google_search.unwrap_or(false),
                                crate::ai::TranslateRequest {
                                    model,
                                    chars_limit: false,
                                    query: &req.word,
                                    instruction: req.instruction().as_deref(),
                                    dst_lang: req.dst_lang.as_deref(),
                                },
                            )
                            .await?,
                        )
                    }
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
                Err(e) => return Err(Error::Internal(e.to_string())),
            },
            "cj" => match jp::get(req.word.as_str(), "cj").await {
                Ok(v) => v
                    .iter()
                    .map(|x| x.markdown())
                    .collect::<Vec<_>>()
                    .join("\n"),
                Err(e) => return Err(Error::Internal(e.to_string())),
            },
            "kr" => match kr::get(req.word.as_str()).await {
                Ok(v) => v
                    .iter()
                    .map(|x| x.markdown())
                    .collect::<Vec<_>>()
                    .join("\n"),
                Err(e) => return Err(Error::Internal(e.to_string())),
            },
            "en" => match en::get(req.word.as_str()).await {
                Ok(v) => v
                    .iter()
                    .map(|x| x.markdown())
                    .collect::<Vec<_>>()
                    .join("\n"),
                Err(e) => return Err(Error::Internal(e.to_string())),
            },
            "weblio" => match weblio::get(&req.word).await {
                Ok(v) => v.join("\n"),
                Err(e) => return Err(Error::Internal(e.to_string())),
            },
            "ktbk" => match kotobanku::get(&req.word).await {
                Ok(v) => v.join("\n"),
                Err(e) => return Err(Error::Internal(e.to_string())),
            },
            "m2m100_1_2b" => {
                self.translator
                    .m2m100_1_2b(
                        &req.word,
                        req.src_lang,
                        req.dst_lang.unwrap_or_else(|| "en".to_string()),
                    )
                    .await?
            }
            "google" | "googlev1" => {
                let target = req.dst_lang.as_deref().unwrap_or("en");

                let query = async |src: Option<String>, target: &str| {
                    if req.method == "googlev1" {
                        return google::translate(req.word.as_ref(), src, target).await;
                    } else {
                        return google::translatev2(req.word.as_ref(), src, target).await;
                    }
                };

                match query(req.src_lang, target).await {
                    Ok(v) => v
                        .iter()
                        .map(|x| x.translation.as_ref())
                        .collect::<Vec<_>>()
                        .join(""),
                    Err(e) => return Err(Error::Internal(e.to_string())),
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

    pub async fn word_query_stream(&self, body: Vec<u8>) -> Result<UnifiedResponse, Error> {
        let req = serde_json::from_slice::<WordQueryRequest>(&body)?;

        let stream = match req.method.as_str() {
            "custom_llm" => {
                let (name, model) = match req.custom_llm.as_ref() {
                    Some(llm) => (&llm.name, &llm.model),
                    None => return Err(Error::Internal("custom llm is empty".to_string())),
                };

                match name.as_str() {
                    "workers-ai" => {
                        let mut provider = self.workers_ai.as_ref().unwrap().clone();
                        if let hj_ai::provider::Provider::WorkersAI(ref mut w) = provider {
                            w.model = model.to_string();
                        }
                        crate::ai::explain_stream(
                            &provider,
                            req.google_search.unwrap_or(false),
                            crate::ai::TranslateRequest {
                                model,
                                chars_limit: false,
                                query: &req.word,
                                instruction: req.instruction().as_deref(),
                                dst_lang: req.dst_lang.as_deref(),
                            },
                        )
                        .await?
                    }
                    _ => {
                        let mut provider = self.get_llm_provider_by_name(name).await?;
                        provider.set_model(model);
                        crate::ai::explain_stream(
                            &provider,
                            req.google_search.unwrap_or(false),
                            crate::ai::TranslateRequest {
                                model,
                                chars_limit: false,
                                query: &req.word,
                                instruction: req.instruction().as_deref(),
                                dst_lang: req.dst_lang.as_deref(),
                            },
                        )
                        .await?
                    }
                }
            }
            _ => return Err(Error::Internal("method not support stream".to_string())),
        };

        use futures_util::StreamExt;
        let s = stream.map(|res| match res {
            Ok(v) => {
                let resp = WordQueryResponse {
                    result: v.content,
                    reasoning: v.thinking,
                };
                let json = serde_json::to_string(&resp).unwrap_or_default();
                Ok(bytes::Bytes::from(format!("data: {}\n\n", json)))
            }
            Err(e) => Err(e),
        });

        Ok(UnifiedResponse::stream(Box::pin(s)))
    }
}

use futures_util::Stream;
use std::pin::Pin;

pub type BoxedStream =
    Pin<Box<dyn Stream<Item = Result<bytes::Bytes, Box<dyn std::error::Error>>>>>;

pub enum UnifiedBody {
    Bytes(Vec<u8>),
    Stream(BoxedStream),
}

impl UnifiedResponse {
    pub fn stream(stream: BoxedStream) -> Self {
        Self {
            status: 200,
            body: UnifiedBody::Stream(stream),
            headers: vec![
                ("Content-Type".to_string(), "text/event-stream".to_string()),
                ("Cache-Control".to_string(), "no-cache".to_string()),
                ("Connection".to_string(), "keep-alive".to_string()),
            ],
        }
    }
}
