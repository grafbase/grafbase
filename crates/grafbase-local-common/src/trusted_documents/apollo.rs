#![allow(unused)]

/// See [the Apollo docs](https://www.apollographql.com/docs/graphos/operations/persisted-queries/#manifest-format).
#[derive(Debug, serde::Deserialize)]
pub struct ApolloOperationManifest {
    pub(super) format: String,
    pub(super) version: u32,
    pub(super) operations: Vec<ApolloOperation>,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct ApolloOperation {
    pub(super) id: String,
    pub(super) body: String,
    pub(super) name: String,
    pub(super) r#type: String,
}
