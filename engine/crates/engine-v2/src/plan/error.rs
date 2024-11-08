#[derive(thiserror::Error, Debug)]
pub enum PlanError {
    #[error("Internal Error")]
    InternalError,
    #[error(transparent)]
    QueryPlanning(#[from] query_planning::Error),
}
