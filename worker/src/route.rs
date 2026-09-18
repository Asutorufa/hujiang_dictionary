use base64::{
    Engine,
    engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD},
};
use chrono::{Duration, Utc};
use d1_orm::DatabaseExecutor;
use frankenstein::updates::Update;
use hjdict::{en, google, jp, kotobanku, kr, weblio};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::str;
use std::sync::Arc;
use subtle::ConstantTimeEq;

use crate::{
    ai::{self, Translator},
    d1::{CodexOAuthState, Count, Error as D1Error, LlmProvider, Queries, Word, list_word_query},
    mcp,
    opts::WorkerState,
};

const CODEX_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const CODEX_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const CODEX_DEVICE_USER_CODE_URL: &str = "https://auth.openai.com/api/accounts/deviceauth/usercode";
const CODEX_DEVICE_TOKEN_URL: &str = "https://auth.openai.com/api/accounts/deviceauth/token";
const CODEX_DEVICE_VERIFICATION_URL: &str = "https://auth.openai.com/codex/device";
const CODEX_DEVICE_REDIRECT_URI: &str = "https://auth.openai.com/deviceauth/callback";
const CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api";
const CODEX_OAUTH_STATE_TTL_SECONDS: i64 = 10 * 60;
const CODEX_TOKEN_REFRESH_SKEW_SECONDS: i64 = 60;

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

#[derive(Debug, Deserialize)]
pub struct CodexLoginRequest {
    pub name: String,
    pub models: String,
}

#[derive(Debug, Serialize)]
struct CodexLoginResponse {
    state: String,
    user_code: String,
    verification_url: &'static str,
    interval_seconds: i64,
}

