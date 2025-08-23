use hjcommon::d1::{DBv2, Error, SQL};
use log::*;
use serde::{Deserialize, Serialize};

// see: https://developers.cloudflare.com/api/resources/d1/subresources/database
#[derive(Clone)]
pub struct D1 {
    pub(crate) account_id: String,
    pub(crate) database_id: String,
    pub(crate) api_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryBody {
    pub sql: String,
    pub params: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawQueryResult {
    pub errors: serde_json::Value,
    pub messages: serde_json::Value,
    pub result: Vec<QueryResult>,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub meta: Meta,
    pub results: Vec<serde_json::Value>,
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

    pub async fn get_database_id(&self, database_name: &str) -> Result<String, Error> {
        let r = reqwest::Client::builder()
            .build()?
            .get(format!("/accounts/{}/d1/database", self.account_id,))
            .query(&vec![("name", database_name)])
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;

        let lr = r.json::<ListDatabaseResult>().await?;

        if lr.result.len() == 0 {
            return Err(Error(
                format!("database {} not found", database_name).to_string(),
            ));
        }

        Ok(lr.result[0].name.to_owned())
    }

    pub async fn post<T>(&self, path: &str, sql: SQL<'_>) -> Result<T, Error>
    where
        T: for<'a> Deserialize<'a>,
    {
        if self.database_id == "" {
            return Err(Error("database_id is empty".to_string()));
        }

        if self.account_id == "" {
            return Err(Error("account_id is empty".to_string()));
        }

        if self.api_token == "" {
            return Err(Error("api_token is empty".to_string()));
        }

        info!(
            "exec sql: [{}] args: {:?}",
            sql.sql(),
            sql.params::<String>()
        );

        let body = serde_json::to_string(&QueryBody {
            sql: sql.sql().to_string(),
            params: sql.params::<String>(),
        })?;

        let r = reqwest::Client::builder()
            .build()?
            .post(format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/d1/database/{}/{}",
                self.account_id, self.database_id, path
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .body(body)
            .send()
            .await?;

        if r.status() != 200 {
            return Err(Error::from(r.text().await?));
        }

        let result = r.json::<T>().await?;

        Ok(result)
    }

    pub async fn query<T>(&self, sql: SQL<'_>) -> Result<Vec<T>, Error>
    where
        T: for<'a> Deserialize<'a>,
    {
        let result = self.post::<RawQueryResult>("query", sql).await?;

        info!("result messages: {}", result.messages);

        if !result.success {
            return Err(Error::from(format!("query failed: {}", result.errors)));
        }

        let mut rs: Vec<T> = vec![];

        for v in result.result {
            for rv in v.results {
                rs.push(serde_json::from_value(rv)?);
            }
        }

        Ok(rs)
    }
}

impl DBv2 for D1 {
    async fn exec<T>(&self, sql: SQL<'_>) -> Result<Vec<T>, Error>
    where
        T: for<'a> Deserialize<'a>,
    {
        self.query::<T>(sql).await
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use hjcommon::d1::{DBv2, SQL, Word};
    use serde::{Deserialize, Serialize};

    use crate::d1::D1;

    #[derive(Serialize, Deserialize)]
    struct Auth {
        account_id: String,
        database_id: String,
        api_token: String,
    }

    async fn new_d1() -> D1 {
        let auth_json = fs::read_to_string("src/.api.json").unwrap();

        let auth = serde_json::from_str::<Auth>(&auth_json).unwrap();

        let d1 = D1::new(
            auth.account_id.as_str(),
            auth.api_token.as_str(),
            crate::d1::Database::UUID(auth.database_id),
        )
        .await;

        d1
    }

    #[tokio::test]
    async fn test() {
        let d1 = new_d1().await;

        println!("random: {:?}", d1.random_word().await.unwrap());

        let result = d1.raw::<Word>(SQL::ListWord(10, 1)).await.unwrap();

        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_list_word() {
        let d1 = new_d1().await;

        println!("{:?}", d1.list_word(10, 1).await.unwrap());
    }
}
