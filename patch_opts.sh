#!/bin/bash
cat << 'INNER_EOF' > native/src/opts.rs
use crate::ai::Workers;
use crate::d1::{D1, Database};
use frankenstein::client_reqwest;
use hjcommon::opts::RunOpt;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub async fn run_opts() -> Result<RunOpt<D1, Workers>, Box<dyn std::error::Error>> {
    let maintainer_id = std::env::var("MAINTAINER_ID")?.parse::<i64>()?;
    let telegram_bot_token = std::env::var("TELOXIDE_TOKEN")?;

    let allow_users = std::env::var("ALLOW_USERS")
        .unwrap_or("".to_string())
        .split(",")
        .map(|v| v.parse::<i64>().unwrap_or(0))
        .collect::<Vec<_>>();

    let cloudflare_account_id = std::env::var("CLOUDFLARE_ACCOUNT_ID").unwrap_or("".to_string());
    let cloudflare_api_token = std::env::var("CLOUDFLARE_API_TOKEN").unwrap_or("".to_string());
    let cloudflare_d1_database_id =
        std::env::var("CLOUDFLARE_D1_DATABASE_ID").unwrap_or("".to_string());
    let cloudflare_d1_database_name =
        std::env::var("CLOUDFLARE_D1_DATABASE_NAME").unwrap_or("".to_string());

    let auth_secret = std::env::var("AUTH_SECRET")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default_secret".to_string());

    let auth_username = std::env::var("AUTH_USERNAME").unwrap_or("".to_string());
    let auth_password = std::env::var("AUTH_PASSWORD").unwrap_or("".to_string());
    let auth_token_expiration = std::env::var("AUTH_TOKEN_EXPIRATION")
        .unwrap_or("1".to_string())
        .parse::<i64>()
        .unwrap_or(1);

    let mut set = HashSet::from([maintainer_id]);

    for v in allow_users {
        set.insert(v);
    }

    let workers = Workers::new(&cloudflare_account_id, &cloudflare_api_token);

    let database: Database = if cloudflare_d1_database_id.is_empty() {
        Database::Name(cloudflare_d1_database_name)
    } else {
        Database::UUID(cloudflare_d1_database_id)
    };

    let d1 = D1::new(&cloudflare_account_id, &cloudflare_api_token, database).await;

    Ok(RunOpt {
        allow_users: Arc::new(set),
        matainer: maintainer_id,
        d1,
        translator: workers.clone(),
        workers_ai: if workers.account_id.is_empty() || workers.api_key.is_empty() { None } else { Some(hj_ai::provider::Provider::OpenAI(hj_ai::openai::OpenAI { name: "workers-ai".to_string(), base_url: format!("https://api.cloudflare.com/client/v4/accounts/{}/ai/v1", workers.account_id), api_key: workers.api_key.clone(), model: "".to_string(), models: std::collections::HashSet::new(), ..Default::default() })) },
        bot: client_reqwest::Bot::new(&telegram_bot_token),
        custom_llms: HashMap::new(),
        auth_secret,
        auth_username,
        auth_password,
        auth_token_expiration,
    })
}
INNER_EOF
cat << 'INNER_EOF' > worker/src/lib.rs
pub mod ai;
pub mod consolelog;

use crate::ai::WasmAI;
use frankenstein::client_reqwest;
use hjcommon::ai::providers_from_assets;
use hjcommon::opts::RunOpt;
use hjcommon::tg::send_random_word;
use log::error;
use std::sync::{Once, OnceLock};
use std::{collections::HashMap, collections::HashSet, sync::Arc};
use worker::*;

static INIT: Once = Once::new();
static ENV_CONFIG: OnceLock<EnvConfig> = OnceLock::new();
static CUSTOM_LLMS: OnceLock<HashMap<String, hj_ai::provider::Provider>> = OnceLock::new();

struct EnvConfig {
    allow_users: Arc<HashSet<i64>>,
    maintainer_id: i64,
    bot: client_reqwest::Bot,
    auth_secret: String,
    auth_username: String,
    auth_password: String,
    auth_token_expiration: i64,
}