#[derive(Debug, Serialize)]
struct CodexPollResponse {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_after_seconds: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct CodexDeviceAuthResponse {
    device_auth_id: String,
    user_code: String,
    #[serde(default)]
    interval: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct CodexDeviceTokenResponse {
    authorization_code: Option<String>,
    code_verifier: Option<String>,
    #[serde(default)]
    error: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct CodexPollRequest {
    state: String,
}

#[derive(Debug, Serialize)]
struct CodexDeviceAuthRequest<'a> {
    client_id: &'a str,
}

#[derive(Debug, Serialize)]
struct CodexDeviceTokenRequest<'a> {
    device_auth_id: &'a str,
    user_code: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodexOAuthCredentials {
    access_token: String,
    refresh_token: String,
    expires_at: i64,
    account_id: String,
}

#[derive(Debug, Deserialize)]
struct CodexTokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    expires_in: i64,
    #[serde(default)]
    id_token: Option<String>,
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
    pub prompt_mode: Option<crate::ai::PromptMode>,
    pub google_search: Option<bool>,
    pub search_engine: Option<String>,
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

#[derive(Debug, Deserialize)]
struct ListLlmModelsRequest {
    #[serde(default)]
    name: String,
    provider: String,
    #[serde(default)]
    base_url: String,
    #[serde(default)]
    api_key: String,
    #[serde(default)]
    project_id: String,
    #[serde(default)]
    location: String,
    #[serde(default)]
    anthropic_version: String,
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

fn is_codex_provider(provider: &str) -> bool {
    matches!(provider, "codex" | "openai-codex")
}

fn provider_type(provider: &str) -> Result<crate::ai::ProviderType, Error> {
    match provider {
        "openai" => Ok(crate::ai::ProviderType::OpenAI),
        "gemini" => Ok(crate::ai::ProviderType::Gemini),
        "vertexai" => Ok(crate::ai::ProviderType::VertexAI),
        "workersai" => Ok(crate::ai::ProviderType::WorkersAI),
        "anthropic" | "claude" => Ok(crate::ai::ProviderType::Anthropic),
        "codex" | "openai-codex" => Ok(crate::ai::ProviderType::Codex),
        _ => Err(Error::Internal(format!(
            "unsupported provider type: {provider}"
        ))),
    }
}

fn normalize_codex_models(models: &str) -> Result<String, Error> {
    let mut normalized = Vec::new();
    for model in models
        .split(',')
        .map(str::trim)
        .filter(|model| !model.is_empty())
    {
        if model.len() > 100 {
            return Err(Error::Internal(
                "Codex model names must contain at most 100 characters".to_string(),
            ));
        }
        if !normalized.iter().any(|existing| existing == model) {
            normalized.push(model.to_string());
        }
    }

    if normalized.is_empty() {
        return Err(Error::Internal(
            "at least one Codex model is required".to_string(),
        ));
    }

    Ok(normalized.join(","))
}

fn codex_credentials_from_value(value: &Value) -> Option<CodexOAuthCredentials> {
    let oauth = value.get("oauth")?;
    let credentials = CodexOAuthCredentials {
        access_token: oauth.get("access_token")?.as_str()?.to_string(),
        refresh_token: oauth.get("refresh_token")?.as_str()?.to_string(),
        expires_at: oauth.get("expires_at")?.as_i64()?,
        account_id: oauth.get("account_id")?.as_str()?.to_string(),
    };
    if credentials.access_token.is_empty()
        || credentials.refresh_token.is_empty()
        || credentials.expires_at <= 0
        || credentials.account_id.is_empty()
    {
        return None;
    }
    Some(credentials)
}

fn codex_credentials_from_features(features: &str) -> Option<CodexOAuthCredentials> {
    serde_json::from_str(features)
        .ok()
        .and_then(|value: Value| codex_credentials_from_value(&value))
}

fn codex_features(credentials: &CodexOAuthCredentials) -> String {
    json!({ "oauth": credentials }).to_string()
}

fn public_codex_features(features: &str) -> String {
    let Some(credentials) = codex_credentials_from_features(features) else {
        return "{}".to_string();
    };

    json!({
        "oauth": {
            "connected": true,
            "account_id": credentials.account_id,
        }
    })
    .to_string()
}

fn redact_llm_provider(mut provider: LlmProvider) -> LlmProvider {
    if is_codex_provider(&provider.provider) {
        provider.api_key.clear();
        provider.features = public_codex_features(&provider.features);
    }
    provider
}

fn random_urlsafe(bytes_len: usize) -> Result<String, Error> {
    let mut bytes = vec![0_u8; bytes_len];
    getrandom::fill(&mut bytes).map_err(|error| Error::Internal(error.to_string()))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn jwt_account_id(token: &str) -> Option<String> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .or_else(|_| URL_SAFE.decode(payload))
        .ok()?;
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    value
        .get("https://api.openai.com/auth")
        .and_then(|auth| auth.get("chatgpt_account_id"))
        .and_then(Value::as_str)
        .filter(|account_id| !account_id.is_empty())
        .map(str::to_string)
}

fn device_interval_seconds(interval: Option<&Value>) -> i64 {
    let interval = interval
        .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok()))
        .unwrap_or(5);
    interval.clamp(1, 60)
}

async fn request_codex_device_auth() -> Result<CodexDeviceAuthResponse, Error> {
    let response = reqwest::Client::new()
        .post(CODEX_DEVICE_USER_CODE_URL)
        .json(&CodexDeviceAuthRequest {
            client_id: CODEX_CLIENT_ID,
        })
        .send()
        .await
        .map_err(|error| Error::Internal(error.to_string()))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| Error::Internal(error.to_string()))?;
    if !status.is_success() {
        return Err(Error::Internal(format!(
            "Codex device login request failed ({}): {}",
            status,
            if body.is_empty() {
                "empty response"
            } else {
                body.as_str()
            }
        )));
    }

    let response = serde_json::from_str::<CodexDeviceAuthResponse>(&body).map_err(|error| {
        Error::Internal(format!("invalid Codex device login response: {error}"))
    })?;
    if response.device_auth_id.trim().is_empty() || response.user_code.trim().is_empty() {
        return Err(Error::Internal(
            "Codex device login response is missing the device id or user code".to_string(),
        ));
    }
    Ok(response)
}

enum CodexDevicePoll {
    Pending,
    Authorized {
        authorization_code: String,
        code_verifier: String,
    },
}

