#[derive(thiserror::Error, Debug)]
pub enum PlanError {
    #[error(transparent)]
    QueryPlanning(#[from] query_planning::Error),
}
