use std::collections::HashSet;

use teloxide::types::UserId;

use crate::{
    ai::Workers,
    d1::{D1, Database},
    telegram::RunOpt,
};

pub async fn run_opts() -> Result<RunOpt, Box<dyn std::error::Error>> {
    let allow_users = std::env::var("ALLOW_USERS")?
        .split(",")
        .map(|v| return UserId(v.parse::<u64>().unwrap_or(0)))
        .collect::<Vec<_>>();

    let maintainer_id = std::env::var("MAINTAINER_ID")?.parse::<u64>()?;

    let cloudflare_account_id = std::env::var("CLOUDFLARE_ACCOUNT_ID")?;
    let cloudflare_api_token = std::env::var("CLOUDFLARE_API_TOKEN")?;
    let cloudflare_d1_database_id = std::env::var("CLOUDFLARE_D1_DATABASE_ID");
    let cloudflare_d1_database_name = std::env::var("CLOUDFLARE_D1_DATABASE_NAME");

    let mut set = HashSet::from([UserId(maintainer_id)]);

    for v in allow_users {
        set.insert(v);
    }

    let workers = Workers::new(&cloudflare_account_id, &cloudflare_api_token);

    let database: Database = match cloudflare_d1_database_id {
        Ok(v) if !v.is_empty() => Database::UUID(v),
        _ => Database::Name(cloudflare_d1_database_name?),
    };

    let d1 = D1::new(&cloudflare_account_id, &cloudflare_api_token, database).await?;

    Ok(RunOpt {
        allow_users: set,
        matainer: UserId(maintainer_id),
        d1,
        workers_ai: workers,
    })
}