async fn poll_codex_device_auth(
    device_auth_id: &str,
    user_code: &str,
) -> Result<CodexDevicePoll, Error> {
    let response = reqwest::Client::new()
        .post(CODEX_DEVICE_TOKEN_URL)
        .json(&CodexDeviceTokenRequest {
            device_auth_id,
            user_code,
        })
        .send()
        .await
        .map_err(|error| Error::Internal(error.to_string()))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| Error::Internal(error.to_string()))?;

    if status.is_success() {
        let response =
            serde_json::from_str::<CodexDeviceTokenResponse>(&body).map_err(|error| {
                Error::Internal(format!("invalid Codex device token response: {error}"))
            })?;
        let Some(authorization_code) = response
            .authorization_code
            .filter(|value| !value.trim().is_empty())
        else {
            return Err(Error::Internal(
                "Codex device token response is missing an authorization code".to_string(),
            ));
        };
        let Some(code_verifier) = response
            .code_verifier
            .filter(|value| !value.trim().is_empty())
        else {
            return Err(Error::Internal(
                "Codex device token response is missing a code verifier".to_string(),
            ));
        };
        return Ok(CodexDevicePoll::Authorized {
            authorization_code,
            code_verifier,
        });
    }

    let pending = status.as_u16() == 403
        || status.as_u16() == 404
        || serde_json::from_str::<CodexDeviceTokenResponse>(&body)
            .ok()
            .and_then(|response| response.error)
            .and_then(|error| {
                error
                    .as_str()
                    .map(str::to_string)
                    .or_else(|| error.get("code")?.as_str().map(str::to_string))
            })
            .is_some_and(|error| {
                matches!(
                    error.as_str(),
                    "deviceauth_authorization_pending" | "authorization_pending" | "slow_down"
                )
            });
    if pending {
        return Ok(CodexDevicePoll::Pending);
    }

    Err(Error::Internal(format!(
        "Codex device authorization failed ({}): {}",
        status,
        if body.is_empty() {
            "empty response"
        } else {
            body.as_str()
        }
    )))
}

async fn read_codex_token_response(
    response: reqwest::Response,
    operation: &str,
) -> Result<CodexTokenResponse, Error> {
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| Error::Internal(error.to_string()))?;

    if !status.is_success() {
        return Err(Error::Internal(format!(
            "Codex OAuth {operation} failed ({}): {}",
            status,
            if body.is_empty() {
                "empty response"
            } else {
                body.as_str()
            }
        )));
    }

    let token = serde_json::from_str::<CodexTokenResponse>(&body)
        .map_err(|error| Error::Internal(format!("invalid Codex OAuth token response: {error}")))?;
    if token.access_token.trim().is_empty() || token.expires_in <= 0 {
        return Err(Error::Internal(
            "Codex OAuth token response is missing a valid access token or expiry".to_string(),
        ));
    }
    Ok(token)
}

