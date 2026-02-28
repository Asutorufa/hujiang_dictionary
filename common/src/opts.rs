use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use frankenstein::client_reqwest;

use d1_orm::DatabaseExecutor;

#[derive(Clone)]
pub struct RunOpt<T: DatabaseExecutor, T2: crate::ai::Translator> {
    pub allow_users: Arc<HashSet<i64>>,
    pub matainer: i64,
    pub d1: T,
    pub translator: T2,
    pub workers_ai: Option<hj_ai::provider::Provider>,
    pub bot: client_reqwest::Bot,
    pub custom_llms: HashMap<String, hj_ai::provider::Provider>,
    pub auth_secret: String,
    pub auth_username: String,
    pub auth_password: String,
    pub auth_token_expiration: i64,
}
