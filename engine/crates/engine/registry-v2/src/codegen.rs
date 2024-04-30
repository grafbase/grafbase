#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CodegenConfig {
    pub enabled: bool,
    pub path: Option<String>,
}
