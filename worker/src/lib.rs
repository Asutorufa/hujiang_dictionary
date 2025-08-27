pub mod ai;
pub mod d1;
pub mod d1v1;

use crate::{ai::WasmAI, d1::WasmD1};
use frankenstein::{client_reqwest, updates::Update};
use hjcommon::d1::DBv2;
use hjcommon::opts::RunOpt;
use hjcommon::tg::{self, send_random_word};
use log::{debug, error, info};
use std::ops::Deref;
use std::sync::Once;
use std::{collections::HashSet, sync::Arc};
use worker::*;

static INIT: Once = Once::new();

async fn get_opt(env: Arc<Env>) -> Arc<RunOpt<WasmD1, WasmAI>> {
    console_error_panic_hook::set_once();
    INIT.call_once(|| {
        match console_log::init_with_level(log::Level::Debug) {
            Err(e) => console_error!("Failed to init console log: {}", e),
            _ => console_log!("Console log initialized"),
        };
    });

    let token = env
        .var("TELEGRAM_TOKEN")
        .unwrap()
        .as_ref()
        .as_string()
        .unwrap();

    let maintainer_id = env
        .var("MAINTAINER_ID")
        .unwrap()
        .as_ref()
        .as_string()
        .unwrap()
        .parse::<i64>()
        .unwrap();

    let allow_users = match env.var("ALLOW_USERS") {
        Ok(v) => v
            .as_ref()
            .as_string()
            .unwrap_or("".to_string())
            .split(",")
            .map(|v| return v.parse::<i64>().unwrap_or(0))
            .collect::<Vec<_>>(),
        _ => vec![],
    };

    let mut set = HashSet::from([maintainer_id]);

    for v in allow_users {
        set.insert(v);
    }

    Arc::new(RunOpt {
        allow_users: set,
        d1: WasmD1::new(env.clone(), "DB").await,
        workers_ai: WasmAI::new(env.clone(), "AI"),
        matainer: maintainer_id,
        bot: client_reqwest::Bot::new(&token),
    })
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    if req.method() == Method::Options {
        return Response::ok("");
    }

    let env = Arc::new(env);
    let opt = get_opt(env.clone()).await;

    let mut router = Router::new();

    router = router.on_async("/tgbot/register", async |req, _ctx| {
        let url = format!("https://{}/tgbot", req.url()?.host().unwrap());

        tg::set_webhook(&opt.bot, url.as_ref(), opt.matainer)
            .await
            .map_err(|e| worker::Error::from(e.to_string()))?;

        Response::ok(format!("register telegram bot to {} successful", url))
    });

    router = router.on_async("/d1/create_table", async |_, _ctx| {
        opt.d1
            .create_table()
            .await
            .map_err(|e| worker::Error::from(e.to_string()))?;
        Response::ok(format!("create table [words] successful"))
    });

    router = router.post_async("/tgbot", async |mut req, _ctx| {
        let update = req.json::<Update>().await?;

        debug!("body: {:?}", update);

        return match tg::handle(opt.clone(), update).await {
            Ok(_) => {
                debug!("Update was handled by bot.");
                Response::ok("Update was handled by bot.")
            }
            Err(e) => {
                error!("Update was not handled by bot: {}", e);
                Response::ok(format!("Update was not handled by bot: {}", e))
            }
        };
    });

    router = router.post_async("/word/:path", async |mut req, _ctx| {
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
    });

    let resp = router
        .run(req.clone().unwrap(), env.deref().clone())
        .await?;

    if resp.status_code() == 404 {
        return env
            .assets("ASSETS")?
            .fetch_request(req.clone().unwrap())
            .await;
    }

    Ok(resp)
}

#[event(scheduled)]
pub async fn scheduled(_: ScheduledEvent, env: Env, _: ScheduleContext) {
    let opt = get_opt(env.into()).await;

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
