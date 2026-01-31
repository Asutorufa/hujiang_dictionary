use std::collections::{HashMap, HashSet};

use chrono::{Duration, Utc};
use frankenstein::client_reqwest;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::ai::OpenAI;

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

#[derive(Clone)]
pub struct RunOpt<T: crate::d1::DB, T2: crate::ai::WorkersAI> {
    pub allow_users: HashSet<i64>,
    pub matainer: i64,
    pub d1: T,
    pub workers_ai: T2,
    pub bot: client_reqwest::Bot,
    pub custom_llms: HashMap<String, OpenAI>,
    pub auth_secret: String,
    pub auth_username: String,
    pub auth_password: String,
    pub auth_token_expiration: i64,
}

impl<T: crate::d1::DB, T2: crate::ai::WorkersAI> RunOpt<T, T2> {
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
}
