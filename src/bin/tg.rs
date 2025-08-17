use std::sync::Arc;

use frankenstein::{
    AsyncTelegramApi,
    methods::{DeleteWebhookParams, GetUpdatesParams, SendMessageParams},
    types::ChatId,
};
use hjcommon::d1::DB;
use hjcommon::tg::handle;
use hjnative::opts::run_opts;
use log::*;

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
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let opt = Arc::new(run_opts().await.unwrap());

    match opt.d1.create_table().await {
        Ok(_) => info!("create table [words] successful"),
        Err(e) => {
            let _ = opt
                .bot
                .send_message(
                    &SendMessageParams::builder()
                        .chat_id(ChatId::Integer(opt.matainer as i64))
                        .text(format!("create_table [words] error: {}", e))
                        .build(),
                )
                .await;
        }
    }

    opt.bot
        .delete_webhook(
            &DeleteWebhookParams::builder()
                .drop_pending_updates(true)
                .build(),
        )
        .await
        .unwrap();

    opt.bot
        .send_message(
            &SendMessageParams::builder()
                .chat_id(ChatId::Integer(opt.matainer as i64))
                .text("start new bot")
                .build(),
        )
        .await
        .unwrap();

    let mut update_params = GetUpdatesParams::builder().build();

    loop {
        let result = opt.bot.get_updates(&update_params).await;
        match result {
            Ok(response) => {
                for update in response.result {
                    let opt = opt.clone();
                    let update_id = update.update_id;
                    tokio::spawn(async move {
                        match handle(opt, update).await {
                            Ok(_) => {}
                            Err(e) => error!("Failed to handle update: {e:?}"),
                        }
                    });
                    update_params.offset = Some(i64::from(update_id) + 1);
                }
            }
            Err(error) => {
                error!("Failed to get updates: {error:?}");
            }
        }
    }
}
