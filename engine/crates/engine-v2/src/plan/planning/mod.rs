use std::collections::BTreeMap;

use schema::Schema;

use crate::{
    operation::{Operation, Variables},
    response::GraphqlError,
};

mod boundary;
mod collect;
mod logic;
mod planner;
mod walker_ext;

use super::OperationPlan;

pub type PlanningResult<T> = Result<T, PlanningError>;

#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    #[error("Could not plan fields: {}", .missing.join(", "))]
    CouldNotPlanAnyField {
        missing: Vec<String>,
        query_path: Vec<String>,
    },
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<PlanningError> for GraphqlError {
    fn from(error: PlanningError) -> Self {
        let message = error.to_string();
        let query_path = match error {
            PlanningError::CouldNotPlanAnyField { query_path, .. } => query_path
                .into_iter()
                .map(serde_json::Value::String)
                .collect::<Vec<_>>(),
            PlanningError::InternalError { .. } => vec![],
        };

        GraphqlError {
            message,
            locations: vec![],
            path: None,
            extensions: BTreeMap::from([("queryPath".into(), serde_json::Value::Array(query_path))]),
        }
    }
}

impl From<String> for PlanningError {
    fn from(error: String) -> Self {
        PlanningError::InternalError(error)
    }
}

impl From<&str> for PlanningError {
    fn from(error: &str) -> Self {
        PlanningError::InternalError(error.to_string())
    }
}

pub(super) fn plan_operation(
    schema: &Schema,
    variables: &Variables,
    operation: Operation,
) -> PlanningResult<OperationPlan> {
    let mut planner = planner::Planner::new(schema, variables, operation);
    planner.plan_all_fields()?;
    planner.finalize_operation()
}
