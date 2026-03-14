use std::sync::Arc;

use frankenstein::{
    AsyncTelegramApi,
    methods::{DeleteWebhookParams, GetUpdatesParams, SendMessageParams},
    types::ChatId,
};
use hjcommon::tg::handle;
use hjnative::opts::run_opts;
use log::*;

/*
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

    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let opt = Arc::new(run_opts().await.unwrap());

            match d1_orm::migrate(
                &opt.d1,
                hjcommon::d1::migrations(),
                None,
                Some(|s: &str| info!("{}", s)),
            )
            .await
            {
                Ok(_) => info!("create table [words] successful"),
                Err(e) => {
                    let config = opt.get_config().await.unwrap();
                    if let Some(bot) = config.bot {
                        let _ = bot.send_message(
                            &SendMessageParams::builder()
                                .chat_id(ChatId::Integer(config.maintainer_id as i64))
                                .text(format!("create_table [words] error: {}", e))
                                .build(),
                        )
                        .await;
                    }
                }
            }

            let config = opt.get_config().await.unwrap();
            if let Some(bot) = &config.bot {
                bot.delete_webhook(
                    &DeleteWebhookParams::builder()
                        .drop_pending_updates(true)
                        .build(),
                )
                .await
                .unwrap();

                bot.send_message(
                    &SendMessageParams::builder()
                        .chat_id(ChatId::Integer(config.maintainer_id as i64))
                        .text("start new bot")
                        .build(),
                )
                .await
                .unwrap();
            } else {
                error!("telegram bot token not configured at startup");
            }

            let mut update_params = GetUpdatesParams::builder().build();

            loop {
                // Fetch the config on each iteration (the cache and TTL will limit DB hits)
                let config = match opt.get_config().await {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to get config: {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        continue;
                    }
                };

                let bot = match config.bot.as_ref() {
                    Some(b) => b,
                    None => {
                        error!("Telegram bot token not configured");
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                        continue;
                    }
                };

                let result = bot.get_updates(&update_params).await;
                match result {
                    Ok(response) => {
                        for update in response.result {
                            let opt = opt.clone();
                            let update_id = update.update_id;
                            tokio::task::spawn_local(async move {
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
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        })
        .await;
}
