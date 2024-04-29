#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CorsConfig {
    pub max_age: Option<u32>,
    pub allowed_origins: Option<Vec<String>>,
}
