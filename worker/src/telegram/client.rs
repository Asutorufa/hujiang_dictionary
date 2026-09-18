use async_trait::async_trait;
use frankenstein::AsyncTelegramApi;
use frankenstein::response::ErrorResponse;
use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Clone)]
pub struct TelegramBotClient {
    api_url: String,
    client: Client,
}

impl TelegramBotClient {
    pub fn new(token: &str) -> Self {
        Self {
            api_url: format!("{}{}", frankenstein::BASE_API_URL, token),
            client: Client::new(),
        }
    }
}

#[derive(Debug)]
pub struct TelegramClientError(String);

impl TelegramClientError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for TelegramClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for TelegramClientError {}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl AsyncTelegramApi for TelegramBotClient {
    type Error = TelegramClientError;

    async fn request<Params, Output>(
        &self,
        method: &str,
        params: Option<Params>,
    ) -> Result<Output, Self::Error>
    where
        Params: Serialize + std::fmt::Debug + Send,
        Output: DeserializeOwned,
    {
        let mut request = self
            .client
            .post(format!("{}/{}", self.api_url, method))
            .header("Content-Type", "application/json");

        if let Some(params) = params {
            request = request.json(&params);
        }

        let response = request
            .send()
            .await
            .map_err(|error| TelegramClientError::new(error.to_string()))?;
        let body = response
            .text()
            .await
            .map_err(|error| TelegramClientError::new(error.to_string()))?;
        let value: serde_json::Value = serde_json::from_str(&body).map_err(|error| {
            TelegramClientError::new(format!("invalid Telegram response: {error}"))
        })?;

        if !value
            .get("ok")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        {
            let error = serde_json::from_value::<ErrorResponse>(value).map_or_else(
                |_| "Telegram API returned an unsuccessful response".to_string(),
                |error| {
                    format!(
                        "Telegram API error {}: {}",
                        error.error_code, error.description
                    )
                },
            );
            return Err(TelegramClientError::new(error));
        }

        serde_json::from_value(value)
            .map_err(|error| TelegramClientError::new(format!("invalid Telegram payload: {error}")))
    }

    async fn request_with_form_data<Params, Output>(
        &self,
        _method: &str,
        _params: Params,
        _files: Vec<(&str, PathBuf)>,
    ) -> Result<Output, Self::Error>
    where
        Params: Serialize + std::fmt::Debug + Send,
        Output: DeserializeOwned,
    {
        Err(TelegramClientError::new(
            "Telegram multipart requests are not supported by the Workers client",
        ))
    }
}

impl From<TelegramClientError> for crate::tg::Error {
    fn from(error: TelegramClientError) -> Self {
        Self(error.to_string())
    }
}
