pub mod ai;
pub mod consolelog;
pub mod d1;

use crate::{ai::WasmAI, d1::WasmD1};
use chrono::{Duration, Utc};
use frankenstein::{client_reqwest, updates::Update};
use hjcommon::ai::OpenAI;
use hjcommon::d1::DB;
use hjcommon::opts::RunOpt;
use hjcommon::tg::{self, send_random_word};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::sync::Once;
use std::{collections::HashSet, sync::Arc};
use worker::*;

static INIT: Once = Once::new();

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
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

fn get_secret(env: &Env) -> String {
    let s = get_string_from_env(env, "AUTH_SECRET");
    if s.is_empty() {
        "default_secret".to_string()
    } else {
        s
    }
}

fn create_token(env: &Env, username: &str) -> Result<String, worker::Error> {
    let secret = get_secret(env);
    let expiration_days = get_string_from_env(env, "AUTH_TOKEN_EXPIRATION")
        .parse::<i64>()
        .unwrap_or(1);

    let expiration = Utc::now()
        .checked_add_signed(Duration::days(expiration_days))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: username.to_owned(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| worker::Error::from(e.to_string()))
}

fn check_auth(req: &Request, env: &Env) -> Result<(), Response> {
    let username = get_string_from_env(env, "AUTH_USERNAME");
    let password = get_string_from_env(env, "AUTH_PASSWORD");

    // If no credentials configured, auth is disabled.
    if username.is_empty() || password.is_empty() {
        return Ok(());
    }

    let auth_header = match req.headers().get("Authorization") {
        Ok(Some(h)) => h,
        Ok(None) => return Err(Response::error("Missing Authorization header", 401).unwrap()),
        Err(e) => return Err(Response::error(e.to_string(), 500).unwrap()),
    };

    if !auth_header.starts_with("Bearer ") {
        return Err(Response::error("Invalid Authorization header format", 401).unwrap());
    }

    let token = &auth_header[7..];

    let secret = get_secret(env);
    let validation = Validation::default();

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ) {
        Ok(_) => Ok(()),
        Err(_) => Err(Response::error("Invalid or expired token", 401).unwrap()),
    }
}

async fn get_opt(env: Env) -> Arc<RunOpt<WasmD1, WasmAI>> {
    console_error_panic_hook::set_once();
    INIT.call_once(|| {
        match consolelog::init_with_level(log::Level::Info) {
            Err(e) => console_error!("Failed to init console log: {}", e),
            _ => console_log!("Console log initialized"),
        };
    });

    let token = get_string_from_env(&env, "TELEGRAM_TOKEN");

    let maintainer_id = get_string_from_env(&env, "MAINTAINER_ID")
        .parse::<i64>()
        .unwrap_or(0);

    let mut set = HashSet::from([maintainer_id]);

    for v in get_string_from_env(&env, "ALLOW_USERS")
        .split(",")
        .map(|v| return v.parse::<i64>().unwrap_or(0))
    {
        set.insert(v);
    }

    Arc::new(RunOpt {
        allow_users: set,
        d1: WasmD1::new(&env, "DB").await,
        workers_ai: WasmAI::new(&env, "AI"),
        matainer: maintainer_id,
        bot: client_reqwest::Bot::new(&token),
        custom_llms: OpenAI::from_assets(),
    })
}

#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    if req.method() == Method::Options {
        return Response::ok("");
    }

    ctx.pass_through_on_exception();

    Router::new()
        .post_async("/login", async |mut req, ctx| {
            let username = get_string_from_env(&ctx.env, "AUTH_USERNAME");
            let password = get_string_from_env(&ctx.env, "AUTH_PASSWORD");

            if username.is_empty() || password.is_empty() {
                return Response::error("Authentication not configured", 500);
            }

            let body: Result<LoginRequest, _> = req.json().await;
            match body {
                Ok(creds) => {
                    if creds.username == username && creds.password == password {
                        match create_token(&ctx.env, &username) {
                            Ok(token) => Response::from_json(&LoginResponse { token }),
                            Err(e) => Response::error(e.to_string(), 500),
                        }
                    } else {
                        Response::error("Invalid credentials", 401)
                    }
                }
                Err(_) => Response::error("Invalid request body", 400),
            }
        })
        .on_async("/tgbot/register", async |req, ctx| {
            if let Err(resp) = check_auth(&req, &ctx.env) {
                return Ok(resp);
            }

            let url = format!("https://{}/tgbot", req.url()?.host().unwrap());

            let opt = get_opt(ctx.env).await;

            tg::set_webhook(&opt.bot, url.as_ref(), opt.matainer)
                .await
                .map_err(|e| worker::Error::from(e.to_string()))?;

            Response::ok(format!("register telegram bot to {} successful", url))
        })
        .on_async("/d1/create_table", async |req, ctx| {
            if let Err(resp) = check_auth(&req, &ctx.env) {
                return Ok(resp);
            }

            let opt = get_opt(ctx.env).await;
            opt.d1
                .create_table()
                .await
                .map_err(|e| worker::Error::from(e.to_string()))?;
            Response::ok(format!("create table [words] successful"))
        })
        .post_async("/tgbot", async |mut req, ctx| {
            let opt = get_opt(ctx.env).await;

            let update = req.json::<Update>().await?;

            debug!("body: {:?}", update);

            return match tg::handle(opt, update).await {
                Ok(_) => {
                    debug!("Update was handled by bot.");
                    Response::ok("Update was handled by bot.")
                }
                Err(e) => {
                    error!("Update was not handled by bot: {}", e);
                    Response::ok(format!("Update was not handled by bot: {}", e))
                }
            };
        })
        .post_async("/word/:path", async |mut req, ctx| {
            if let Err(resp) = check_auth(&req, &ctx.env) {
                return Ok(resp);
            }

            let opt = get_opt(ctx.env).await;

            info!("new word request, path: {}", req.url()?.path());

            let body = match req.bytes().await {
                Ok(v) => v,
                Err(e) => return Response::error(e.to_string(), 500),
            };

            let words = match opt.route(req.url()?.path(), body).await {
                Ok(v) => v,
                Err(e) => return Response::error(e.to_string(), 500),
            };

            Response::ok(String::from_utf8_lossy(&words).to_string())
        })
        .on_async("/", async |req, ctx| {
            ctx.env.assets("ASSETS")?.fetch_request(req).await
        })
        .or_else_any_method_async("/*catchall", async |req, ctx| {
            ctx.env.assets("ASSETS")?.fetch_request(req).await
        })
        .run(req, env)
        .await
}

#[event(scheduled)]
pub async fn scheduled(_: ScheduledEvent, env: Env, _: ScheduleContext) {
    let opt = get_opt(env).await;

    match send_random_word(opt).await {
        Err(e) => {
            error!("Error: {}", e);
        }
        Ok(_) => {}
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
