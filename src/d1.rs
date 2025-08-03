use serde::{Deserialize, Serialize};
use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};
use value2struct::FromValueVec;

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
pub struct QueryResult {
    pub errors: serde_json::Value,
    pub messages: serde_json::Value,
    pub result: Option<Vec<ResultItem>>,
    pub success: bool,
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
pub struct ResultItem {
    pub meta: Meta,
    pub results: ItemResult,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemResult {
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

#[derive(Serialize, Deserialize, Debug, FromValueVec, Clone)]
pub struct Word {
    pub word: String,
    pub explain: String,
    add_time: i64,
    update_time: i64,
    triger_time: i64,
}

pub struct Empty {}

impl From<Vec<serde_json::Value>> for Empty {
    fn from(_: Vec<serde_json::Value>) -> Self {
        Empty {}
    }
}

impl D1 {
    pub fn new(account_id: &str, database_id: &str, api_token: &str) -> D1 {
        D1 {
            account_id: account_id.to_string(),
            database_id: database_id.to_string(),
            api_token: api_token.to_string(),
        }
    }

    pub async fn query<T: From<Vec<serde_json::Value>>>(
        &self,
        sql: &str,
        params: Vec<String>,
    ) -> Result<Vec<T>, D1Error> {
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

        let body = serde_json::to_string(&QueryBody {
            sql: sql.to_string(),
            params: params.clone(),
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

        let response_body = r.text().await?;

        let result = serde_json::from_str::<QueryResult>(&response_body)?;

        println!("[{}] args: {:?} messages: {}", sql, params, result.messages);

        if !result.success {
            return Err(D1Error::from(format!("query failed: {}", result.errors)));
        }

        let mut rs: Vec<T> = vec![];

        for v in result.result.unwrap() {
            for rv in v.results.rows {
                rs.push(T::from(rv));
            }
        }

        Ok(rs)
    }

    pub async fn random_word(&self) -> Result<Word, D1Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let words = match self
            .query::<Word>(
                "SELECT * FROM words WHERE reminder_time <= ? ORDER BY RANDOM() LIMIT 1",
                vec![now.clone()],
            )
            .await
        {
            Ok(v) if !v.is_empty() => v,
            _ => {
                self.query::<Word>("SELECT * FROM words ORDER BY RANDOM() LIMIT 1", vec![])
                    .await?
            }
        };

        if words.len() == 0 {
            return Err(D1Error::from("no word found"));
        }

        let v = words[0].clone();

        match self
            .query::<Empty>(
                "UPDATE words SET reminder_time = ? WHERE word = ?",
                vec![now.clone(), v.word.clone()],
            )
            .await
        {
            Err(e) => println!("update reminder_time error: {}", e),
            _ => {}
        }

        Ok(v)
    }
}

#[cfg(test)]
mod test {
    use std::fs;

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
            auth.database_id.as_str(),
            auth.api_token.as_str(),
        );

        let result = d1
            .query::<Word>("select * from words limit 10", vec![])
            .await
            .unwrap();

        println!("{:?}", result);
    }
}
