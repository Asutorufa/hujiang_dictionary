use hj_rust::{opts::run_opts, telegram::run_bot};

/*
 telegram bot token env: TELOXIDE_TOKEN=
 maintainer od env: MAINTAINER_ID=
 allow user ids env: ALLOW_USERS=12231,1213314
 cloudflare account id: CLOUDFLARE_ACCOUNT_ID=
 cloudflare api token: CLOUDFLARE_API_TOKEN=
 cloudflare d1 database id: CLOUDFLARE_D1_DATABASE_ID=
 cloudflare d1 database name: CLOUDFLARE_D1_DATABASE_NAME=
*/
#[tokio::main]
async fn main() {
    run_bot(run_opts().await.unwrap()).await.dispatch().await;
}
