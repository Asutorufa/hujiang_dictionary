use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use worker::{D1Database, Env};

#[derive(Debug)]
pub struct D1Error(String);

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

impl From<String> for D1Error {
    fn from(s: String) -> Self {
        D1Error(s)
    }
}
impl From<worker::Error> for D1Error {
    fn from(s: worker::Error) -> Self {
        D1Error(s.to_string())
    }
}

pub struct Wasm {
    d1: D1Database,
}

impl Wasm {
    pub async fn new(env: Env, binding: &str) -> Result<Wasm, worker::Error> {
        let d1 = env.d1(binding)?;

        Ok(Wasm { d1 })
    }

    pub async fn save_word(&self, word: String, explain: String) -> Result<(), D1Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let result =  self.d1
            .prepare(
                "INSERT INTO words (word, explain, add_time, update_time) VALUES (?, ?, ?, ?) ON CONFLICT(word) DO UPDATE SET explain = ?, update_time = ?",
            )
            .bind(&[word.into(), explain.clone().into(), now.clone().into(), now.clone().into(), explain.into(), now.into()])
            ?.run().await?;

        if !result.error().is_none() {
            return Err(D1Error(result.error().unwrap().to_string()));
        }

        Ok(())
    }

    pub async fn delete_word(&self, word: String) -> Result<(), D1Error> {
        let result = self
            .d1
            .prepare("DELETE FROM words WHERE word = ?")
            .bind(&[word.into()])?
            .run()
            .await?;

        if !result.error().is_none() {
            return Err(D1Error(result.error().unwrap().to_string()));
        }

        Ok(())
    }

    pub async fn random_word(&self) -> Result<hj_rust::d1::Word, D1Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let words = match self
            .d1
            .prepare("SELECT * FROM words WHERE reminder_time <= ? ORDER BY RANDOM() LIMIT 1")
            .bind(&[now.clone().into()])?
            .first::<hj_rust::d1::Word>(None)
            .await
        {
            Ok(v) if !v.is_none() => v.unwrap(),
            _ => self
                .d1
                .prepare("SELECT * FROM words ORDER BY RANDOM() LIMIT 1")
                .first::<hj_rust::d1::Word>(None)
                .await?
                .ok_or(D1Error("can't get random word".to_string()))?,
        };

        let v = words.clone();

        let result = self
            .d1
            .prepare("UPDATE words SET reminder_time = ? WHERE word = ?")
            .bind(&[now.clone().into(), v.word.clone().into()])?
            .run()
            .await?;

        if !result.error().is_none() {
            return Err(D1Error(result.error().unwrap().to_string()));
        }

        Ok(v)
    }
}
