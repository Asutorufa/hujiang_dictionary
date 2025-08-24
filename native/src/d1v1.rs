use hjcommon::d1::{DB, Empty, Error, SQL, Word};
use log::*;
use serde::{Deserialize, Serialize};

use crate::d1::{D1, Meta};

#[derive(Debug, Serialize, Deserialize)]
pub struct RawExecResult {
    pub errors: serde_json::Value,
    pub messages: serde_json::Value,
    pub result: Vec<RawResult>,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawResult {
    pub meta: Meta,
    pub results: RawResults,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawResults {
    columns: Vec<String>,
    rows: Vec<Vec<serde_json::Value>>,
}

impl DB for D1 {
    async fn create_table(&self) -> Result<(), Error> {
        self.raw::<Empty>(SQL::CreateTable).await?;
        Ok(())
    }

    async fn save_word(&self, word: &str, explain: &str) -> Result<(), Error> {
        self.raw::<Empty>(SQL::SaveWord(word, explain)).await?;
        Ok(())
    }

    async fn delete_word(&self, word: &str) -> Result<(), Error> {
        self.raw::<Empty>(SQL::DeleteWord(word)).await?;
        Ok(())
    }

    async fn random_word(&self) -> Result<Word, Error> {
        let words = match self.raw::<Word>(SQL::RandomNotRemind).await {
            Ok(v) if !v.is_empty() => v,
            _ => self.raw::<Word>(SQL::Random).await?,
        };

        if words.len() == 0 {
            return Err(Error::from("no word found"));
        }

        let v = words[0].clone();

        match self
            .raw::<Empty>(SQL::UpdateRemindTime(v.word.as_ref()))
            .await
        {
            Err(e) => error!("update reminder_time error: {}", e),
            _ => {}
        }

        Ok(v)
    }

    async fn list_word(
        &self,
        page_size: u64,
        page_number: u64,
        order_by: &str,
    ) -> Result<Vec<Word>, Error> {
        self.raw::<Word>(SQL::ListWord(page_size, page_number, order_by))
            .await
    }
}

impl D1 {
    pub async fn raw<T: From<Vec<serde_json::Value>>>(
        &self,
        sql: SQL<'_>,
    ) -> Result<Vec<T>, Error> {
        let result = self.post::<RawExecResult>("raw", sql).await?;

        info!("result messages: {}", result.messages);

        if !result.success {
            return Err(Error::from(format!("query failed: {}", result.errors)));
        }

        let mut rs: Vec<T> = vec![];

        for v in result.result {
            for rv in v.results.rows {
                rs.push(T::from(rv));
            }
        }

        Ok(rs)
    }
}
