use crate::ai::Workers;
use crate::d1::{D1, Database};
use frankenstein::client_reqwest;
use hjcommon::opts::RunOpt;
use std::collections::HashSet;

pub async fn run_opts() -> Result<RunOpt<D1, Workers>, Box<dyn std::error::Error>> {
    let maintainer_id = std::env::var("MAINTAINER_ID")?.parse::<i64>()?;
    let telegram_bot_token = std::env::var("TELOXIDE_TOKEN")?;

    let allow_users = std::env::var("ALLOW_USERS")
        .unwrap_or("".to_string())
        .split(",")
        .map(|v| return v.parse::<i64>().unwrap_or(0))
        .collect::<Vec<_>>();

    let cloudflare_account_id = std::env::var("CLOUDFLARE_ACCOUNT_ID").unwrap_or("".to_string());
    let cloudflare_api_token = std::env::var("CLOUDFLARE_API_TOKEN").unwrap_or("".to_string());
    let cloudflare_d1_database_id =
        std::env::var("CLOUDFLARE_D1_DATABASE_ID").unwrap_or("".to_string());
    let cloudflare_d1_database_name =
        std::env::var("CLOUDFLARE_D1_DATABASE_NAME").unwrap_or("".to_string());

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
        allow_users: set,
        matainer: maintainer_id,
        d1,
        workers_ai: workers,
        bot: client_reqwest::Bot::new(&telegram_bot_token),
    })
}
