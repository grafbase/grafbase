mod ext;
mod tcp;

use async_trait::async_trait;
pub use ext::TransportExt;
use futures::stream::BoxStream;
use serde_json::Value;
pub use tcp::{TcpTransport, Transaction};

use crate::{database_definition::ScalarType, error::Error};

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
    async fn close(self) -> crate::Result<()>;

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
