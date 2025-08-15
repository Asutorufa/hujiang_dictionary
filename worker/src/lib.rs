pub mod ai;
pub mod d1;

use crate::{ai::WasmAI, d1::WasmD1};
use frankenstein::{
    AsyncTelegramApi, client_reqwest,
    methods::{SetMyCommandsParams, SetWebhookParams},
    updates::Update,
};
use hjdef::opts::RunOpt;
use hjtg::tg::{self, bot_commands, send_random_word};
use std::{collections::HashSet, sync::Arc};
use worker::*;

async fn get_opt(env: Env) -> Arc<RunOpt<WasmD1, WasmAI>> {
    let token = env
        .var("TELEGRAM_TOKEN")
        .unwrap()
        .as_ref()
        .as_string()
        .unwrap();

    let maintainer_id = env
        .var("MAINTAINER_ID")
        .unwrap()
        .as_ref()
        .as_string()
        .unwrap()
        .parse::<i64>()
        .unwrap();

    let allow_users = match env.var("ALLOW_USERS") {
        Ok(v) => v
            .as_ref()
            .as_string()
            .unwrap_or("".to_string())
            .split(",")
            .map(|v| return v.parse::<i64>().unwrap_or(0))
            .collect::<Vec<_>>(),
        _ => vec![],
    };

    let mut set = HashSet::from([maintainer_id]);

    for v in allow_users {
        set.insert(v);
    }

    Arc::new(RunOpt {
        allow_users: set,
        d1: WasmD1::new(env.clone(), "DB").await,
        workers_ai: WasmAI::new(env.clone(), "AI"),
        matainer: maintainer_id,
        bot: client_reqwest::Bot::new(&token),
    })
}

#[event(fetch)]
async fn main(req: worker::Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let opt = get_opt(env.clone()).await;

    let mut router = Router::new();

    router = router.on_async("/tgbot/register", async |req, _ctx| {
        let url = format!("https://{}/tgbot", req.url()?.host().unwrap().to_string());

        println!("Registering webhook: {}", url.clone());

        opt.bot
            .set_my_commands(
                &SetMyCommandsParams::builder()
                    .commands(bot_commands())
                    .build(),
            )
            .await
            .map_err(|e| worker::Error::from(e.to_string()))?;

        opt.bot
            .set_webhook(&SetWebhookParams::builder().url(url.clone()).build())
            .await
            .map_err(|e| worker::Error::from(e.to_string()))?;

        Response::ok(format!("register telegram bot to {} successful", url))
    });

    router = router.post_async("/tgbot", async |mut req, _ctx| {
        let update = req.json::<Update>().await?;

        let result = tg::handle(opt.clone(), update).await;

        return match result {
            Ok(_) => {
                println!("Update was handled by bot.");
                Response::ok("Update was handled by bot.")
            }
            Err(e) => {
                println!("Update was not handled by bot: {}", e);
                Response::error(format!("Update was not handled by bot: {}", e), 500)
            }
        };
    });

    router.run(req, env.clone()).await
}

#[event(scheduled)]
pub async fn scheduled(_: ScheduledEvent, env: Env, _: ScheduleContext) {
    console_error_panic_hook::set_once();

    let opt = get_opt(env.clone()).await;

    match send_random_word(opt).await {
        Err(e) => {
            println!("Error: {}", e);
        }
        Ok(_) => {}
    }
}