async fn exchange_codex_code(
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<CodexTokenResponse, Error> {
    let response = reqwest::Client::new()
        .post(CODEX_TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", CODEX_CLIENT_ID),
            ("code", code),
            ("code_verifier", verifier),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await
        .map_err(|error| Error::Internal(error.to_string()))?;
    read_codex_token_response(response, "exchange").await
}

async fn refresh_codex_token(refresh_token: &str) -> Result<CodexTokenResponse, Error> {
    let response = reqwest::Client::new()
        .post(CODEX_TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", CODEX_CLIENT_ID),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await
        .map_err(|error| Error::Internal(error.to_string()))?;
    read_codex_token_response(response, "refresh").await
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

impl WorkerState {
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
        origin: &str,
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
            "/mcp/tokens/list" | "/mcp/tokens/create" | "/mcp/tokens/revoke" => {
                if method != "POST" {
                    return Ok(UnifiedResponse::error(
                        405,
                        "Method not allowed".to_string(),
                    ));
                }
                if let Err(e) = self.check_auth(auth_header) {
                    return Ok(UnifiedResponse::error(401, e));
                }
                let response = match path {
                    "/mcp/tokens/list" => self.list_mcp_tokens().await,
                    "/mcp/tokens/create" => self.create_mcp_token(body).await,
                    "/mcp/tokens/revoke" => self.revoke_mcp_token(body).await,
                    _ => unreachable!(),
                }?;
                Ok(UnifiedResponse::json(response))
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
                    let resp = self.route(path, body, origin).await?;
                    Ok(UnifiedResponse::json(resp))
                } else {
                    Err(Error::NotFound)
                }
            }
        }
    }

    pub async fn route(&self, path: &str, body: Vec<u8>, origin: &str) -> Result<Vec<u8>, Error> {
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
            "/llm/models" => self.list_llm_models(body).await,
            "/llm/save" => self.save_llm_provider(body).await,
            "/llm/delete" => self.delete_llm_provider(body).await,
            "/llm/codex/login" => self.start_codex_login(body, origin).await,
            "/llm/codex/poll" => self.poll_codex_login(body).await,
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

            return Ok(b"{}".to_vec());
        }

        self.d1
            .execute(Queries::SaveWord {
                word: &req.word,
                explain: &req.explain,
                word_type: req.r#type.unwrap_or(0),
                example: req.example.as_deref().unwrap_or(""),
            })
            .await?;

        Ok(b"{}".to_vec())
    }

    pub async fn delete_word(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<SingleWordRequest>(&body)?;

        self.d1
            .execute(Queries::DeleteWord { word: &req.word })
            .await?;

        Ok(b"{}".to_vec())
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
        Ok(b"{}".to_vec())
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

        Ok(b"{}".to_vec())
    }

    pub async fn list_mcp_tokens(&self) -> Result<Vec<u8>, Error> {
        let tokens = mcp::list_tokens(self).await.map_err(Error::Internal)?;
        Ok(serde_json::to_vec(&tokens)?)
    }

    pub async fn create_mcp_token(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let request = serde_json::from_slice::<mcp::CreateTokenRequest>(&body)?;
        let token = mcp::create_token(self, request)
            .await
            .map_err(Error::Internal)?;
        Ok(serde_json::to_vec(&token)?)
    }

    pub async fn revoke_mcp_token(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        #[derive(Deserialize)]
        struct Request {
            id: String,
        }

        let request = serde_json::from_slice::<Request>(&body)?;
        mcp::revoke_token(self, &request.id)
            .await
            .map_err(Error::Internal)?;
        Ok(b"{}".to_vec())
    }

    pub async fn start_codex_login(&self, body: Vec<u8>, origin: &str) -> Result<Vec<u8>, Error> {
        let request = serde_json::from_slice::<CodexLoginRequest>(&body)?;
        let name = request.name.trim().to_string();
        if name.is_empty() || name.len() > 100 {
            return Err(Error::Internal(
                "Codex provider name must contain 1 to 100 characters".to_string(),
            ));
        }
        let models = normalize_codex_models(&request.models)?;

        let device = request_codex_device_auth().await?;
        let state = random_urlsafe(32)?;
        let return_url = format!("{origin}/#/docs/config");
        let interval_seconds = device_interval_seconds(device.interval.as_ref());

        self.d1
            .execute(Queries::DeleteExpiredCodexOAuthStates {
                before: Utc::now().timestamp() - CODEX_OAUTH_STATE_TTL_SECONDS,
            })
            .await?;
        self.d1
            .execute(Queries::SaveCodexOAuthState {
                state: &state,
                verifier: "",
                device_auth_id: &device.device_auth_id,
                device_user_code: &device.user_code,
                device_interval_seconds: interval_seconds,
                provider_name: &name,
                models: &models,
                redirect_uri: CODEX_DEVICE_REDIRECT_URI,
                return_url: &return_url,
                created_at: Utc::now().timestamp(),
            })
            .await?;

        Ok(serde_json::to_vec(&CodexLoginResponse {
            state,
            user_code: device.user_code,
            verification_url: CODEX_DEVICE_VERIFICATION_URL,
            interval_seconds,
        })?)
    }

    pub async fn poll_codex_login(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let request = serde_json::from_slice::<CodexPollRequest>(&body)?;
        let state = request.state.trim();
        if state.is_empty() {
            return Err(Error::Internal("Missing Codex OAuth state".to_string()));
        }
        let state_record: Option<CodexOAuthState> = self
            .d1
            .query_first(Queries::GetCodexOAuthState { state })
            .await?;
        let Some(state_record) = state_record else {
            return Err(Error::Internal(
                "Unknown or expired Codex OAuth state".to_string(),
            ));
        };

        if state_record.created_at < Utc::now().timestamp() - CODEX_OAUTH_STATE_TTL_SECONDS {
            self.d1
                .execute(Queries::DeleteCodexOAuthState { state })
                .await?;
            return Err(Error::Internal("Codex OAuth login expired".to_string()));
        }

        if state_record.device_auth_id.is_empty() || state_record.device_user_code.is_empty() {
            return Err(Error::Internal(
                "Codex OAuth state is missing device authorization data".to_string(),
            ));
        }

        match poll_codex_device_auth(&state_record.device_auth_id, &state_record.device_user_code)
            .await?
        {
            CodexDevicePoll::Pending => Ok(serde_json::to_vec(&CodexPollResponse {
                status: "pending",
                retry_after_seconds: Some(state_record.device_interval_seconds.max(1)),
            })?),
            CodexDevicePoll::Authorized {
                authorization_code,
                code_verifier,
            } => {
                // Consume the state before exchanging the one-time authorization code.
                self.d1
                    .execute(Queries::DeleteCodexOAuthState { state })
                    .await?;
                self.finish_codex_login(&state_record, &authorization_code, &code_verifier)
                    .await
            }
        }
    }

    async fn finish_codex_login(
        &self,
        state_record: &CodexOAuthState,
        authorization_code: &str,
        code_verifier: &str,
    ) -> Result<Vec<u8>, Error> {
        let token =
            exchange_codex_code(authorization_code, code_verifier, CODEX_DEVICE_REDIRECT_URI)
                .await?;
        let refresh_token = token
            .refresh_token
            .filter(|refresh_token| !refresh_token.is_empty())
            .ok_or_else(|| {
                Error::Internal(
                    "Codex OAuth token response did not include a refresh token".to_string(),
                )
            })?;
        let account_id = token
            .id_token
            .as_deref()
            .and_then(jwt_account_id)
            .or_else(|| jwt_account_id(&token.access_token))
            .ok_or_else(|| {
                Error::Internal(
                    "Failed to extract the ChatGPT account id from the OAuth token".to_string(),
                )
            })?;

        let credentials = CodexOAuthCredentials {
            access_token: token.access_token,
            refresh_token,
            expires_at: Utc::now().timestamp() + token.expires_in,
            account_id,
        };
        let features = codex_features(&credentials);

        self.d1
            .execute(Queries::SaveLlmProvider {
                name: &state_record.provider_name,
                base_url: CODEX_BASE_URL,
                api_key: "",
                provider: "codex",
                models: &state_record.models,
                features: &features,
            })
            .await?;

        Ok(serde_json::to_vec(&CodexPollResponse {
            status: "connected",
            retry_after_seconds: None,
        })?)
    }

    pub async fn change_priority(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let req = serde_json::from_slice::<ChangeWordPriorityRequest>(&body)?;
        self.d1
            .execute(Queries::ChangePriority {
                priority: req.priority,
                word: &req.word,
            })
            .await?;
        Ok(b"{}".to_vec())
    }

    pub async fn list_llm_providers(&self) -> Result<Vec<u8>, Error> {
        let providers: Vec<LlmProvider> = self
            .d1
            .query_all(crate::d1::Queries::ListLlmProviders)
            .await?;
        let providers = providers
            .into_iter()
            .map(redact_llm_provider)
            .collect::<Vec<_>>();
        Ok(serde_json::to_vec(&providers)?)
    }

    pub async fn list_llm_models(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let request = serde_json::from_slice::<ListLlmModelsRequest>(&body)?;
        let provider_name = request.provider.trim().to_ascii_lowercase();
        provider_type(&provider_name)?;

        let provider = if is_codex_provider(&provider_name) {
            let name = request.name.trim();
            if name.is_empty() {
                return Err(Error::Internal(
                    "Codex model discovery requires an existing connected provider".to_string(),
                ));
            }
            let provider: Option<LlmProvider> = self
                .d1
                .query_first(Queries::GetLlmProviderByName { name })
                .await?;
            let provider = provider.ok_or_else(|| {
                Error::Internal(
                    "Codex model discovery requires an existing connected provider".to_string(),
                )
            })?;
            Self::map_llm_provider_to_config(self.refresh_codex_provider(provider).await?)
        } else {
            let features = if provider_name == "vertexai" {
                let project_id = request.project_id.trim();
                if project_id.is_empty() {
                    return Err(Error::Internal(
                        "VertexAI model discovery requires a project ID".to_string(),
                    ));
                }
                json!({
                    "project_id": project_id,
                    "location": if request.location.trim().is_empty() {
                        "us-central1"
                    } else {
                        request.location.trim()
                    },
                })
            } else if provider_name == "anthropic" || provider_name == "claude" {
                json!({
                    "anthropic_version": if request.anthropic_version.trim().is_empty() {
                        "2023-06-01"
                    } else {
                        request.anthropic_version.trim()
                    },
                })
            } else {
                json!({})
            };

            Self::map_llm_provider_to_config(LlmProvider {
                name: request.name,
                base_url: request.base_url,
                api_key: request.api_key,
                provider: provider_name,
                models: String::new(),
                features: features.to_string(),
            })
        };

        let models = provider
            .list_models()
            .await
            .map_err(|error| Error::Internal(error.to_string()))?;
        if models.is_empty() {
            return Err(Error::Internal(
                "provider returned no usable models".to_string(),
            ));
        }

        Ok(serde_json::to_vec(&models)?)
    }

    pub async fn save_llm_provider(&self, body: Vec<u8>) -> Result<Vec<u8>, Error> {
        let mut req = serde_json::from_slice::<LlmProvider>(&body)?;
        if is_codex_provider(&req.provider) {
            let existing: Option<LlmProvider> = self
                .d1
                .query_first(Queries::GetLlmProviderByName { name: &req.name })
                .await?;

            if codex_credentials_from_features(&req.features).is_none() {
                let Some(existing) = existing else {
                    return Err(Error::Internal(
                        "Sign in with ChatGPT before saving a Codex provider".to_string(),
                    ));
                };
                let Some(_) = codex_credentials_from_features(&existing.features) else {
                    return Err(Error::Internal(
                        "Sign in with ChatGPT before saving a Codex provider".to_string(),
                    ));
                };
                req.features = existing.features;
            }
            req.provider = "codex".to_string();
            req.base_url = CODEX_BASE_URL.to_string();
            req.models = normalize_codex_models(&req.models)?;
            req.api_key.clear();
        }
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
        Ok(b"{}".to_vec())
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
        Ok(b"{}".to_vec())
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
        let p: Option<LlmProvider> = self
            .d1
            .query_first(crate::d1::Queries::GetLlmProviderByName { name })
            .await?;
        let p = p.ok_or_else(|| Error::Internal("custom llm not found".to_string()))?;
        let p = if is_codex_provider(&p.provider) {
            self.refresh_codex_provider(p).await?
        } else {
            p
        };
        Ok(Self::map_llm_provider_to_config(p))
    }

    async fn refresh_codex_provider(
        &self,
        mut provider: LlmProvider,
    ) -> Result<LlmProvider, Error> {
        let current = codex_credentials_from_features(&provider.features).ok_or_else(|| {
            Error::Internal("Codex provider is not connected to ChatGPT".to_string())
        })?;
        let now = Utc::now().timestamp();
        if !current.access_token.is_empty()
            && current.expires_at > now + CODEX_TOKEN_REFRESH_SKEW_SECONDS
        {
            return Ok(provider);
        }

        let token = refresh_codex_token(&current.refresh_token).await?;
        let refresh_token = token
            .refresh_token
            .filter(|refresh_token| !refresh_token.is_empty())
            .unwrap_or(current.refresh_token);
        let account_id = token
            .id_token
            .as_deref()
            .and_then(jwt_account_id)
            .or_else(|| jwt_account_id(&token.access_token))
            .unwrap_or(current.account_id);
        if account_id.is_empty() {
            return Err(Error::Internal(
                "Failed to extract the ChatGPT account id from the refreshed token".to_string(),
            ));
        }

        let credentials = CodexOAuthCredentials {
            access_token: token.access_token,
            refresh_token,
            expires_at: now + token.expires_in,
            account_id,
        };
        provider.features = codex_features(&credentials);
        self.d1
            .execute(Queries::SaveLlmProvider {
                name: &provider.name,
                base_url: CODEX_BASE_URL,
                api_key: "",
                provider: "codex",
                models: &provider.models,
                features: &provider.features,
            })
            .await?;

        Ok(provider)
    }

    fn map_llm_provider_to_config(p: LlmProvider) -> hj_ai::provider::Provider {
        let codex = is_codex_provider(&p.provider);
        let config = crate::ai::ConfigProvider {
            name: p.name.clone(),
            base_url: if codex {
                Some(CODEX_BASE_URL.to_string())
            } else if p.base_url.is_empty() {
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
                "anthropic" | "claude" => Some(crate::ai::ProviderType::Anthropic),
                "codex" | "openai-codex" => Some(crate::ai::ProviderType::Codex),
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

    async fn custom_llm_provider(
        &self,
        custom_llm: Option<&CustomLLM>,
    ) -> Result<(hj_ai::provider::Provider, String), Error> {
        let llm = custom_llm.ok_or_else(|| Error::Internal("custom llm is empty".to_string()))?;

        let mut provider = if llm.name == "workers-ai" {
            self.workers_ai.as_ref().cloned().ok_or_else(|| {
                Error::Internal("workers-ai provider is not available".to_string())
            })?
        } else {
            self.get_llm_provider_by_name(&llm.name).await?
        };

        provider.set_model(&llm.model);
        Ok((provider, llm.model.clone()))
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
        let config = self
            .get_config()
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;

        let response = match req.method.as_str() {
            "custom_llm" => {
                let (provider, model) = self.custom_llm_provider(req.custom_llm.as_ref()).await?;
                Some(
                    crate::ai::explain(
                        &provider,
                        req.google_search.unwrap_or(false),
                        crate::ai::TranslateRequest {
                            model: &model,
                            chars_limit: false,
                            query: &req.word,
                            instruction: req.instruction().as_deref(),
                            prompt_mode: req.prompt_mode,
                            dst_lang: req.dst_lang.as_deref(),
                            search_engine: req.search_engine.as_deref(),
                            google_search_api_key: Some(config.google_search_api_key.clone()),
                            google_search_cx: Some(config.google_search_cx.clone()),
                        },
                    )
                    .await?,
                )
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
        let config = self
            .get_config()
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;

        let stream = match req.method.as_str() {
            "custom_llm" => {
                let (provider, model) = self.custom_llm_provider(req.custom_llm.as_ref()).await?;
                crate::ai::explain_stream(
                    &provider,
                    req.google_search.unwrap_or(false),
                    crate::ai::TranslateRequest {
                        model: &model,
                        chars_limit: false,
                        query: &req.word,
                        instruction: req.instruction().as_deref(),
                        prompt_mode: req.prompt_mode,
                        dst_lang: req.dst_lang.as_deref(),
                        search_engine: req.search_engine.as_deref(),
                        google_search_api_key: Some(config.google_search_api_key.clone()),
                        google_search_cx: Some(config.google_search_cx.clone()),
                    },
                )
                .await?
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_models_are_trimmed_and_deduplicated() {
        assert_eq!(
            normalize_codex_models(" gpt-5.4, gpt-5.3-codex, gpt-5.4 ,, ").unwrap(),
            "gpt-5.4,gpt-5.3-codex"
        );
        assert!(normalize_codex_models(" , ").is_err());
    }

    #[test]
    fn codex_device_interval_accepts_number_or_string() {
        assert_eq!(device_interval_seconds(Some(&Value::from(12))), 12);
        assert_eq!(device_interval_seconds(Some(&Value::from("8"))), 8);
        assert_eq!(device_interval_seconds(Some(&Value::from(0))), 1);
        assert_eq!(device_interval_seconds(Some(&Value::from(120))), 60);
        assert_eq!(device_interval_seconds(None), 5);
    }

    #[test]
    fn codex_account_id_is_read_from_jwt_claim() {
        let payload = URL_SAFE_NO_PAD.encode(
            json!({
                "https://api.openai.com/auth": {
                    "chatgpt_account_id": "acct_test"
                }
            })
            .to_string(),
        );
        let token = format!("header.{payload}.signature");
        assert_eq!(jwt_account_id(&token).as_deref(), Some("acct_test"));
        assert_eq!(jwt_account_id("not-a-jwt"), None);
    }

    #[test]
    fn codex_provider_listing_does_not_expose_oauth_tokens() {
        let credentials = CodexOAuthCredentials {
            access_token: "access-secret".to_string(),
            refresh_token: "refresh-secret".to_string(),
            expires_at: 1,
            account_id: "acct_test".to_string(),
        };
        let public = public_codex_features(&codex_features(&credentials));
        assert_eq!(
            serde_json::from_str::<Value>(&public)
                .unwrap()
                .pointer("/oauth/connected")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert!(!public.contains("access-secret"));
        assert!(!public.contains("refresh-secret"));
    }
}
