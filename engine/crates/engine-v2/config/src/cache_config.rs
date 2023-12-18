use federated_graph::{FieldId, ObjectId};

#[derive(Default, Debug, Hash, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct CacheConfig {
    pub max_age: usize,
    pub stale_while_revalidate: usize,
}

#[derive(Hash, PartialEq, Eq, Ord, PartialOrd, serde::Deserialize, serde::Serialize)]
pub enum CacheConfigTarget {
    Object(ObjectId),
    Field(FieldId),
}
