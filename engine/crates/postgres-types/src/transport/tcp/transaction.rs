use crate::{error::Error, transport::Transport};
use async_trait::async_trait;
use futures::stream::BoxStream;
use serde_json::Value;
use tokio_postgres::Transaction;

use super::executor;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Transport for Transaction<'_> {
    fn parameterized_query<'a>(&'a self, query: &'a str, params: Vec<Value>) -> BoxStream<'a, Result<Value, Error>> {
        executor::query(self, query, params)
    }

    async fn parameterized_execute(&self, query: &str, params: Vec<Value>) -> crate::Result<i64> {
        executor::execute(self, query, params).await
    }

    // we don't care about this in a tx
    fn connection_string(&self) -> &str {
        ""
    }
}
