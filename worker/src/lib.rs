pub mod ai;
pub mod consolelog;

use crate::ai::WasmAI;
use frankenstein::client_reqwest;
use hjcommon::d1::migrations;
use hjcommon::opts::RunOpt;
use hjcommon::tg::send_random_word;
use log::error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Once, OnceLock};
use std::{collections::HashSet, sync::Arc};
use worker::*;

static INIT: Once = Once::new();
static ENV_CONFIG: OnceLock<EnvConfig> = OnceLock::new();
static MIGRATION_DONE: AtomicBool = AtomicBool::new(false);

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
    fn from_env(env: &Env) -> Result<Self> {
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

        let auth_secret = get_string_from_env(env, "AUTH_SECRET");
        if auth_secret.is_empty() {
            return Err(worker::Error::from("AUTH_SECRET is required"));
        }

        Ok(Self {
            allow_users: Arc::new(set),
            maintainer_id,
            bot,
            auth_secret,
            auth_username: get_string_from_env(env, "AUTH_USERNAME"),
            auth_password: get_string_from_env(env, "AUTH_PASSWORD"),
            auth_token_expiration: get_string_from_env(env, "AUTH_TOKEN_EXPIRATION")
                .parse::<i64>()
                .unwrap_or(1),
        })
    }
}

fn get_string_from_env(env: &Env, key: &str) -> String {
    match env.var(key) {
        Ok(v) => v.to_string(),
        Err(_) => "".to_string(),
    }
}

async fn get_opt(env: Env) -> Result<Arc<RunOpt<worker::D1Database, WasmAI>>> {
    console_error_panic_hook::set_once();
    INIT.call_once(|| {
        match consolelog::init_with_level(log::Level::Info) {
            Err(e) => console_error!("Failed to init console log: {}", e),
            _ => console_log!("Console log initialized"),
        };
    });

    let config = match ENV_CONFIG.get() {
        Some(c) => c,
        None => {
            let c = EnvConfig::from_env(&env)?;
            let _ = ENV_CONFIG.set(c);
            ENV_CONFIG.get().unwrap()
        }
    };

    if let Ok(ai) = env.ai("AI") {
        hj_ai::workers::set_global_ai(ai);
    }

    Ok(Arc::new(RunOpt {
        allow_users: config.allow_users.clone(),
        d1: env.d1("DB").expect("D1 binding not found"),
        translator: WasmAI::new(&env, "AI"),
        workers_ai: if env.ai("AI").is_ok() {
            Some(hj_ai::provider::Provider::WorkersAI(
                hj_ai::workers::WorkersAI {
                    model: "".to_string(),
                    models: vec![],
                    binding: Some(Arc::new(env.ai("AI").unwrap())),
                    ..Default::default()
                },
            ))
        } else {
            None
        },
        matainer: config.maintainer_id,
        bot: config.bot.clone(),
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

    let opt = get_opt(env.clone()).await?;

    if MIGRATION_DONE
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
        && let Err(e) = d1_orm::migrate(
            &opt.d1,
            migrations(),
            None,
            Some(|s: &str| console_log!("{}", s)),
        )
        .await
    {
        error!("Migration failed: {}", e);
    }

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
                    let stream = s.map(|res: Result<bytes::Bytes, Box<dyn std::error::Error>>| {
                        res.map(|b| b.to_vec())
                            .map_err(|e| worker::Error::RustError(e.to_string()))
                    });
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
    let opt = match get_opt(env).await {
        Ok(opt) => opt,
        Err(e) => {
            error!("Failed to get opt: {}", e);
            return;
        }
    };

    if let Err(e) = send_random_word(opt).await {
        error!("Error: {}", e);
    }
}
