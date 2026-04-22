pub mod ai;
pub mod consolelog;

use crate::ai::WasmAI;
use async_trait::async_trait;
use d1_orm::{DatabaseExecutor, Query, QueryExt, SqlBackend};
use hjcommon::d1::migrations;
use hjcommon::opts::{ConfigCache, RunOpt};
use hjcommon::tg::send_random_word;
use log::error;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Once, OnceLock};
use std::{collections::HashSet, sync::Arc};
use worker::*;

static INIT: Once = Once::new();
static ENV_CONFIG: OnceLock<EnvConfig> = OnceLock::new();
static MIGRATION_DONE: AtomicBool = AtomicBool::new(false);
static GLOBAL_CONFIG_CACHE: OnceLock<Arc<RwLock<ConfigCache>>> = OnceLock::new();

struct EnvConfig {
    auth_secret: String,
    auth_username: String,
    auth_password: String,
    auth_token_expiration: i64,
}

impl EnvConfig {
    fn from_env(env: &Env) -> Result<Self> {
        let auth_secret = get_string_from_env(env, "AUTH_SECRET");
        if auth_secret.is_empty() {
            return Err(worker::Error::from("AUTH_SECRET is required"));
        }

        Ok(Self {
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

pub struct WasmBackend;
impl SqlBackend for WasmBackend {
    type Param = worker::wasm_bindgen::JsValue;
    fn convert(value: d1_orm::types::DatabaseValue) -> Self::Param {
        match value {
            d1_orm::types::DatabaseValue::Text(s) => worker::wasm_bindgen::JsValue::from_str(&s),
            d1_orm::types::DatabaseValue::Int(i) => {
                worker::wasm_bindgen::JsValue::from_f64(i as f64)
            }
            d1_orm::types::DatabaseValue::UInt(u) => {
                worker::wasm_bindgen::JsValue::from_f64(u as f64)
            }
            d1_orm::types::DatabaseValue::Real(r) => worker::wasm_bindgen::JsValue::from_f64(r),
            d1_orm::types::DatabaseValue::Bool(b) => worker::wasm_bindgen::JsValue::from_bool(b),
            d1_orm::types::DatabaseValue::Blob(b) => js_sys::Uint8Array::from(&b[..]).into(),
            d1_orm::types::DatabaseValue::Null => worker::wasm_bindgen::JsValue::NULL,
        }
    }
}

pub struct D1(worker::D1Database);

#[async_trait(?Send)]
impl DatabaseExecutor for D1 {
    async fn query_all<T, Q>(&self, sql: Q) -> std::result::Result<Vec<T>, d1_orm::error::Error>
    where
        T: serde::de::DeserializeOwned,
        Q: Query + 'async_trait,
    {
        let (sql_str, params) = sql.build_params::<WasmBackend>()?;
        let sql_str: String = sql_str.into_owned();
        self.0
            .prepare(sql_str)
            .bind(&params)
            .map_err(|e| d1_orm::error::Error::Other(e.to_string()))?
            .all()
            .await
            .map_err(|e| d1_orm::error::Error::Other(e.to_string()))?
            .results()
            .map_err(|e| d1_orm::error::Error::Other(e.to_string()))
    }

    async fn query_first<T, Q>(
        &self,
        sql: Q,
    ) -> std::result::Result<Option<T>, d1_orm::error::Error>
    where
        T: serde::de::DeserializeOwned,
        Q: Query + 'async_trait,
    {
        let (sql_str, params) = sql.build_params::<WasmBackend>()?;
        let sql_str: String = sql_str.into_owned();
        self.0
            .prepare(sql_str)
            .bind(&params)
            .map_err(|e| d1_orm::error::Error::Other(e.to_string()))?
            .first(None)
            .await
            .map_err(|e| d1_orm::error::Error::Other(e.to_string()))
    }

    async fn execute<Q>(&self, sql: Q) -> std::result::Result<(), d1_orm::error::Error>
    where
        Q: Query + 'async_trait,
    {
        let (sql_str, params) = sql.build_params::<WasmBackend>()?;
        let sql_str: String = sql_str.into_owned();
        self.0
            .prepare(sql_str)
            .bind(&params)
            .map_err(|e| d1_orm::error::Error::Other(e.to_string()))?
            .run()
            .await
            .map_err(|e| d1_orm::error::Error::Other(e.to_string()))?;
        Ok(())
    }

    async fn execute_batch<Q>(&self, sqls: Vec<Q>) -> std::result::Result<(), d1_orm::error::Error>
    where
        Q: Query + 'async_trait,
    {
        let mut statements = Vec::with_capacity(sqls.len());
        for sql in sqls {
            let (sql_str, params) = sql.build_params::<WasmBackend>()?;
            let sql_str: String = sql_str.into_owned();
            statements.push(
                self.0
                    .prepare(sql_str)
                    .bind(&params)
                    .map_err(|e| d1_orm::error::Error::Other(e.to_string()))?,
            );
        }
        self.0
            .batch(statements)
            .await
            .map_err(|e| d1_orm::error::Error::Other(e.to_string()))?;
        Ok(())
    }
}

async fn get_opt(env: Env) -> Result<Arc<RunOpt<D1, WasmAI>>> {
    console_error_panic_hook::set_once();
    INIT.call_once(|| {
        match consolelog::init_with_level(log::Level::Info) {
            Err(e) => console_error!("Failed to init console log: {}", e),
            _ => console_log!("Console log initialized"),
        };
    });

    if ENV_CONFIG.get().is_none() {
        let c = EnvConfig::from_env(&env)?;
        // In a race, the first thread to call `set` wins. We can ignore the error.
        let _ = ENV_CONFIG.set(c);
    }
    let config = ENV_CONFIG
        .get()
        .expect("ENV_CONFIG is guaranteed to be initialized here");

    if let Ok(ai) = env.ai("AI") {
        hj_ai::workers::set_global_ai(ai);
    }

    Ok(Arc::new(RunOpt {
        d1: D1(env.d1("DB").expect("D1 binding not found")),
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
        auth_secret: config.auth_secret.clone(),
        auth_username: config.auth_username.clone(),
        auth_password: config.auth_password.clone(),
        auth_token_expiration: config.auth_token_expiration,
        config_cache: GLOBAL_CONFIG_CACHE
            .get_or_init(|| {
                Arc::new(RwLock::new(ConfigCache {
                    allow_users: Arc::new(HashSet::new()),
                    maintainer_id: 0,
                    bot: None,
                    last_updated: 0,
                    google_search_api_key: "".to_string(),
                    google_search_cx: "".to_string(),
                }))
            })
            .clone(),
    }))
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
            for header in resp.headers {
                r.headers_mut().set(&header.0, &header.1)?;
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
