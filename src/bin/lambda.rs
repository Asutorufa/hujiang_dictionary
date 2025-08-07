use aws_lambda_events::lambda_function_urls::LambdaFunctionUrlRequest;
use base64::{Engine, engine::general_purpose};
use hj_rust::{
    opts::run_opts,
    telegram::{Command, RunOpt, handler},
};
use lambda_runtime::LambdaEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use teloxide::{
    dptree::{self},
    prelude::{Request, *},
    sugar::request::RequestLinkPreviewExt,
    types::{Me, Update},
    utils::{command::BotCommands, html},
};

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    let bot = Bot::from_env();

    let me = bot.get_me().await?;
    println!("Bot: {}, {}", me.username.as_ref().unwrap(), me.id.0);

    let run_opt = run_opts().await.unwrap();

    let handler = LambdaHandler { bot, me, run_opt };
    lambda_runtime::run(lambda_runtime::service_fn(|event| handler.handler(event))).await
}

#[derive(Serialize, Deserialize)]
struct Response {
    msg: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RequestCommand {
    SendRandomWord,
}

#[derive(Serialize, Deserialize)]
struct LambdaRequest {
    command: RequestCommand,
}

struct LambdaHandler {
    bot: Bot,
    me: Me,
    run_opt: RunOpt,
}

impl LambdaHandler {
    async fn handler(&self, event: LambdaEvent<Value>) -> Result<Response, lambda_runtime::Error> {
        let (payload, _) = event.into_parts();
        if let Ok(v) = serde_json::from_value::<LambdaFunctionUrlRequest>(payload.clone()) {
            return self.bot_handler(v).await;
        } else if let Ok(v) = serde_json::from_value::<LambdaRequest>(payload.clone()) {
            return self.request_handler(v).await;
        } else {
            return Err(lambda_runtime::Error::from("Unknown request"));
        }
    }

    async fn request_handler(
        &self,
        request: LambdaRequest,
    ) -> Result<Response, lambda_runtime::Error> {
        match request.command {
            RequestCommand::SendRandomWord => {
                let reply = match self.run_opt.d1.random_word().await {
                    Err(e) => e.to_string(),
                    Ok(v) => {
                        format!(
                            "<b>{}</b>\n<tg-spoiler><blockquote expandable>{}</blockquote></tg-spoiler>",
                            html::escape(v.word.as_str()),
                            html::escape(v.explain.as_str())
                        )
                    }
                };

                self.bot
                    .send_message(self.run_opt.matainer, reply)
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .disable_link_preview(true)
                    .await?;

                return Ok(Response {
                    msg: "Send successful.".to_string(),
                });
            }
        }
    }

    async fn bot_handler(
        &self,
        event: LambdaFunctionUrlRequest,
    ) -> Result<Response, lambda_runtime::Error> {
        match event.raw_path.ok_or("Path is None")?.as_str() {
            "/tgbot/register" => {
                let url = format!(
                    "https://{}/tgbot",
                    event
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

            "/tgbot" => {
                let bytes = event.body.ok_or("body is none")?;

                let body = if event.is_base64_encoded {
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

                return match result {
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
                };
            }
            _ => {}
        }

        Err(lambda_runtime::Error::from(format!("404 NOT FOUND")))
    }
}

#[cfg(test)]
mod test {
    use crate::LambdaRequest;

    #[test]
    fn marshal() {
        let data = serde_json::to_string(&LambdaRequest {
            command: crate::RequestCommand::SendRandomWord,
        })
        .unwrap();

        println!("{}", data);
    }
}
