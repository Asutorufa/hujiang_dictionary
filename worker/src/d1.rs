use async_trait::async_trait;
use d1_orm::{DatabaseExecutor, DatabaseValue, Error, Query};
use std::sync::Arc;
use worker::wasm_bindgen::JsValue;
use worker::{D1Database, Env};

#[derive(Clone)]
pub struct WasmD1 {
    d1: Arc<D1Database>,
}

impl WasmD1 {
    pub async fn new(env: &Env, binding: &str) -> WasmD1 {
        WasmD1 {
            d1: Arc::new(env.d1(binding).expect("D1 binding not found")),
        }
    }
}

fn to_js_value(v: &DatabaseValue) -> JsValue {
    match v {
        DatabaseValue::Null => JsValue::NULL,
        DatabaseValue::Bool(b) => JsValue::from_bool(*b),
        DatabaseValue::Int(i) => JsValue::from_f64(*i as f64),
        DatabaseValue::UInt(u) => JsValue::from_f64(*u as f64),
        DatabaseValue::Real(f) => JsValue::from_f64(*f),
        DatabaseValue::Text(s) => JsValue::from_str(s),
        DatabaseValue::Blob(_) => JsValue::NULL, // Blob support skipped
    }
}

#[async_trait(?Send)]
impl DatabaseExecutor for WasmD1 {
    async fn execute<Q>(&self, query: Q) -> Result<(), Error>
    where
        Q: Query,
    {
        let (sql, params) = query.build()?;
        let js_params: Vec<JsValue> = params.iter().map(to_js_value).collect();

        let stmt = self
            .d1
            .prepare(&sql)
            .bind(&js_params)
            .map_err(|e| Error::Other(e.to_string()))?;
        stmt.run().await.map_err(|e| Error::Other(e.to_string()))?;
        Ok(())
    }

    async fn query_all<T, Q>(&self, query: Q) -> Result<Vec<T>, Error>
    where
        T: serde::de::DeserializeOwned,
        Q: Query,
    {
        let (sql, params) = query.build()?;
        let js_params: Vec<JsValue> = params.iter().map(to_js_value).collect();

        let stmt = self
            .d1
            .prepare(&sql)
            .bind(&js_params)
            .map_err(|e| Error::Other(e.to_string()))?;
        let result = stmt.run().await.map_err(|e| Error::Other(e.to_string()))?;

        result
            .results::<T>()
            .map_err(|e| Error::Other(e.to_string()))
    }

    async fn query_first<T, Q>(&self, query: Q) -> Result<Option<T>, Error>
    where
        T: serde::de::DeserializeOwned,
        Q: Query,
    {
        let (sql, params) = query.build()?;
        let js_params: Vec<JsValue> = params.iter().map(to_js_value).collect();

        let stmt = self
            .d1
            .prepare(&sql)
            .bind(&js_params)
            .map_err(|e| Error::Other(e.to_string()))?;
        let result = stmt
            .first::<T>(None)
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
        Ok(result)
    }

    async fn execute_batch<Q>(&self, queries: Vec<Q>) -> Result<(), Error>
    where
        Q: Query,
    {
        let mut batch = vec![];
        for q in queries {
            let (sql, params) = q.build()?;
            let js_params: Vec<JsValue> = params.iter().map(to_js_value).collect();
            let stmt = self
                .d1
                .prepare(&sql)
                .bind(&js_params)
                .map_err(|e| Error::Other(e.to_string()))?;
            batch.push(stmt);
        }
        self.d1
            .batch(batch)
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
        Ok(())
    }
}
