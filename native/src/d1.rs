use hjcommon::d1::{D1Error, DB, SQL, Word};
use log::*;
use serde::{Deserialize, Serialize};

// see: https://developers.cloudflare.com/api/resources/d1/subresources/database
#[derive(Clone)]
pub struct D1 {
    account_id: String,
    database_id: String,
    api_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct QueryBody {
    sql: String,
    params: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawExecResult {
    pub errors: serde_json::Value,
    pub messages: serde_json::Value,
    pub result: Vec<RawResult>,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListDatabaseResult {
    pub errors: serde_json::Value,
    pub messages: serde_json::Value,
    pub result: Vec<ListDatabaseResultItem>,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListDatabaseResultItem {
    name: String,
    uuid: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub code: Option<i32>,
    pub message: Option<String>,
    pub documentation_url: Option<String>,
    pub source: Option<Source>,
}

impl ToString for Message {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    pub pointer: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub changed_db: bool,
    pub changes: i32,
    pub duration: f64,
    pub last_row_id: i32,
    pub rows_read: i32,
    pub rows_written: i32,
    pub served_by_primary: bool,
    pub served_by_region: String,
    pub size_after: i32,
    pub timings: Timings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Timings {
    pub sql_duration_ms: f64,
}

pub struct Empty {}

impl From<Vec<serde_json::Value>> for Empty {
    fn from(_: Vec<serde_json::Value>) -> Self {
        Empty {}
    }
}

pub enum Database {
    UUID(String),
    Name(String),
}

impl D1 {
    pub async fn new(account_id: &str, api_token: &str, database: Database) -> D1 {
        let mut d1 = D1 {
            account_id: account_id.to_string(),
            database_id: "".to_string(),
            api_token: api_token.to_string(),
        };

        d1.database_id = match database {
            Database::Name(v) => d1.get_database_id(&v).await.unwrap_or("".to_string()),
            Database::UUID(v) => v,
        };

        d1
    }

    pub async fn get_database_id(&self, database_name: &str) -> Result<String, D1Error> {
        let r = reqwest::Client::builder()
            .build()?
            .get(format!("/accounts/{}/d1/database", self.account_id,))
            .query(&vec![("name", database_name)])
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;

        let lr = r.json::<ListDatabaseResult>().await?;

        if lr.result.len() == 0 {
            return Err(D1Error(
                format!("database {} not found", database_name).to_string(),
            ));
        }

        Ok(lr.result[0].name.to_owned())
    }

    pub async fn exec_sql<T: From<Vec<serde_json::Value>>>(
        &self,
        sql: SQL<'_>,
    ) -> Result<Vec<T>, D1Error> {
        self.raw(sql.sql(), sql.params()).await
    }

    pub async fn raw<T: From<Vec<serde_json::Value>>>(
        &self,
        sql: &str,
        params: Vec<String>,
    ) -> Result<Vec<T>, D1Error> {
        if self.database_id == "" {
            return Err(D1Error("database_id is empty".to_string()));
        }

        if self.account_id == "" {
            return Err(D1Error("account_id is empty".to_string()));
        }

        if self.api_token == "" {
            return Err(D1Error("api_token is empty".to_string()));
        }

        let pre_log = format!("exec sql: [{}] args: {:?}", sql, params);

        let body = serde_json::to_string(&QueryBody {
            sql: sql.to_string(),
            params: params,
        })?;

        let r = reqwest::Client::builder()
            .build()?
            .post(format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/d1/database/{}/raw",
                self.account_id, self.database_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .body(body)
            .send()
            .await?;

        if r.status() != 200 {
            return Err(D1Error::from(r.text().await?));
        }

        let result = r.json::<RawExecResult>().await?;

        info!("{} messages: {}", pre_log, result.messages);

        if !result.success {
            return Err(D1Error::from(format!("query failed: {}", result.errors)));
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

impl DB for D1 {
    async fn create_table(&self) -> Result<(), D1Error> {
        self.exec_sql::<Empty>(SQL::CreateTable).await?;
        Ok(())
    }

    async fn save_word(&self, word: &str, explain: &str) -> Result<(), D1Error> {
        self.exec_sql::<Empty>(SQL::SaveWord(word, explain)).await?;
        Ok(())
    }

    async fn delete_word(&self, word: &str) -> Result<(), D1Error> {
        self.exec_sql::<Empty>(SQL::DeleteWord(word)).await?;
        Ok(())
    }

    async fn random_word(&self) -> Result<Word, D1Error> {
        let words = match self.exec_sql::<Word>(SQL::RandomNotRemind).await {
            Ok(v) if !v.is_empty() => v,
            _ => self.exec_sql::<Word>(SQL::Random).await?,
        };

        if words.len() == 0 {
            return Err(D1Error::from("no word found"));
        }

        let v = words[0].clone();

        match self
            .exec_sql::<Empty>(SQL::UpdateRemindTime(v.word.as_ref()))
            .await
        {
            Err(e) => error!("update reminder_time error: {}", e),
            _ => {}
        }

        Ok(v)
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use hjcommon::d1::DB;
    use serde::{Deserialize, Serialize};

    use crate::d1::{D1, Word};

    #[derive(Serialize, Deserialize)]
    struct Auth {
        account_id: String,
        database_id: String,
        api_token: String,
    }

    #[tokio::test]
    async fn test() {
        let auth_json = fs::read_to_string("src/.api.json").unwrap();

        let auth = serde_json::from_str::<Auth>(&auth_json).unwrap();

        let d1 = D1::new(
            auth.account_id.as_str(),
            auth.api_token.as_str(),
            crate::d1::Database::UUID(auth.database_id),
        )
        .await;

        println!("random: {:?}", d1.random_word().await.unwrap());

        let result = d1
            .raw::<Word>("select * from words limit 10", vec![])
            .await
            .unwrap();

        println!("{:?}", result);
    }
}
