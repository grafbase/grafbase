mod tcp;

pub use tcp::TcpTransport;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::database_definition::ScalarType;

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

    async fn parameterized_query(&self, query: &str, params: Vec<Value>) -> crate::Result<Vec<Value>>;

    fn connection_string(&self) -> &str;

    async fn query(&self, query: &str) -> crate::Result<Vec<Value>> {
        self.parameterized_query(query, Vec::new()).await
    }

    async fn execute(&self, query: &str) -> crate::Result<i64> {
        self.parameterized_execute(query, Vec::new()).await
    }
}

pub fn map_result<T: DeserializeOwned + Send>(values: Vec<Value>) -> Vec<T> {
    values
        .into_iter()
        .map(|value| serde_json::from_value::<T>(value).expect("should deserialize to expected type"))
        .collect()
}
