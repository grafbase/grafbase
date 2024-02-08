#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum AutomaticallyPersistedQuery {
    V1 { query: String },
}
