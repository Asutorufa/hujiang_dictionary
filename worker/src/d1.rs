use hjdef::d1::{D1Error, DB, SQL};
use std::sync::Arc;
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
    d1: Option<Arc<D1Database>>,
}

unsafe impl Send for WasmD1 {}

impl WasmD1 {
    pub async fn new(env: Env, binding: &str) -> WasmD1 {
        WasmD1 {
            d1: match env.d1(binding) {
                Ok(v) => Some(Arc::new(v)),
                Err(_) => None,
            },
        }
    }

    fn get_d1(&self) -> Result<Arc<D1Database>, D1Error> {
        match self.d1.clone() {
            Some(v) => Ok(v),
            None => Err(D1Error("d1 is not initialized".to_string())),
        }
    }

    pub async fn exec(&self, sql: &str, args: Vec<String>) -> Result<(), D1Error> {
        let mut a = Vec::<JsValue>::new();
        for arg in args {
            a.push(JsValue::from(arg));
        }

        let result = self
            .get_d1()?
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
        let sql = SQL::SaveWord(word.clone(), explain.clone());

        let result = self
            .get_d1()?
            .prepare(sql.sql())
            .bind(&sql.params())
            .map_err(|v| MyError(v))?
            .run()
            .await
            .map_err(|v| MyError(v))?;

        if !result.error().is_none() {
            return Err(D1Error(result.error().unwrap().to_string()));
        }

        Ok(())
    }

    pub async fn delete_word(&self, word: String) -> Result<(), D1Error> {
        let sql = SQL::DeleteWord(word.clone());

        let result = self
            .get_d1()?
            .prepare(sql.sql())
            .bind(&sql.params())
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
        let words = match self
            .get_d1()?
            .prepare(SQL::RandomNotRemind.sql())
            .first::<hjdef::d1::Word>(None)
            .await
        {
            Ok(v) if !v.is_none() => v.unwrap(),
            _ => self
                .get_d1()?
                .prepare(SQL::Random.sql())
                .first::<hjdef::d1::Word>(None)
                .await
                .map_err(|v| MyError(v))?
                .ok_or(D1Error("can't get random word".to_string()))?,
        };

        let v = words.clone();

        let update_sql = SQL::UpdateRemindTime(v.word.clone());

        let result = self
            .get_d1()?
            .prepare(update_sql.sql())
            .bind(&update_sql.params())
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
        }
    }
}

impl DB for WasmD1 {
    async fn delete_word(&self, word: String) -> Result<(), D1Error> {
        self.delete_word(word).await
    }

    async fn random_word(&self) -> Result<hjdef::d1::Word, D1Error> {
        self.random_word().await
    }

    async fn save_word(&self, word: String, explain: String) -> Result<(), D1Error> {
        self.save_word(word, explain).await
    }
}
