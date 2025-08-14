use hjdef::d1::{D1Error, DB};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use wasm_bindgen::JsValue;
use worker::{D1Database, Env};

pub struct MyError(worker::Error);

impl From<worker::Error> for MyError {
    fn from(err: worker::Error) -> Self {
        MyError(err)
    }
}

impl From<MyError> for D1Error {
    fn from(err: MyError) -> Self {
        D1Error(err.0.to_string())
    }
}

pub struct WasmD1 {
    env: Env,
    binding: String,
    d1: Arc<D1Database>,
}

unsafe impl Send for WasmD1 {}

impl WasmD1 {
    pub async fn new(env: Env, binding: &str) -> Result<WasmD1, worker::Error> {
        let d1 = env.d1(binding)?;

        Ok(WasmD1 {
            d1: Arc::new(d1),
            env,
            binding: binding.to_string(),
        })
    }

    pub async fn exec(&self, sql: &str, args: Vec<String>) -> Result<(), D1Error> {
        let mut a = Vec::<JsValue>::new();
        for arg in args {
            a.push(JsValue::from(arg));
        }

        let result = self
            .d1
            .prepare(sql)
            .bind(&a)
            .map_err(|v| MyError(v))?
            .run()
            .await
            .map_err(|v| MyError(v))?;

        if !result.error().is_none() {
            return Err(D1Error(result.error().unwrap().to_string()));
        }

        Ok(())
    }

    pub async fn save_word(&self, word: String, explain: String) -> Result<(), D1Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let d1 = self.env.d1(&self.binding).map_err(|v| MyError(v))?;

        let result =  d1
            .prepare(
                "INSERT INTO words (word, explain, add_time, update_time) VALUES (?, ?, ?, ?) ON CONFLICT(word) DO UPDATE SET explain = ?, update_time = ?",
            )
            .bind(&[word.into(), explain.clone().into(), now.clone().into(), now.clone().into(), explain.into(), now.into()])
           .map_err(|v| MyError(v))?.run().await.map_err(|v| MyError(v))?;

        if !result.error().is_none() {
            return Err(D1Error(result.error().unwrap().to_string()));
        }

        Ok(())
    }

    pub async fn delete_word(&self, word: String) -> Result<(), D1Error> {
        let result = self
            .d1
            .prepare("DELETE FROM words WHERE word = ?")
            .bind(&[word.into()])
            .map_err(|v| MyError(v))?
            .run()
            .await
            .map_err(|v| MyError(v))?;

        if !result.error().is_none() {
            return Err(D1Error(result.error().unwrap().to_string()));
        }

        Ok(())
    }

    pub async fn random_word(&self) -> Result<hjdef::d1::Word, D1Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let words = match self
            .d1
            .prepare("SELECT * FROM words WHERE reminder_time <= ? ORDER BY RANDOM() LIMIT 1")
            .bind(&[now.clone().into()])
            .map_err(|v| MyError(v))?
            .first::<hjdef::d1::Word>(None)
            .await
        {
            Ok(v) if !v.is_none() => v.unwrap(),
            _ => self
                .d1
                .prepare("SELECT * FROM words ORDER BY RANDOM() LIMIT 1")
                .first::<hjdef::d1::Word>(None)
                .await
                .map_err(|v| MyError(v))?
                .ok_or(D1Error("can't get random word".to_string()))?,
        };

        let v = words.clone();

        let result = self
            .d1
            .prepare("UPDATE words SET reminder_time = ? WHERE word = ?")
            .bind(&[now.clone().into(), v.word.clone().into()])
            .map_err(|v| MyError(v))?
            .run()
            .await
            .map_err(|v| MyError(v))?;

        if !result.error().is_none() {
            return Err(D1Error(result.error().unwrap().to_string()));
        }

        Ok(v)
    }
}

impl Clone for WasmD1 {
    fn clone(&self) -> Self {
        WasmD1 {
            d1: self.d1.clone(),
            binding: self.binding.clone(),
            env: self.env.clone(),
        }
    }
}

// see: https://github.com/teloxide/teloxide/issues/1210
impl DB for WasmD1 {
    async fn delete_word(&self, _word: String) -> Result<(), D1Error> {
        Err(D1Error("teloxide not support wasm".to_string()))
    }

    async fn random_word(&self) -> Result<hjdef::d1::Word, D1Error> {
        Err(D1Error("teloxide not support wasm".to_string()))
    }

    async fn save_word(&self, _word: String, _explain: String) -> Result<(), D1Error> {
        Err(D1Error("teloxide not support wasm".to_string()))
    }
}
