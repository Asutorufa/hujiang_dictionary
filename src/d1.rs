use std::fmt;

use serde::{Deserialize, Serialize};

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
pub struct QueryResult {
    pub errors: Vec<Message>,
    pub messages: Vec<Message>,
    pub result: Vec<ResultItem>,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub code: i32,
    pub message: String,
    pub documentation_url: String,
    pub source: Source,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    pub pointer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultItem {
    pub meta: Meta,
    pub results: Vec<serde_json::Value>,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub changed_db: bool,
    pub changes: i32,
    pub duration: i32,
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
    pub sql_duration_ms: i32,
}

impl D1 {
    pub async fn query(
        &self,
        sql: String,
        params: Vec<String>,
    ) -> Result<Vec<ResultItem>, D1Error> {
        /*
                     curl https://api.cloudflare.com/client/v4/accounts/$ACCOUNT_ID/d1/database/$DATABASE_ID/raw \
                 -H 'Content-Type: application/json' \
                 -H "X-Auth-Email: $CLOUDFLARE_EMAIL" \
                 -H "X-Auth-Key: $CLOUDFLARE_API_KEY" \
                 -d '{
                       "sql": "SELECT * FROM myTable WHERE field = ? OR field = ?;",
                       "params": [
                         "firstParam",
                         "secondParam"
                       ]
                     }'


         {
           "errors": [
             {
               "code": 1000,
               "message": "message",
               "documentation_url": "documentation_url",
               "source": {
                 "pointer": "pointer"
               }
             }
           ],
           "messages": [
             {
               "code": 1000,
               "message": "message",
               "documentation_url": "documentation_url",
               "source": {
                 "pointer": "pointer"
               }
             }
           ],
           "result": [
             {
               "meta": {
                 "changed_db": true,
                 "changes": 0,
                 "duration": 0,
                 "last_row_id": 0,
                 "rows_read": 0,
                 "rows_written": 0,
                 "served_by_primary": true,
                 "served_by_region": "EEUR",
                 "size_after": 0,
                 "timings": {
                   "sql_duration_ms": 0
                 }
               },
               "results": [
                 {}
               ],
               "success": true
             }
           ],
           "success": true
         }
        */

        let body = serde_json::to_string(&QueryBody { sql, params })?;

        let r = reqwest::Client::builder()
            .build()?
            .post(format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/d1/database/{}/raw",
                self.account_id, self.database_id
            ))
            .header("Authorization", self.api_token.clone())
            .body(body)
            .send()
            .await?;

        let result = serde_json::from_str::<QueryResult>(r.text().await?.as_str())?;

        if !result.success {
            return Err(D1Error::from(format!("query failed: {:?}", result.errors)));
        }

        Ok(result.result)
    }
}
