use super::{HostPgPool, PgPool, PgPoolOptions};
use crate::WasiState;

use dashmap::Entry;
use sqlx::{
    Postgres, Transaction,
    pool::{PoolConnection, PoolOptions},
};
use std::time::Duration;
use wasmtime::component::Resource;

impl HostPgPool for WasiState {
    async fn connect(
        &mut self,
        name: String,
        url: String,
        options: PgPoolOptions,
    ) -> wasmtime::Result<Result<Resource<sqlx::Pool<Postgres>>, String>> {
        let pool = match self.postgres_pools().entry(name) {
            Entry::Occupied(occupied_entry) => occupied_entry.get().clone(),
            Entry::Vacant(vacant_entry) => {
                let pool = match create_new_pool(url, options).await {
                    Ok(pool) => pool,
                    Err(err) => return Ok(Err(err)),
                };

                vacant_entry.insert(pool.clone());

                pool
            }
        };

        Ok(Ok(self.push_resource(pool)?))
    }

    async fn acquire(
        &mut self,
        self_: Resource<PgPool>,
    ) -> wasmtime::Result<Result<Resource<PoolConnection<Postgres>>, String>> {
        let pool = self.get_mut(&self_)?;

        let connection = match pool.acquire().await {
            Ok(connection) => connection,
            Err(err) => return Ok(Err(err.to_string())),
        };

        Ok(Ok(self.push_resource(connection)?))
    }

    async fn begin_transaction(
        &mut self,
        self_: Resource<PgPool>,
    ) -> wasmtime::Result<Result<Resource<Transaction<'static, Postgres>>, String>> {
        let pool = self.get_mut(&self_)?;

        let transaction = match pool.begin().await {
            Ok(tx) => tx,
            Err(err) => return Ok(Err(err.to_string())),
        };

        Ok(Ok(self.push_resource(transaction)?))
    }

    async fn drop(&mut self, rep: Resource<PgPool>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}

async fn create_new_pool(url: String, options: PgPoolOptions) -> Result<sqlx::Pool<Postgres>, String> {
    let PgPoolOptions {
        max_connections,
        min_connections,
        idle_timeout_ms,
        acquisition_timeout_ms,
        max_lifetime_ms,
    } = options;

    let mut options = PoolOptions::<Postgres>::new();

    options = match max_connections {
        Some(max_connections) => options.max_connections(max_connections),
        None => options,
    };

    options = match min_connections {
        Some(min_connections) => options.min_connections(min_connections),
        None => options,
    };

    options = match idle_timeout_ms {
        Some(idle_timeout_ms) => options.idle_timeout(Duration::from_millis(idle_timeout_ms)),
        None => options,
    };

    options = match acquisition_timeout_ms {
        Some(acquisition_timeout_ms) => options.acquire_timeout(Duration::from_millis(acquisition_timeout_ms)),
        None => options,
    };

    options = match max_lifetime_ms {
        Some(max_lifetime_ms) => options.max_lifetime(Duration::from_millis(max_lifetime_ms)),
        None => options,
    };

    let pool = options.connect(&url).await.map_err(|e| e.to_string())?;

    Ok(pool)
}
