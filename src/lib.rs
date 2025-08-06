pub mod ai;
pub mod d1;
pub mod en;
pub mod google;
pub mod jp;
pub mod kotobakku;
pub mod opts;
pub mod telegram;
pub mod weblio;

use std::ops::ControlFlow;
use teloxide::{
    dptree::{self},
    types::Update,
    utils::command::BotCommands,
};

use teloxide::{
    Bot,
    prelude::{Request, Requester},
};
use worker::*;

use crate::{
    opts::run_opts,
    telegram::{Command, handler},
};

// TODO split to different package, because the reqwest dep hyper which can't compile to
// wasm
#[event(fetch)]
async fn main(req: worker::Request, _env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    let bot = Bot::from_env();

    let me = bot
        .get_me()
        .await
        .map_err(|e| worker::Error::from(e.to_string()))?;

    // TODO worker env
    let run_opt = run_opts().await.unwrap();

    println!("Bot: {}, {}", me.username.as_ref().unwrap(), me.id.0);

    let mut router = Router::new();

    router = router.on_async("/tgbot/register", async |req, _ctx| {
        let url = format!("https://{}/tgbot", req.url()?.host().unwrap().to_string());

        println!("Registering webhook: {}", url.clone());

        let _ = bot.set_my_commands(Command::bot_commands());
        let _ = bot
            .set_webhook(url::Url::parse(&url)?)
            .send()
            .await
            .map_err(|x| worker::Error::from(x.to_string()))?;

        Response::ok(format!("register telegram bot to {} successful", url))
    });

    router = router.post_async("/tgbot", async |mut req, _ctx| {
        let update = req.json::<Update>().await?;

        let handler = handler();

        let dependencies = dptree::deps![me.clone(), bot.clone(), update, run_opt.clone()];

        let result = handler.dispatch(dependencies).await;

        match result {
            ControlFlow::Break(Ok(())) => {
                println!("Update was handled by bot.");
                Response::ok("Update was handled by bot.")
            }
            ControlFlow::Break(Err(e)) => {
                println!("Error: {}", e);
                Err(worker::Error::from(e.to_string()))
            }
            ControlFlow::Continue(_) => {
                println!("Update was not handled by bot.");
                Response::ok("Update was not handled by bot.")
            }
        }
    });

    router.run(req, _env).await
}
