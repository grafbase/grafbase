#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum SchemaPlan {
    Projection(PlanProjection),
}

impl SchemaPlan {
    pub fn projection(fields: Vec<String>) -> Self {
        Self::Projection(PlanProjection { fields })
    }
}

/// Describe the fields projected
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PlanProjection {
    pub fields: Vec<String>,
}
