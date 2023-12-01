use std::{cell::RefCell, collections::HashMap, sync::atomic::AtomicBool};

use schema::SchemaWalker;
use serde::{de::DeserializeSeed, Deserializer};

use super::{
    deserialize::{SeedContext, UpdateSeed},
    ExpectedSelectionSetWriter, GroupedFieldWriter, ResponsePartBuilder,
};
use crate::{
    execution::Variables,
    plan::ExpectedSelectionSet,
    request::Operation,
    response::{GraphqlError, ResponseObjectRoot, ResponseValue},
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

pub struct ResponseObjectWriter<'a> {
    schema_walker: SchemaWalker<'a, ()>,
    operation: &'a Operation,
    variables: &'a Variables<'a>,
    data: &'a mut ResponsePartBuilder,
    root: ResponseObjectRoot,
    expectation: &'a ExpectedSelectionSet,
}

impl<'a> ResponseObjectWriter<'a> {
    pub fn new(
        schema_walker: SchemaWalker<'a, ()>,
        operation: &'a Operation,
        variables: &'a Variables<'a>,
        data: &'a mut ResponsePartBuilder,
        root: ResponseObjectRoot,
        expectation: &'a ExpectedSelectionSet,
    ) -> Self {
        Self {
            schema_walker,
            operation,
            variables,
            data,
            root,
            expectation,
        }
    }

    pub fn update_with(self, f: impl Fn(GroupedFieldWriter<'_>) -> WriteResult<ResponseValue>) {
        let writer = ExpectedSelectionSetWriter {
            schema_walker: self.schema_walker,
            operation: self.operation,
            variables: self.variables,
            data: self.data,
            path: &self.root.path,
            selection_set: self.expectation,
        };
        match writer.write_fields(self.root.object_id, f) {
            Ok(fields) => {
                self.data.push_update(super::ResponseObjectUpdate {
                    id: self.root.id,
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
                        path: Some(self.root.path.clone()),
                        extensions: HashMap::with_capacity(0),
                    });
                }
                self.data.push_error_to_propagate(self.root.path.clone());
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
                schema_walker: self.schema_walker,
                operation: self.operation,
                data: RefCell::new(self.data),
                propagating_error: AtomicBool::new(false),
            },
            root: self.root,
            expected: self.expectation,
        }
        .deserialize(deserializer)
    }
}
