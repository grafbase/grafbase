#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum AutomaticPersistedQuery {
    V1 { query: String },
}
