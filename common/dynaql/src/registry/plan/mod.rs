use super::{SchemaID, SchemaIDGenerator};

#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum SchemaPlan {
    /// Create a projection by selecting only a subset of the data.
    Projection(PlanProjection),
    /// Get the Entities related to this node
    Related(PlanRelated),
}

impl SchemaPlan {
    pub fn projection(fields: Vec<String>) -> Self {
        Self::Projection(PlanProjection { fields })
    }

    pub fn related(from: Option<SchemaID>, to: SchemaID, relation_name: Option<String>) -> Self {
        Self::Related(PlanRelated {
            from,
            to,
            relation_name,
        })
    }
}

/// Describe the fields projected
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PlanProjection {
    pub(crate) fields: Vec<String>,
}

/// Describe the relation
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PlanRelated {
    pub(crate) from: Option<SchemaID>,
    pub(crate) to: SchemaID,
    pub(crate) relation_name: Option<String>,
}
