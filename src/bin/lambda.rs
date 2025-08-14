use std::env;

use aws_lambda_events::lambda_function_urls::LambdaFunctionUrlRequest;
use base64::{Engine, engine::general_purpose};
use frankenstein::AsyncTelegramApi;
use frankenstein::methods::{SendMessageParams, SetMyCommandsParams, SetWebhookParams};
use hjcommon::opts::run_opts;
use hjcommon::{ai::Workers, d1::D1};
use hjdef::d1::DB;
use hjdef::opts::RunOpt;
use hjtg::tg::{bot_commands, html_escape};
use lambda_runtime::LambdaEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    let run_opt = run_opts().await.unwrap();

    let handler = LambdaHandler {
        telegram_api: env::var("TELOXIDE_TOKEN").unwrap(),
        run_opt,
    };
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
    telegram_api: String,
    run_opt: RunOpt<D1, Workers>,
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
                            html_escape(v.word.as_str()),
                            html_escape(v.explain.as_str())
                        )
                    }
                };

                let bot =
                    frankenstein::client_reqwest::Bot::new(self.telegram_api.clone().as_str());

                bot.send_message(
                    &SendMessageParams::builder()
                        .chat_id(frankenstein::types::ChatId::Integer(
                            self.run_opt.matainer as i64,
                        ))
                        .text(reply)
                        .parse_mode(frankenstein::ParseMode::Html)
                        .link_preview_options(frankenstein::types::LinkPreviewOptions::DISABLED)
                        .build(),
                )
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

                let bot =
                    frankenstein::client_reqwest::Bot::new(self.telegram_api.clone().as_str());

                match bot
                    .set_my_commands(
                        &SetMyCommandsParams::builder()
                            .commands(bot_commands())
                            .build(),
                    )
                    .await
                {
                    Ok(_) => println!("Set my commands successful."),
                    Err(e) => {
                        return Ok(Response {
                            msg: format!("Set my commands failed: {}", e).to_string(),
                        });
                    }
                };

                match bot
                    .set_webhook(&SetWebhookParams::builder().url(url.clone()).build())
                    .await
                {
                    Ok(_) => println!("Set webhook successful."),
                    Err(e) => {
                        return Ok(Response {
                            msg: format!("Set webhook failed: {}", e).to_string(),
                        });
                    }
                };

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

                let update2: frankenstein::updates::Update = serde_json::from_slice(&body)?;
                let result =
                    hjtg::tg::handle(self.run_opt.clone(), &self.telegram_api, update2).await;

                return match result {
                    Ok(_) => {
                        println!("Update was handled by bot.");
                        Ok(Response {
                            msg: "Update was handled by bot.".to_string(),
                        })
                    }
                    Err(e) => {
                        println!("Update was not handled by bot: {}", e);
                        Ok(Response {
                            msg: format!("Update was not handled by bot: {}", e).to_string(),
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
