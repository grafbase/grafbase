mod neon;

pub use neon::NeonTransport;

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

#[derive(Debug, Clone)]
pub struct QueryResponse<T> {
    columns: Vec<Column>,
    row_count: usize,
    rows: Vec<T>,
}

impl<T> QueryResponse<T> {
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }

    pub fn into_rows(self) -> impl ExactSizeIterator<Item = T> {
        self.rows.into_iter()
    }

    pub fn into_single_row(self) -> Option<T> {
        self.into_rows().next()
    }

    pub fn clone_rows(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.rows.clone()
    }
}

#[derive(Debug, Clone)]
pub struct ExecuteResponse {
    row_count: usize,
}

impl ExecuteResponse {
    pub fn row_count(&self) -> usize {
        self.row_count
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Transport {
    async fn query<T>(&self, query: &str) -> crate::Result<QueryResponse<T>>
    where
        T: DeserializeOwned + Send,
    {
        self.parameterized_query(query, Vec::new()).await
    }

    async fn execute(&self, query: &str) -> crate::Result<ExecuteResponse> {
        self.parameterized_execute(query, Vec::new()).await
    }

    async fn parameterized_query<T>(&self, query: &str, params: Vec<Value>) -> crate::Result<QueryResponse<T>>
    where
        T: DeserializeOwned + Send;

    async fn parameterized_execute(&self, query: &str, params: Vec<Value>) -> crate::Result<ExecuteResponse>;

    fn connection_string(&self) -> &str;
}
