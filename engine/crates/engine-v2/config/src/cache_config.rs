use federated_graph::{FieldId, ObjectId};

#[derive(Default, Debug, Hash, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct CacheConfig {
    /// Scope is public, default is false.
    pub public: bool,

    /// Cache max age, default is 0.
    pub max_age: usize,

    /// Cache stale_while_revalidate, default is 0.
    pub stale_while_revalidate: usize,
}

#[derive(Hash, PartialEq, Eq, Ord, PartialOrd, serde::Deserialize, serde::Serialize)]
pub enum CacheConfigTarget {
    Object(ObjectId),
    Field(FieldId),
}
