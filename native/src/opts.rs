use crate::ai::Workers;
use crate::d1::{D1, Database};
use hjcommon::opts::{ConfigCache, RunOpt};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::RwLock;

pub async fn run_opts() -> Result<RunOpt<D1, Workers>, Box<dyn std::error::Error>> {
    let cloudflare_account_id = std::env::var("CLOUDFLARE_ACCOUNT_ID").unwrap_or("".to_string());
    let cloudflare_api_token = std::env::var("CLOUDFLARE_API_TOKEN").unwrap_or("".to_string());
    let cloudflare_d1_database_id =
        std::env::var("CLOUDFLARE_D1_DATABASE_ID").unwrap_or("".to_string());
    let cloudflare_d1_database_name =
        std::env::var("CLOUDFLARE_D1_DATABASE_NAME").unwrap_or("".to_string());

    let auth_secret = std::env::var("AUTH_SECRET")?;
    if auth_secret.is_empty() {
        return Err("AUTH_SECRET is empty".into());
    }

    let auth_username = std::env::var("AUTH_USERNAME").unwrap_or("".to_string());
    let auth_password = std::env::var("AUTH_PASSWORD").unwrap_or("".to_string());
    let auth_token_expiration = std::env::var("AUTH_TOKEN_EXPIRATION")
        .unwrap_or("1".to_string())
        .parse::<i64>()
        .unwrap_or(1);

    let workers = Workers::new(&cloudflare_account_id, &cloudflare_api_token);

    let database: Database = if cloudflare_d1_database_id.is_empty() {
        Database::Name(cloudflare_d1_database_name)
    } else {
        Database::UUID(cloudflare_d1_database_id)
    };

    let d1 = D1::new(&cloudflare_account_id, &cloudflare_api_token, database).await;

    Ok(RunOpt {
        d1,
        translator: workers.clone(),
        workers_ai: if workers.account_id.is_empty() || workers.api_key.is_empty() {
            None
        } else {
            Some(hj_ai::provider::Provider::OpenAI(hj_ai::openai::OpenAI {
                name: "workers-ai".to_string(),
                base_url: format!(
                    "https://api.cloudflare.com/client/v4/accounts/{}/ai/v1",
                    workers.account_id
                ),
                api_key: workers.api_key.clone(),
                model: "".to_string(),
                models: std::collections::HashSet::new(),
                ..Default::default()
            }))
        },
        auth_secret,
        auth_username,
        auth_password,
        auth_token_expiration,
        config_cache: Arc::new(RwLock::new(ConfigCache {
            allow_users: Arc::new(HashSet::new()),
            maintainer_id: 0,
            bot: None,
            last_updated: 0,
            google_search_api_key: "".to_string(),
            google_search_cx: "".to_string(),
        })),
    })
}
