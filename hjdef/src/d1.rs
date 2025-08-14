use std::fmt;

use serde::{Deserialize, Serialize};
use value2struct::FromValueVec;

#[derive(Serialize, Deserialize, Debug, FromValueVec, Clone)]
pub struct Word {
    pub word: String,
    pub explain: String,
    add_time: i64,
    update_time: i64,
    reminder_time: i64,
}

#[derive(Debug)]
pub struct D1Error(pub String);

unsafe impl Send for D1Error {}

impl fmt::Display for D1Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for D1Error {}

impl From<&str> for D1Error {
    fn from(s: &str) -> Self {
        D1Error(s.to_string())
    }
}

impl From<reqwest::Error> for D1Error {
    fn from(value: reqwest::Error) -> Self {
        D1Error(value.to_string())
    }
}

impl From<serde_json::Error> for D1Error {
    fn from(value: serde_json::Error) -> Self {
        D1Error(value.to_string())
    }
}

impl From<String> for D1Error {
    fn from(s: String) -> Self {
        D1Error(s)
    }
}

pub trait DB: Send + Sync + Clone + 'static {
    fn save_word(
        &self,
        word: String,
        explain: String,
    ) -> impl Future<Output = Result<(), D1Error>> + Send;
    fn delete_word(&self, word: String) -> impl Future<Output = Result<(), D1Error>> + Send;
    fn random_word(&self) -> impl Future<Output = Result<Word, D1Error>> + Send;
}
