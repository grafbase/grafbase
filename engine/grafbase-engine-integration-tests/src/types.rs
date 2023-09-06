#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseData<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<Error>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    pub message: String,
}
