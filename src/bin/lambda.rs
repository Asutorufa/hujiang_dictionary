use aws_lambda_events::lambda_function_urls::LambdaFunctionUrlRequest;
use base64::{Engine, engine::general_purpose};
use hj_rust::{
    opts::run_opts,
    telegram::{Command, RunOpt, handler},
};
use serde::Serialize;
use teloxide::{
    dptree::{self},
    prelude::*,
    types::{Me, Update},
    utils::command::BotCommands,
};

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    let bot = Bot::from_env();

    let me = bot.get_me().await?;
    println!("Bot: {}, {}", me.username.as_ref().unwrap(), me.id.0);

    let run_opt = run_opts().await.unwrap();

    let handler = LambdaHandler { bot, me, run_opt };
    lambda_runtime::run(lambda_runtime::service_fn(|event| {
        handler.bot_handler(event)
    }))
    .await
}

#[derive(Serialize)]
struct Response {
    msg: String,
}

struct LambdaHandler {
    bot: Bot,
    me: Me,
    run_opt: RunOpt,
}

impl LambdaHandler {
    async fn bot_handler(
        &self,
        event: lambda_runtime::LambdaEvent<LambdaFunctionUrlRequest>,
    ) -> Result<Response, lambda_runtime::Error> {
        match event.payload.raw_path {
            Some(path) if path == "/tgbot/register" => {
                let url = format!(
                    "https://{}/tgbot",
                    event
                        .payload
                        .request_context
                        .domain_name
                        .ok_or("domain name is none")?
                );

                println!("Registering webhook: {}", url);

                let _ = self.bot.set_my_commands(Command::bot_commands()).await;
                let _ = self.bot.set_webhook(url::Url::parse(&url)?).send().await?;

                return Ok(Response {
                    msg: format!("register telegram bot to {} successful", url).to_string(),
                });
            }
            _ => {}
        }

        let bytes = event.payload.body.ok_or("body is none")?;

        let body = if event.payload.is_base64_encoded {
            general_purpose::STANDARD.decode(bytes)?
        } else {
            bytes.as_bytes().to_vec()
        };

        println!("body: {}", String::from_utf8_lossy(&body));

        let update: Update = serde_json::from_slice(&body)?;

        let handler = handler();

        let dependencies = dptree::deps![
            self.me.clone(),
            self.bot.clone(),
            update,
            self.run_opt.clone()
        ];

        let result = handler.dispatch(dependencies).await;

        match result {
            ControlFlow::Break(Ok(())) => {
                println!("Update was handled by bot.");
                Ok(Response {
                    msg: "Update was handled by bot.".to_string(),
                })
            }
            ControlFlow::Break(Err(e)) => {
                println!("Error: {}", e);
                Err(lambda_runtime::Error::from(e))
            }
            ControlFlow::Continue(_) => {
                println!("Update was not handled by bot.");
                Ok(Response {
                    msg: "Update was not handled by bot.".to_string(),
                })
            }
        }
    }
}
