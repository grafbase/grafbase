#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct CustomResolver {
    pub resolver_name: String,
}
