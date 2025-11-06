use hjcommon::d1::{DB, Error as D1Error, SQL};
use log::info;
use serde::Deserialize;
use std::sync::Arc;
use worker::{D1Database, Env};

pub struct Error(worker::Error);

impl From<worker::Error> for Error {
    fn from(err: worker::Error) -> Self {
        Error(err)
    }
}

impl From<Error> for D1Error {
    fn from(err: Error) -> Self {
        D1Error(err.0.to_string())
    }
}

pub struct WasmD1 {
    d1: Option<Arc<D1Database>>,
}

impl WasmD1 {
    pub async fn new(env: &Env, binding: &str) -> WasmD1 {
        WasmD1 {
            d1: match env.d1(binding) {
                Ok(v) => Some(Arc::new(v)),
                Err(_) => None,
            },
        }
    }

    fn get_d1(&self) -> Result<Arc<D1Database>, D1Error> {
        match self.d1.as_ref() {
            Some(v) => Ok(v.clone()),
            None => Err(D1Error("d1 is not initialized".to_string())),
        }
    }

    pub async fn exec<T>(&self, sql: SQL<'_>) -> Result<Vec<T>, D1Error>
    where
        T: for<'a> Deserialize<'a>,
    {
        let prepare_statement = self
            .get_d1()?
            .prepare(sql.sql())
            .bind(&sql.params())
            .map_err(|v| Error(v))?;

        let result = match prepare_statement.run().await {
            Ok(v) => Ok(v),
            // Err(worker::Error::D1(e)) if e.cause().contains("no such table") => {
            //     self.create_table().await?;
            //     prepare_statement.run().await
            // }
            Err(e) => Err(e),
        }
        .map_err(|v| Error(v))?;

        info!(
            "exec sql [{}], args: [{:?}], result: {:?}",
            sql.sql(),
            sql.params::<String>(),
            result,
        );

        if !result.error().is_none() {
            return Err(D1Error(result.error().unwrap().to_string()));
        }

        Ok(result.results::<T>().unwrap())
    }
}

impl Clone for WasmD1 {
    fn clone(&self) -> Self {
        WasmD1 {
            d1: self.d1.clone(),
        }
    }
}

impl DB for WasmD1 {
    async fn exec<T>(&self, sql: SQL<'_>) -> Result<Vec<T>, D1Error>
    where
        T: for<'a> Deserialize<'a>,
    {
        self.exec(sql).await
    }
}
