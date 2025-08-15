use std::sync::Arc;

use aws_lambda_events::lambda_function_urls::LambdaFunctionUrlRequest;
use base64::{Engine, engine::general_purpose};
use frankenstein::AsyncTelegramApi;
use frankenstein::methods::{SetMyCommandsParams, SetWebhookParams};
use hjcommon::opts::run_opts;
use hjcommon::{ai::Workers, d1::D1};
use hjdef::opts::RunOpt;
use hjtg::tg::{bot_commands, send_random_word};
use lambda_runtime::LambdaEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    let run_opt = run_opts().await.unwrap();

    let handler = LambdaHandler {
        run_opt: Arc::new(run_opt),
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
    run_opt: Arc<RunOpt<D1, Workers>>,
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
                send_random_word(self.run_opt.clone()).await?;
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

                match self
                    .run_opt
                    .bot
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

                match self
                    .run_opt
                    .bot
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
                let result = hjtg::tg::handle(self.run_opt.clone(), update2).await;

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
