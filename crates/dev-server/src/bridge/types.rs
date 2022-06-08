use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::vec::IntoIter;

#[derive(Deserialize)]
pub struct Payload {
    pub query: String,
    pub variables: Option<Vec<String>>,
}

impl Payload {
    pub fn iter_variables(&self) -> IntoIter<String> {
        self.variables.clone().unwrap_or_default().into_iter()
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub id: String,
    pub r#type: String,
    pub document: Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
