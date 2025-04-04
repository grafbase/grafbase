mod conversion;

use super::{
    HostPgConnection, HostPgRow, HostPgTransaction, PgBoundValue, PgConnection, PgRow, PgTransaction, PgValueTree,
};
use crate::WasiState;
use sqlx::{Column, Row};
use wasmtime::component::Resource;

impl HostPgConnection for WasiState {
    async fn query(
        &mut self,
        self_: Resource<PgConnection>,
        query: String,
        (params, tree): (Vec<PgBoundValue>, PgValueTree),
    ) -> wasmtime::Result<Result<Vec<Resource<PgRow>>, String>> {
        let connection = self.get_mut(&self_)?;
        let mut query = sqlx::query(&query);

        for param in params.into_iter() {
            query = conversion::bind_value(query, param.value, param.type_, &tree, param.is_array);
        }

        match query.fetch_all(connection.as_mut()).await {
            Ok(rows) => {
                let mut result = Vec::with_capacity(rows.len());

                for row in rows {
                    result.push(self.push_resource(row)?);
                }

                Ok(Ok(result))
            }
            Err(err) => Ok(Err(err.to_string())),
        }
    }

    async fn execute(
        &mut self,
        self_: Resource<PgConnection>,
        query: String,
        (params, tree): (Vec<PgBoundValue>, PgValueTree),
    ) -> wasmtime::Result<Result<u64, String>> {
        let connection = self.get_mut(&self_)?;
        let mut query = sqlx::query(&query);

        for param in params.into_iter() {
            query = conversion::bind_value(query, param.value, param.type_, &tree, param.is_array);
        }

        match query.execute(connection.as_mut()).await {
            Ok(result) => Ok(Ok(result.rows_affected())),
            Err(err) => Ok(Err(err.to_string())),
        }
    }

    async fn drop(&mut self, rep: Resource<PgConnection>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;

        Ok(())
    }
}

impl HostPgTransaction for WasiState {
    async fn query(
        &mut self,
        self_: Resource<PgTransaction>,
        query: String,
        (params, tree): (Vec<PgBoundValue>, PgValueTree),
    ) -> wasmtime::Result<Result<Vec<Resource<PgRow>>, String>> {
        let tx = self.get_mut(&self_)?;
        let mut query = sqlx::query(&query);

        for param in params.into_iter() {
            query = conversion::bind_value(query, param.value, param.type_, &tree, param.is_array);
        }

        match query.fetch_all(tx.as_mut()).await {
            Ok(rows) => {
                let mut result = Vec::with_capacity(rows.len());

                for row in rows {
                    result.push(self.push_resource(row)?);
                }

                Ok(Ok(result))
            }
            Err(err) => Ok(Err(err.to_string())),
        }
    }

    async fn execute(
        &mut self,
        self_: Resource<PgTransaction>,
        query: String,
        (params, tree): (Vec<PgBoundValue>, PgValueTree),
    ) -> wasmtime::Result<Result<u64, String>> {
        let tx = self.get_mut(&self_)?;
        let mut query = sqlx::query(&query);

        for param in params.into_iter() {
            query = conversion::bind_value(query, param.value, param.type_, &tree, param.is_array);
        }

        match query.execute(tx.as_mut()).await {
            Ok(result) => Ok(Ok(result.rows_affected())),
            Err(err) => Ok(Err(err.to_string())),
        }
    }

    async fn commit(&mut self, self_: Resource<PgTransaction>) -> wasmtime::Result<Result<(), String>> {
        let tx: PgTransaction = self.take_resource(self_.rep())?;

        match tx.commit().await {
            Ok(_) => Ok(Ok(())),
            Err(err) => Ok(Err(err.to_string())),
        }
    }

    async fn rollback(&mut self, self_: Resource<PgTransaction>) -> wasmtime::Result<Result<(), String>> {
        let tx: PgTransaction = self.take_resource(self_.rep())?;

        match tx.rollback().await {
            Ok(_) => Ok(Ok(())),
            Err(err) => Ok(Err(err.to_string())),
        }
    }

    async fn drop(&mut self, rep: Resource<PgTransaction>) -> wasmtime::Result<()> {
        let tx: PgTransaction = self.take_resource(rep.rep())?;

        match tx.rollback().await {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}

impl HostPgRow for WasiState {
    async fn columns(&mut self, self_: Resource<PgRow>) -> wasmtime::Result<Vec<String>> {
        let row = self.get(&self_)?;
        Ok(row.columns().iter().map(|c| c.name().to_string()).collect())
    }

    async fn as_bytes(
        &mut self,
        self_: Resource<PgRow>,
        index: u64,
    ) -> wasmtime::Result<Result<Option<Vec<u8>>, String>> {
        let row = self.get(&self_)?;

        match row.try_get_raw(index as usize) {
            Ok(data) => Ok(Ok(data.as_bytes().ok().map(|b| b.to_vec()))),
            Err(e) => Ok(Err(e.to_string())),
        }
    }

    async fn len(&mut self, self_: Resource<PgRow>) -> wasmtime::Result<u64> {
        let row = self.get(&self_)?;
        Ok(row.len() as u64)
    }

    async fn drop(&mut self, rep: Resource<PgRow>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
