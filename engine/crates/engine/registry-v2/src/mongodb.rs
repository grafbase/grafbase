#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MongoDBConfiguration {
    pub name: String,
    pub api_key: String,
    pub url: String,
    pub data_source: String,
    pub database: String,
    pub namespace: bool,
}
