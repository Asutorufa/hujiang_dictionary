use frankenstein::{
    AsyncTelegramApi, client_reqwest,
    methods::{DeleteWebhookParams, GetUpdatesParams, SendMessageParams},
    types::ChatId,
};
use hjcommon::opts::run_opts;
use hjtg::tg::handle;
use std::env;

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
    let token = env::var("TELOXIDE_TOKEN").unwrap();
    let opt = run_opts().await.unwrap();
    let bot = client_reqwest::Bot::new(&token);

    bot.delete_webhook(
        &DeleteWebhookParams::builder()
            .drop_pending_updates(true)
            .build(),
    )
    .await
    .unwrap();

    bot.send_message(
        &SendMessageParams::builder()
            .chat_id(ChatId::Integer(opt.matainer as i64))
            .text("start new bot")
            .build(),
    )
    .await
    .unwrap();

    let mut update_params = GetUpdatesParams::builder().build();

    loop {
        let result = bot.get_updates(&update_params).await;
        match result {
            Ok(response) => {
                for update in response.result {
                    let tk = token.clone();
                    let opt = opt.clone();
                    let update_id = update.update_id;
                    tokio::spawn(async move {
                        match handle(opt, &tk, update).await {
                            Ok(_) => {}
                            Err(e) => println!("Failed to handle update: {e:?}"),
                        }
                    });
                    update_params.offset = Some(i64::from(update_id) + 1);
                }
            }
            Err(error) => {
                println!("Failed to get updates: {error:?}");
            }
        }
    }
}
