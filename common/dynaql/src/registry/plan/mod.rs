#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum SchemaPlan {
    Projection(PlanProjection),
}

/// Describe the fields projected
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PlanProjection {
    field: Vec<String>,
}
