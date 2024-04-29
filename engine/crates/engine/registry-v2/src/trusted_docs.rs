#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrustedDocuments {
    pub bypass_header_name: Option<String>,
    pub bypass_header_value: Option<String>,
}
