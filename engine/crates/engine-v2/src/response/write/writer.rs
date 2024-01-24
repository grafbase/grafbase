use super::{ExecutorOutput, ExpectedObjectFieldsWriter, GroupedFieldWriter};
use crate::{
    plan::PlanOutput,
    request::PlanWalker,
    response::{GraphqlError, ResponseBoundaryItem, ResponseValue},
};

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("Propagating error")]
    ErrorPropagation,
    #[error(transparent)]
    Any(#[from] anyhow::Error),
}

impl From<&str> for WriteError {
    fn from(value: &str) -> Self {
        Self::Any(anyhow::anyhow!(value.to_string()))
    }
}

impl From<String> for WriteError {
    fn from(value: String) -> Self {
        Self::Any(anyhow::anyhow!(value))
    }
}

pub type WriteResult<T> = Result<T, WriteError>;

pub(crate) struct ResponseObjectWriter<'a> {
    walker: PlanWalker<'a>,
    data: &'a mut ExecutorOutput,
    output: &'a PlanOutput,
    boundary_item: &'a ResponseBoundaryItem,
}

impl<'a> ResponseObjectWriter<'a> {
    pub fn new(
        walker: PlanWalker<'a>,
        data: &'a mut ExecutorOutput,
        output: &'a PlanOutput,
        boundary_item: &'a ResponseBoundaryItem,
    ) -> Self {
        Self {
            walker,
            data,
            output,
            boundary_item,
        }
    }

    pub fn update_with(self, f: impl Fn(GroupedFieldWriter<'_>) -> WriteResult<ResponseValue>) {
        let mut writer = ExpectedObjectFieldsWriter {
            walker: self.walker,
            data: self.data,
            path: &self.boundary_item.response_path,
            object_id: self.boundary_item.object_id,
            selection_set: &self.output.expectations.root_selection_set,
        };
        match writer.write_fields(f) {
            Ok(fields) => {
                self.data.push_update(super::ResponseObjectUpdate {
                    id: self.boundary_item.response_object_id,
                    fields,
                });
            }
            Err(err) => {
                if let WriteError::Any(err) = err {
                    self.data.push_error(GraphqlError {
                        message: err.to_string(),
                        path: Some(self.boundary_item.response_path.clone()),
                        ..Default::default()
                    });
                }
                self.data
                    .push_error_path_to_propagate(self.boundary_item.response_path.clone());
            }
        }
    }
}
