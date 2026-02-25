use std::sync::Arc;

use aws_lambda_events::lambda_function_urls::LambdaFunctionUrlRequest;
use base64::{Engine, engine::general_purpose};
use hjcommon::opts::RunOpt;
use hjcommon::tg::{send_random_word, set_webhook};
use hjnative::opts::run_opts;
use hjnative::{ai::Workers, d1::D1};
use lambda_runtime::LambdaEvent;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

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
            Err(lambda_runtime::Error::from("Unknown request"))
        }
    }

    async fn request_handler(
        &self,
        request: LambdaRequest,
    ) -> Result<Response, lambda_runtime::Error> {
        match request.command {
            RequestCommand::SendRandomWord => {
                send_random_word(self.run_opt.clone()).await?;
                Ok(Response {
                    msg: "Send successful.".to_string(),
                })
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

                let result =
                    match set_webhook(&self.run_opt.bot, url.as_ref(), self.run_opt.matainer).await
                    {
                        Ok(_) => format!("register telegram bot to {} successful", url),
                        Err(e) => format!("Set webhook failed: {}", e),
                    };

                info!("{}", result);
                return Ok(Response { msg: result });
            }

            "/d1/create_table" => {
                d1_orm::migrate(
                    &self.run_opt.d1,
                    hjcommon::d1::migrations(),
                    None,
                    Some(|s: &str| info!("{}", s)),
                )
                .await
                .map_err(|e| lambda_runtime::Error::from(e.to_string()))?;
                return Ok(Response {
                    msg: "create table [words] successful".to_string(),
                });
            }

            "/tgbot" => {
                let bytes = event.body.ok_or("body is none")?;

                let body = if event.is_base64_encoded {
                    general_purpose::STANDARD.decode(bytes)?
                } else {
                    bytes.as_bytes().to_vec()
                };

                debug!("body: {}", String::from_utf8_lossy(&body));

                let update2: frankenstein::updates::Update = serde_json::from_slice(&body)?;

                return match hjcommon::tg::handle(self.run_opt.clone(), update2).await {
                    Ok(_) => {
                        debug!("Update was handled by bot.");
                        Ok(Response {
                            msg: "Update was handled by bot.".to_string(),
                        })
                    }
                    Err(e) => {
                        error!("Update was not handled by bot: {}", e);
                        Ok(Response {
                            msg: format!("Update was not handled by bot: {}", e).to_string(),
                        })
                    }
                };
            }

            _ => {}
        }

        Err(lambda_runtime::Error::from("404 NOT FOUND".to_string()))
    }
}

#[cfg(test)]
mod test {
    use log::info;

    use crate::LambdaRequest;

    fn init() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();
    }

    #[test]
    fn marshal() {
        init();

        let data = serde_json::to_string(&LambdaRequest {
            command: crate::RequestCommand::SendRandomWord,
        })
        .unwrap();

        info!("{}", data);
    }
}
