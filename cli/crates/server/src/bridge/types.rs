use common::types::UdfKind;
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

#[derive(Deserialize, Debug)]
pub struct UdfInvocation {
    pub request_id: String,
    pub name: String,
    pub payload: serde_json::Value,
    pub udf_kind: UdfKind,
}

#[serde_with::serde_as]
#[derive(Deserialize, Debug)]
pub enum LogEventType {
    OperationStarted {
        name: Option<String>,
    },
    OperationCompleted {
        name: Option<String>,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        duration: std::time::Duration,
        r#type: common::types::OperationType,
    },
    BadRequest {
        name: Option<String>,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        duration: std::time::Duration,
    },
    NestedRequest {
        url: String,
        method: String,
        #[serde(rename = "statusCode")]
        status_code: u16,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        duration: std::time::Duration,
        body: Option<String>,
        #[serde(rename = "contentType")]
        content_type: Option<String>,
    },
    SqlQuery {
        successful: bool,
        sql: String,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        duration: std::time::Duration,
        body: Option<String>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub request_id: String,
    pub r#type: LogEventType,
}
