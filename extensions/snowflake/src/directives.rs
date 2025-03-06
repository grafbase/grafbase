#[derive(serde::Deserialize)]
pub(crate) struct SnowflakeQueryDirective {
    pub(crate) sql: String,
    pub(crate) bindings: Option<String>,
}
