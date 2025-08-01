use std::collections::HashSet;

use hj_rust::telegram::{RunOpt, run_bot};
use teloxide::prelude::UserId;

/*
 telegram bot token env: TELOXIDE_TOKEN=
 aws instance env: AWS_INSTANCE=
 maintainer od env: MAINTAINER_ID=
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

    let mut set = HashSet::from([UserId(maintainer_id)]);

    for v in allow_users {
        set.insert(v);
    }

    let mut dispatcher = run_bot(RunOpt { maintainer: set }).await;

    dispatcher.dispatch().await;
}
