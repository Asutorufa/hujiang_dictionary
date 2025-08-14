pub mod ai;
pub mod d1;

use crate::{ai::WasmAI, d1::WasmD1};
use frankenstein::{
    AsyncTelegramApi,
    methods::{SetMyCommandsParams, SetWebhookParams},
    updates::Update,
};
use hjdef::opts::RunOpt;
use hjtg::tg::{self, bot_commands};
use std::{collections::HashSet, env};
use worker::*;

// TODO split to different package, because the reqwest dep hyper which can't compile to
// wasm
#[event(fetch)]
async fn main(req: worker::Request, _env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let token = env::var("TELOXIDE_TOKEN").unwrap();
    let opt = RunOpt {
        allow_users: HashSet::new(),
        d1: WasmD1::new(_env.clone(), "d1").await.unwrap(),
        workers_ai: WasmAI {},
        matainer: 0,
    };
    let bot = frankenstein::client_reqwest::Bot::new(&token);

    let mut router = Router::new();

    router = router.on_async("/tgbot/register", async |req, _ctx| {
        let url = format!("https://{}/tgbot", req.url()?.host().unwrap().to_string());

        println!("Registering webhook: {}", url.clone());

        bot.set_my_commands(
            &SetMyCommandsParams::builder()
                .commands(bot_commands())
                .build(),
        )
        .await
        .map_err(|e| worker::Error::from(e.to_string()))?;

        bot.set_webhook(&SetWebhookParams::builder().url(url.clone()).build())
            .await
            .map_err(|e| worker::Error::from(e.to_string()))?;

        Response::ok(format!("register telegram bot to {} successful", url))
    });

    router = router.post_async("/tgbot", async |mut req, _ctx| {
        let update = req.json::<Update>().await?;

        let result = tg::handle(opt.clone(), &token, update).await;

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

    router.run(req, _env.clone()).await
}
