use std::{cell::RefCell, collections::HashMap, sync::atomic::AtomicBool};

use serde::{de::DeserializeSeed, Deserializer};

use super::{
    deserialize::{SeedContext, UpdateSeed},
    ExecutorOutput, ExpectedObjectFieldsWriter, GroupedFieldWriter,
};
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
                        // TODO: should include locations & path of all root fields retrieved by
                        // the plan.
                        locations: vec![],
                        path: Some(self.boundary_item.response_path.clone()),
                        extensions: HashMap::with_capacity(0),
                    });
                }
                self.data
                    .push_error_to_propagate(self.boundary_item.response_path.clone());
            }
        }
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for ResponseObjectWriter<'ctx> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        UpdateSeed {
            ctx: SeedContext {
                walker: self.walker,
                data: RefCell::new(self.data),
                propagating_error: AtomicBool::new(false),
                expectations: &self.output.expectations,
                attribution: &self.output.attribution,
            },
            boundary_item: self.boundary_item,
            expected: &self.output.expectations.root_selection_set,
        }
        .deserialize(deserializer)
    }
}
