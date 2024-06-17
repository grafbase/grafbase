use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde_json::Value;

pub use ext::{TransportExt, TransportTransactionExt};
pub use tcp::{DirectTcpTransport, Transaction, TransportTransaction};
#[cfg(feature = "pooling")]
pub use tcp::{PooledTcpConnection, PooledTcpTransport, PooledTransaction, PoolingConfig};

use crate::{database_definition::ScalarType, error::Error};

mod ext;
mod tcp;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Column {
    name: String,
    column_id: i16,
    data_type_id: u32,
}

impl Column {
    pub fn r#type(&self) -> ScalarType {
        ScalarType::from(self.data_type_id)
    }

    pub fn column_id(&self) -> i16 {
        self.column_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Transport: Send + Sync {
    async fn parameterized_execute(&self, query: &str, params: Vec<Value>) -> crate::Result<i64>;

    fn parameterized_query<'a>(&'a self, query: &'a str, params: Vec<Value>) -> BoxStream<'a, Result<Value, Error>>;

    fn connection_string(&self) -> &str;

    fn query<'a>(&'a self, query: &'a str) -> BoxStream<'a, Result<Value, Error>> {
        self.parameterized_query(query, Vec::new())
    }

    async fn execute(&self, query: &str) -> crate::Result<i64> {
        self.parameterized_execute(query, Vec::new()).await
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Transport for Arc<dyn Transport> {
    async fn parameterized_execute(&self, query: &str, params: Vec<Value>) -> crate::Result<i64> {
        self.as_ref().parameterized_execute(query, params).await
    }

    fn parameterized_query<'a>(&'a self, query: &'a str, params: Vec<Value>) -> BoxStream<'a, Result<Value, Error>> {
        self.as_ref().parameterized_query(query, params)
    }

    fn connection_string(&self) -> &str {
        self.as_ref().connection_string()
    }

    fn query<'a>(&'a self, query: &'a str) -> BoxStream<'a, Result<Value, Error>> {
        self.as_ref().parameterized_query(query, Vec::new())
    }

    async fn execute(&self, query: &str) -> crate::Result<i64> {
        self.as_ref().parameterized_execute(query, Vec::new()).await
    }
}
