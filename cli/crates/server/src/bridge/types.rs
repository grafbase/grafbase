use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::vec::IntoIter;

/// a single sql statement to be executed or grouped for a transaction
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub sql: String,
    pub values: Option<Vec<String>>,
    #[serde(flatten)]
    pub kind: Option<OperationKind>,
}

impl Operation {
    pub fn iter_variables(&self) -> IntoIter<String> {
        self.values.clone().unwrap_or_default().into_iter()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ConstraintKind {
    Unique,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Constraint {
    pub kind: ConstraintKind,
    // intentionally left opaque, the worker populates this for error
    // reporting and knows the structure of the values based on the [`OperationKind`].
    // should ideally have been `Box<serde_json::value::RawValue>` to avoid parsing,
    // but that has numerous issues that cause this not to work: https://github.com/serde-rs/json/issues?q=is%3Aissue+is%3Aopen+883+779+599+545+
    pub reporting_data: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum OperationKind {
    Constraint(Constraint),
}

#[derive(Deserialize, Debug)]
pub struct Mutation {
    pub mutations: Vec<Operation>,
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

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct RecordDocument {
    pub id: String,
    pub document: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub raw_query: String,
    pub limit: u64,
    pub entity_type: String,
    pub schema: SearchSchema,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub matching_records: Vec<String>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct SearchSchema {
    pub fields: Vec<SearchField>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct SearchField {
    pub name: String,
    pub scalar: SearchScalar,
}

#[allow(clippy::upper_case_acronyms)] // for URL which has the same name as the GraphQL scalar.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum SearchScalar {
    URL,
    Email,
    PhoneNumber,
    String,
    Date,
    DateTime,
    Timestamp,
    Int,
    Float,
    Boolean,
    IPAddress,
}
