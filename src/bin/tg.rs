use std::collections::HashSet;

use hj_rust::{
    ai::Workers,
    d1::D1,
    telegram::{RunOpt, run_bot},
};
use teloxide::prelude::UserId;

/*
 telegram bot token env: TELOXIDE_TOKEN=
 maintainer od env: MAINTAINER_ID=
 allow user ids env: ALLOW_USERS=12231,1213314
 cloudflare account id: CLOUDFLARE_ACCOUNT_ID=
 cloudflare api token: CLOUDFLARE_API_TOKEN=
 cloudflare d1  database id: CLOUDFLARE_D1_DATABASE_ID=
*/
#[tokio::main]
async fn main() {
    let allow_users = std::env::var("ALLOW_USERS")
        .unwrap()
        .split(",")
        .map(|v| return UserId(v.parse::<u64>().unwrap_or(0)))
        .collect::<Vec<_>>();

    let maintainer_id = std::env::var("MAINTAINER_ID")
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let cloudflare_account_id = std::env::var("CLOUDFLARE_ACCOUNT_ID").unwrap();
    let cloudflare_api_token = std::env::var("CLOUDFLARE_API_TOKEN").unwrap();
    let cloudflare_d1_database_id = std::env::var("CLOUDFLARE_D1_DATABASE_ID").unwrap();

    let mut set = HashSet::from([UserId(maintainer_id)]);

    for v in allow_users {
        set.insert(v);
    }

    let workers = Workers::new(&cloudflare_api_token, &cloudflare_account_id);
    let d1 = D1::new(
        &cloudflare_account_id,
        &cloudflare_d1_database_id,
        &cloudflare_api_token,
    );

    let mut dispatcher = run_bot(RunOpt {
        allow_users: set,
        d1,
        workers_ai: workers,
    })
    .await;

    dispatcher.dispatch().await;
}
