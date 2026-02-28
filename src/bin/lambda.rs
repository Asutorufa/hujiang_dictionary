use std::collections::HashMap;
use std::sync::Arc;

use aws_lambda_events::lambda_function_urls::LambdaFunctionUrlRequest;
use base64::{Engine, engine::general_purpose};
use hjcommon::opts::RunOpt;
use hjcommon::tg::send_random_word;
use hjnative::opts::run_opts;
use hjnative::{ai::Workers, d1::D1};
use lambda_runtime::LambdaEvent;
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

#[derive(Serialize)]
pub struct LocalLambdaFunctionUrlResponse {
    #[serde(rename = "statusCode")]
    pub status_code: i64,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    #[serde(rename = "isBase64Encoded")]
    pub is_base64_encoded: bool,
    pub cookies: Vec<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum MainResponse {
    Url(LocalLambdaFunctionUrlResponse),
    Direct(Response),
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
    async fn handler(
        &self,
        event: LambdaEvent<Value>,
    ) -> Result<MainResponse, lambda_runtime::Error> {
        let (payload, _) = event.into_parts();
        if let Ok(v) = serde_json::from_value::<LambdaFunctionUrlRequest>(payload.clone()) {
            Ok(MainResponse::Url(self.bot_handler(v).await?))
        } else if let Ok(v) = serde_json::from_value::<LambdaRequest>(payload.clone()) {
            Ok(MainResponse::Direct(self.request_handler(v).await?))
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
    ) -> Result<LocalLambdaFunctionUrlResponse, lambda_runtime::Error> {
        let path = event.raw_path.unwrap_or_default();
        let method = event.request_context.http.method.unwrap_or_default();
        let domain = event.request_context.domain_name.unwrap_or_default();

        let headers = event.headers;
        let auth_header = headers
            .get("authorization")
            .or(headers.get("Authorization"))
            .and_then(|h| h.to_str().ok());

        let body_bytes = if let Some(body) = event.body {
            if event.is_base64_encoded {
                general_purpose::STANDARD.decode(body)?
            } else {
                body.into_bytes()
            }
        } else {
            vec![]
        };

        match self
            .run_opt
            .clone()
            .serve(&method, &path, auth_header, body_bytes, &domain)
            .await
        {
            Ok(resp) => {
                let mut headers = HashMap::new();
                for (k, v) in resp.headers {
                    headers.insert(k, v);
                }

                let body_str = match resp.body {
                    hjcommon::route::UnifiedBody::Bytes(b) => {
                        String::from_utf8_lossy(&b).to_string()
                    }
                    hjcommon::route::UnifiedBody::Stream(_) => {
                        return Err(lambda_runtime::Error::from(
                            "Streaming is not supported in this Lambda integration",
                        ));
                    }
                };

                Ok(LocalLambdaFunctionUrlResponse {
                    status_code: resp.status as i64,
                    headers,
                    body: Some(body_str),
                    is_base64_encoded: false,
                    cookies: vec![],
                })
            }
            Err(e) => {
                let status = match e {
                    hjcommon::route::Error::NotFound => 404,
                    _ => 500,
                };
                Ok(LocalLambdaFunctionUrlResponse {
                    status_code: status,
                    headers: HashMap::new(),
                    body: Some(format!("{{\"error\": \"{}\"}}", e)),
                    is_base64_encoded: false,
                    cookies: vec![],
                })
            }
        }
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
