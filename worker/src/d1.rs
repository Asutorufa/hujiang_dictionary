use hjcommon::d1::{D1Error, DB, SQL};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Empty {}

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

    pub async fn exec<T>(&self, sql: SQL) -> Result<Vec<T>, D1Error>
    where
        T: for<'a> Deserialize<'a>,
    {
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

        Ok(result.results::<T>().unwrap())
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
    async fn create_table(&self) -> Result<(), D1Error> {
        self.exec::<Empty>(SQL::CreateTable).await?;
        Ok(())
    }

    async fn delete_word(&self, word: String) -> Result<(), D1Error> {
        let sql = SQL::DeleteWord(word.clone());
        self.exec::<Empty>(sql).await?;
        Ok(())
    }

    async fn random_word(&self) -> Result<hjcommon::d1::Word, D1Error> {
        let words = match self.exec::<hjcommon::d1::Word>(SQL::RandomNotRemind).await {
            Ok(v) if !v.is_empty() => v[0].clone(),
            _ => self
                .exec::<hjcommon::d1::Word>(SQL::Random)
                .await?
                .first()
                .ok_or(D1Error("no word found".to_string()))?
                .clone(),
        };

        let update_sql = SQL::UpdateRemindTime(words.word.clone());

        self.exec::<Empty>(update_sql).await?;

        Ok(words)
    }

    async fn save_word(&self, word: String, explain: String) -> Result<(), D1Error> {
        let sql = SQL::SaveWord(word.clone(), explain.clone());
        self.exec::<Empty>(sql).await?;
        Ok(())
    }
}
