use async_trait::async_trait;
use d1_orm::{DatabaseExecutor, DatabaseValue, Error, Query};
use log::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

// see: https://developers.cloudflare.com/api/resources/d1/subresources/database
#[derive(Clone)]
pub struct D1 {
    pub(crate) account_id: String,
    pub(crate) database_id: String,
    pub(crate) api_token: String,
    pub(crate) client: reqwest::Client,
}

#[derive(Serialize, Debug)]
pub struct QueryBody<'a> {
    pub sql: &'a str,
    #[serde(serialize_with = "serialize_params")]
    pub params: Vec<DatabaseValue>,
}

fn serialize_params<S>(params: &Vec<DatabaseValue>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(params.len()))?;
    for p in params {
        match p {
            DatabaseValue::Null => seq.serialize_element(&serde_json::Value::Null)?,
            DatabaseValue::Int(i) => seq.serialize_element(i)?,
            DatabaseValue::UInt(u) => seq.serialize_element(u)?,
            DatabaseValue::Real(f) => seq.serialize_element(f)?,
            DatabaseValue::Text(s) => seq.serialize_element(s)?,
            DatabaseValue::Blob(b) => seq.serialize_element(b)?,
            DatabaseValue::Bool(b) => seq.serialize_element(b)?,
        }
    }
    seq.end()
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

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
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
        let client = reqwest::Client::builder().build().unwrap();

        let mut d1 = D1 {
            account_id: account_id.to_string(),
            database_id: "".to_string(),
            api_token: api_token.to_string(),
            client,
        };

        d1.database_id = match database {
            Database::Name(v) => d1.get_database_id(&v).await.unwrap_or("".to_string()),
            Database::UUID(v) => v,
        };

        d1
    }

    pub async fn get_database_id(&self, database_name: &str) -> Result<String, Error> {
        let r = self
            .client
            .get(format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/d1/database",
                self.account_id
            ))
            .query(&vec![("name", database_name)])
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;

        let lr = r
            .json::<ListDatabaseResult>()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;

        if lr.result.is_empty() {
            return Err(Error::Other(
                format!("database {} not found", database_name).to_string(),
            ));
        }

        Ok(lr.result[0].name.to_owned())
    }

    pub async fn request<T>(
        &self,
        path: &str,
        sql: &str,
        params: Vec<DatabaseValue>,
    ) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        if self.database_id.is_empty() {
            return Err(Error::Other("database_id is empty".to_string()));
        }

        if self.account_id.is_empty() {
            return Err(Error::Other("account_id is empty".to_string()));
        }

        if self.api_token.is_empty() {
            return Err(Error::Other("api_token is empty".to_string()));
        }

        info!("exec sql: [{}] args: {:?}", sql, params);

        let body = serde_json::to_string(&QueryBody { sql, params })
            .map_err(|e| Error::Other(e.to_string()))?;

        let r = self
            .client
            .post(format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/d1/database/{}/{}",
                self.account_id, self.database_id, path
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .body(body)
            .send()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;

        if r.status() != 200 {
            return Err(Error::Other(
                r.text().await.map_err(|e| Error::Other(e.to_string()))?,
            ));
        }

        let result = r
            .json::<T>()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;

        Ok(result)
    }
}

#[async_trait(?Send)]
impl DatabaseExecutor for D1 {
    async fn execute<Q>(&self, query: Q) -> Result<(), Error>
    where
        Q: Query,
    {
        let (sql, params) = query.build()?;
        self.request::<RawQueryResult>("query", &sql, params)
            .await?;
        Ok(())
    }

    async fn query_all<T, Q>(&self, query: Q) -> Result<Vec<T>, Error>
    where
        T: DeserializeOwned,
        Q: Query,
    {
        let (sql, params) = query.build()?;
        let result = self
            .request::<RawQueryResult>("query", &sql, params)
            .await?;

        if !result.success {
            return Err(Error::Other(format!("query failed: {}", result.errors)));
        }

        let mut rs: Vec<T> = vec![];

        for v in result.result {
            for rv in v.results {
                rs.push(serde_json::from_value(rv).map_err(|e| Error::Other(e.to_string()))?);
            }
        }

        Ok(rs)
    }

    async fn query_first<T, Q>(&self, query: Q) -> Result<Option<T>, Error>
    where
        T: DeserializeOwned,
        Q: Query,
    {
        let mut rows = self.query_all::<T, Q>(query).await?;
        if rows.is_empty() {
            Ok(None)
        } else {
            Ok(Some(rows.remove(0)))
        }
    }

    async fn execute_batch<Q>(&self, queries: Vec<Q>) -> Result<(), Error>
    where
        Q: Query,
    {
        let futures = queries.into_iter().map(|q| self.execute(q));
        let results = futures::future::join_all(futures).await;
        for result in results {
            result?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    // use crate::d1::D1;
    // use std::fs;
    // use serde::{Deserialize, Serialize};
    // use hjcommon::d1::DatabaseExecutor;

    // #[derive(Serialize, Deserialize)]
    // struct Auth {
    //     account_id: String,
    //     database_id: String,
    //     api_token: String,
    // }

    // async fn new_d1() -> D1 {
    //     let auth_json = fs::read_to_string("src/.api.json").unwrap();
    //
    //     let auth = serde_json::from_str::<Auth>(&auth_json).unwrap();
    //
    //     D1::new(
    //         auth.account_id.as_str(),
    //         auth.api_token.as_str(),
    //         crate::d1::Database::UUID(auth.database_id),
    //     )
    //     .await
    // }

    /*
    #[tokio::test]
    async fn test_list_word() {
        let d1 = new_d1().await;
        // Need to use executor.query(...) now
    }
    */
}
