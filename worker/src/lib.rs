pub mod ai;
pub mod consolelog;

use crate::ai::WasmAI;
use frankenstein::{client_reqwest, updates::Update};
use hjcommon::ai::OpenAI;
use hjcommon::opts::RunOpt;
use hjcommon::route::LoginRequest;
use hjcommon::tg::send_random_word;
use log::{error, info};
use std::sync::{Once, OnceLock};
use std::{collections::HashMap, collections::HashSet, sync::Arc};
use worker::*;

static INIT: Once = Once::new();
static ENV_CONFIG: OnceLock<EnvConfig> = OnceLock::new();
static CUSTOM_LLMS: OnceLock<HashMap<String, OpenAI>> = OnceLock::new();

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
                .split(',')
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse::<i64>().ok()),
        );
        let allow_users = Arc::new(set);

        let auth_secret = {
            let s = get_string_from_env(env, "AUTH_SECRET");
            if s.is_empty() {
                "default_secret".to_string()
            } else {
                s
            }
        };

        EnvConfig {
            allow_users,
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
        Ok(v) => match v.as_ref().as_string() {
            Some(v) => v,
            None => "".to_string(),
        },
        _ => "".to_string(),
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
        workers_ai: WasmAI::new(&env, "AI"),
        matainer: config.maintainer_id,
        bot: config.bot.clone(),
        custom_llms: CUSTOM_LLMS.get_or_init(OpenAI::from_assets).clone(),
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
    let auth_header = headers.get("Authorization").ok().flatten();

    let body = req.bytes().await?;

    match opt
        .serve(&method, path, auth_header.as_deref(), body, &domain)
        .await
    {
        Ok(resp) => {
            let mut r = Response::from_bytes(resp.body)?;
            r = r.with_status(resp.status);
            for (k, v) in resp.headers {
                r.headers_mut().set(&k, &v)?;
            }
            Ok(r)
        }
        Err(e) => {
            if e.to_string() == "not found" {
                env.assets("ASSETS")?.fetch_request(req_clone).await
            } else {
                Response::error(e.to_string(), 500)
            }
        }
    }
}

#[event(scheduled)]
pub async fn scheduled(_: ScheduledEvent, env: Env, _: ScheduleContext) {
    let opt = get_opt(env).await;

    if let Err(e) = send_random_word(opt).await {
        error!("Error: {}", e);
    }
}

/*
pub async fn static_file(req: Request) -> worker::Result<Response> {
    let mut path = req.path().clone();
    if path.ends_with("/") {
        path = path.strip_suffix("/").unwrap().to_string();
    }

    if path.starts_with("/") {
        path = path.strip_prefix("/").unwrap().to_string();
    }

    let mut paths = vec![
        path.clone(),
        format!(
            "{}{}",
            path,
            if path.is_empty() {
                "index.html"
            } else {
                "/index.html"
            }
        ),
    ];

    if !path.is_empty() {
        paths.push(format!("{}.html", path));
    }

    let (path, file) = get_file(paths)?;

    let ext = if let Some((_, ext)) = path.rsplit_once(".") {
        ext
    } else {
        ""
    };

    let ct = match ext {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "text/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        _ => "",
    };

    let mut resp = ResponseBuilder::new();

    if !ct.is_empty() {
        resp = resp.with_header("content-type", ct)?;
    }

    Ok(resp.fixed(file.data.to_vec()))
}

fn get_file(paths: Vec<String>) -> Result<(String, EmbeddedFile)> {
    for p in paths {
        match Assets::get(p.as_str()) {
            Some(file) => return Ok((p, file)),
            None => {}
        };
    }

    Err(worker::Error::from("file not found"))
}
*/

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::time::Instant;

    #[test]
    fn benchmark_env_parsing_vs_cached_config() {
        // Simulate env vars
        let ids: Vec<String> = (0..1000).map(|i| i.to_string()).collect();
        let allow_users_str = ids.join(",");
        let maintainer_id_str = "12345";
        let token_str = "some_long_token_string";
        let auth_secret_str = "some_secret";
        let auth_username_str = "user";
        let auth_password_str = "pass";
        let auth_expiration_str = "3600";

        let iterations = 1000;

        // Baseline: Parse everything every time
        let start = Instant::now();
        for _ in 0..iterations {
            // Simulate maintainer_id parsing
            let maintainer_id = maintainer_id_str.parse::<i64>().unwrap_or(0);

            // Simulate ALLOW_USERS parsing
            let mut set = HashSet::from([maintainer_id]);
            set.extend(
                allow_users_str
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .filter_map(|v| v.parse::<i64>().ok()),
            );
            let allow_users = Arc::new(set);

            // Simulate other env vars retrieval (allocation)
            let token = token_str.to_string();
            let auth_secret = auth_secret_str.to_string();
            let auth_username = auth_username_str.to_string();
            let auth_password = auth_password_str.to_string();
            let auth_expiration = auth_expiration_str.parse::<i64>().unwrap_or(1);

            // Prevent optimization
            let _ = (
                allow_users,
                maintainer_id,
                token,
                auth_secret,
                auth_username,
                auth_password,
                auth_expiration,
            );
        }
        let duration_parse = start.elapsed();

        // Optimization: Clone cached struct
        #[allow(dead_code)]
        struct CachedConfig {
            allow_users: Arc<HashSet<i64>>,
            maintainer_id: i64,
            token: String,
            auth_secret: String,
            auth_username: String,
            auth_password: String,
            auth_expiration: i64,
        }

        // Setup cache once
        let maintainer_id = maintainer_id_str.parse::<i64>().unwrap_or(0);
        let mut set = HashSet::from([maintainer_id]);
        set.extend(
            allow_users_str
                .split(',')
                .filter(|s| !s.is_empty())
                .filter_map(|v| v.parse::<i64>().ok()),
        );
        let cached = Arc::new(CachedConfig {
            allow_users: Arc::new(set),
            maintainer_id,
            token: token_str.to_string(),
            auth_secret: auth_secret_str.to_string(),
            auth_username: auth_username_str.to_string(),
            auth_password: auth_password_str.to_string(),
            auth_expiration: auth_expiration_str.parse::<i64>().unwrap_or(1),
        });

        let start_clone = Instant::now();
        for _ in 0..iterations {
            let _ = cached.clone();
        }
        let duration_clone = start_clone.elapsed();

        println!(
            "Parsing full config {} times took: {:?}",
            iterations, duration_parse
        );
        println!(
            "Cloning cached config {} times took: {:?}",
            iterations, duration_clone
        );

        assert!(
            duration_clone < duration_parse,
            "Optimization should be significantly faster"
        );
    }
}
