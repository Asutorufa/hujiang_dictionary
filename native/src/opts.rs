use crate::ai::Workers;
use crate::d1::{D1, Database};
use frankenstein::client_reqwest;
use hjcommon::opts::RunOpt;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub async fn run_opts() -> Result<RunOpt<D1, Workers>, Box<dyn std::error::Error>> {
    let maintainer_id = std::env::var("MAINTAINER_ID")?.parse::<i64>()?;
    let telegram_bot_token = std::env::var("TELOXIDE_TOKEN")?;

    let allow_users = std::env::var("ALLOW_USERS")
        .unwrap_or("".to_string())
        .split(",")
        .map(|v| v.parse::<i64>().unwrap_or(0))
        .collect::<Vec<_>>();

    let cloudflare_account_id = std::env::var("CLOUDFLARE_ACCOUNT_ID").unwrap_or("".to_string());
    let cloudflare_api_token = std::env::var("CLOUDFLARE_API_TOKEN").unwrap_or("".to_string());
    let cloudflare_d1_database_id =
        std::env::var("CLOUDFLARE_D1_DATABASE_ID").unwrap_or("".to_string());
    let cloudflare_d1_database_name =
        std::env::var("CLOUDFLARE_D1_DATABASE_NAME").unwrap_or("".to_string());

    let auth_secret = std::env::var("AUTH_SECRET")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default_secret".to_string());

    let auth_username = std::env::var("AUTH_USERNAME").unwrap_or("".to_string());
    let auth_password = std::env::var("AUTH_PASSWORD").unwrap_or("".to_string());
    let auth_token_expiration = std::env::var("AUTH_TOKEN_EXPIRATION")
        .unwrap_or("1".to_string())
        .parse::<i64>()
        .unwrap_or(1);

    let mut set = HashSet::from([maintainer_id]);

    for v in allow_users {
        set.insert(v);
    }

    let workers = Workers::new(&cloudflare_account_id, &cloudflare_api_token);

    let database: Database = if cloudflare_d1_database_id.is_empty() {
        Database::Name(cloudflare_d1_database_name)
    } else {
        Database::UUID(cloudflare_d1_database_id)
    };

    let d1 = D1::new(&cloudflare_account_id, &cloudflare_api_token, database).await;

    Ok(RunOpt {
        allow_users: Arc::new(set),
        matainer: maintainer_id,
        d1,
        workers_ai: workers,
        bot: client_reqwest::Bot::new(&telegram_bot_token),
        custom_llms: HashMap::new(),
        auth_secret,
        auth_username,
        auth_password,
        auth_token_expiration,
    })
}