impl EnvConfig {
    fn from_env(env: &Env) -> Self {
        let token = get_string_from_env(env, "TELEGRAM_TOKEN");
        let bot = client_reqwest::Bot::new(&token);

        let maintainer_id = get_string_from_env(env, "MAINTAINER_ID")
            .parse::<i64>()
            .unwrap_or(0);

        let mut set = HashSet::from([maintainer_id]);
        set.extend(
            get_string_from_env(env, "ALLOW_USERS")
                .split(",")
                .map(|v| v.parse::<i64>().unwrap_or(0)),
        );

        let auth_secret = match get_string_from_env(env, "AUTH_SECRET") {
            s if s.is_empty() => "default_secret".to_string(),
            s => s,
        };

        Self {
            allow_users: Arc::new(set),
            maintainer_id,
            bot,
            auth_secret,
            auth_username: get_string_from_env(env, "AUTH_USERNAME"),
            auth_password: get_string_from_env(env, "AUTH_PASSWORD"),
            auth_token_expiration: get_string_from_env(env, "AUTH_TOKEN_EXPIRATION")
                .parse::<i64>()
                .unwrap_or(1),
        }
    }
}

fn get_string_from_env(env: &Env, key: &str) -> String {
    match env.var(key) {
        Ok(v) => v.to_string(),
        Err(_) => "".to_string(),
    }
}

async fn get_opt(env: Env) -> Arc<RunOpt<worker::D1Database, WasmAI>> {
    console_error_panic_hook::set_once();
    INIT.call_once(|| {
        match consolelog::init_with_level(log::Level::Info) {
            Err(e) => console_error!("Failed to init console log: {}", e),
            _ => console_log!("Console log initialized"),
        };
    });

    let config = ENV_CONFIG.get_or_init(|| EnvConfig::from_env(&env));

    Arc::new(RunOpt {
        allow_users: config.allow_users.clone(),
        d1: env.d1("DB").expect("D1 binding not found"),
        translator: WasmAI::new(&env, "AI"),
        workers_ai: if env.ai("AI").is_ok() { Some(hj_ai::provider::Provider::WorkersAI(hj_ai::workers::WorkersAI { model: "".to_string(), binding: Some(Arc::new(env.ai("AI").unwrap())) })) } else { None },
        matainer: config.maintainer_id,
        bot: config.bot.clone(),
        custom_llms: CUSTOM_LLMS.get_or_init(providers_from_assets).clone(),
        auth_secret: config.auth_secret.clone(),
        auth_username: config.auth_username.clone(),
        auth_password: config.auth_password.clone(),
        auth_token_expiration: config.auth_token_expiration,
    })
}

#[event(fetch)]
async fn main(mut req: Request, env: Env, ctx: Context) -> Result<Response> {
    if req.method() == Method::Options {
        return Response::ok("");
    }

    ctx.pass_through_on_exception();

    let opt = get_opt(env.clone()).await;

    let req_clone = req.clone()?;

    let url = req.url()?;
    let domain = url.host_str().unwrap_or("").to_string();
    let path = url.path();
    let method = req.method().to_string();

    let headers = req.headers();
    let auth_header = headers
        .get("Authorization")
        .ok()
        .flatten()
        .or_else(|| headers.get("authorization").ok().flatten());

    let body = req.bytes().await?;

    match opt
        .serve(&method, path, auth_header.as_deref(), body, &domain)
        .await
    {
        Ok(resp) => {
            let mut r = match resp.body {
                hjcommon::route::UnifiedBody::Bytes(b) => Response::from_bytes(b)?,
                hjcommon::route::UnifiedBody::Stream(s) => {
                    use futures_util::StreamExt;
                    let stream = s.map(|res| res.map(|b| b.to_vec()).map_err(|e| worker::Error::RustError(e.to_string())));
                    Response::from_stream(stream)?
                }
            };
            r = r.with_status(resp.status);
            for (k, v) in resp.headers {
                r.headers_mut().set(&k, &v)?;
            }
            Ok(r)
        }
        Err(e) => match e {
            hjcommon::route::Error::NotFound => {
                env.assets("ASSETS")?.fetch_request(req_clone).await
            }
            _ => Response::error(e.to_string(), 500),
        },
    }
}

#[event(scheduled)]
async fn cron(_: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    let opt = get_opt(env).await;

    if let Err(e) = send_random_word(opt).await {
        error!("Error: {}", e);
    }
}
INNER_EOF
