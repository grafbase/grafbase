use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::vec::IntoIter;

#[derive(Deserialize, Debug)]
pub struct Payload {
    pub query: String,
    pub variables: Option<Vec<String>>,
}

impl Payload {
    pub fn iter_variables(&self) -> IntoIter<String> {
        self.variables.clone().unwrap_or_default().into_iter()
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub pk: String,
    pub sk: String,
    pub gsi1pk: String,
    pub gsi1sk: String,
    pub gsi2pk: String,
    pub gsi2sk: String,
    pub entity_type: String,
    pub relation_names: Value,
    pub document: Value,
    pub created_at: String,
    pub updated_at: String,
}
