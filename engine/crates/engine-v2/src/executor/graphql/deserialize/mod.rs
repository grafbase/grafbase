mod unique_root;

pub use unique_root::*;

#[derive(serde::Deserialize)]
pub struct UpstreamGraphqlError {
    pub message: String,
    #[serde(default)]
    pub locations: serde_json::Value,
    #[serde(default)]
    pub path: serde_json::Value,
    #[serde(default)]
    pub extensions: serde_json::Value,
}
