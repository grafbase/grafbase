use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Response {
    result_set_meta_data: Option<ResultSetMetaData>,
    pub(crate) data: Option<Vec<Vec<serde_json::Value>>>,
    pub(crate) code: String,
    statement_status_url: Option<String>,
    pub(crate) request_id: Option<String>,
    pub(crate) sql_state: String,
    pub(crate) statement_handle: String,
    pub(crate) message: String,
    created_on: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResultSetMetaData {
    num_rows: i64,
    format: String,
    partition_info: Vec<PartitionInfo>,
    row_type: Vec<RowType>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PartitionInfo {
    row_count: i64,
    uncompressed_size: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RowType {
    name: String,
    database: String,
    schema: String,
    table: String,
    nullable: bool,
    length: Option<i64>,
    #[serde(rename = "type")]
    type_field: String,
    scale: Option<i64>,
    precision: Option<i64>,
    byte_length: Option<i64>,
    collation: Option<String>,
}
