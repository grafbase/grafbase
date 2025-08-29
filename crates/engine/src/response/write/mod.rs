mod deserialize;
mod merge;
mod part;

use std::sync::Arc;

use grafbase_telemetry::graphql::{GraphqlOperationAttributes, GraphqlResponseStatus};
use schema::{ObjectDefinitionId, Schema};
use walker::Walk;

use super::{
    DataParts, ErrorPartBuilder, ErrorParts, ExecutedResponse, GraphqlError, Response, ResponseData, ResponseObject,
    ResponseObjectId, ResponseObjectRef, ResponseObjectSet, ResponseValueId,
};
use crate::prepare::{OperationPlanContext, PreparedOperation, ResponseObjectSetId};
pub(crate) use deserialize::*;
pub(crate) use part::*;

pub(crate) struct ResponseBuilder<'ctx> {
    // will be None if an error propagated up to the root.
    pub(in crate::response) schema: &'ctx Arc<Schema>,
    pub(in crate::response) operation: &'ctx Arc<PreparedOperation>,
    pub(super) root: Option<(ResponseObjectId, ObjectDefinitionId)>,
    pub(super) data_parts: DataParts,
    pub(super) error_parts: ErrorParts,
    errors: ErrorPartBuilder<'ctx>,
}

impl<'ctx> ResponseBuilder<'ctx> {
    pub fn new(schema: &'ctx Arc<Schema>, operation: &'ctx Arc<PreparedOperation>) -> Self {
        let root_object_definition_id = operation.cached.operation.root_object_id;
        let mut data_parts = DataParts::default();
        let mut part = data_parts.new_part();
        let fields_id = part.push_owned_sorted_fields_by_key(Vec::new());
        let root_id = part.push_object(ResponseObject::new(Some(root_object_definition_id), fields_id));
        data_parts.insert(part);

        Self {
            schema,
            operation,
            root: Some((root_id, root_object_definition_id)),
            data_parts,
            error_parts: ErrorParts::default(),
            errors: ErrorPartBuilder::new(operation),
        }
    }

    pub fn push_error(&mut self, error: impl Into<GraphqlError>) {
        self.errors.push(error.into());
    }

    pub fn propagate_null(&mut self, path: &[ResponseValueId]) {
        let Some(value_id) = path.iter().rev().find(|value| value.is_nullable()) else {
            self.root = None;
            return;
        };
        self.data_parts[value_id.part_id()].make_inaccessible(*value_id);
    }

    pub fn make_inacessible(&mut self, value_id: ResponseValueId) {
        self.data_parts[value_id.part_id()].make_inaccessible(value_id);
    }

    pub fn create_root_part(&mut self) -> (ResponseObjectRef, ResponsePartBuilder<'ctx>) {
        let Some(root_parent_object) = self.root_response_object() else {
            unreachable!("I think?")
        };
        let part = self.create_part();
        (root_parent_object, part)
    }

    pub fn create_part(&mut self) -> ResponsePartBuilder<'ctx> {
        ResponsePartBuilder::new(self.schema, self.operation, self.data_parts.new_part())
    }

    pub fn root_response_object(&self) -> Option<ResponseObjectRef> {
        self.root.map(|(response_object_id, object_id)| ResponseObjectRef {
            id: response_object_id,
            path: Vec::new(),
            definition_id: object_id,
        })
    }

    pub fn ingest(&mut self, part: ResponsePartBuilder<'ctx>) -> PartIngestionResult {
        let new_part_id = part.data.id;
        self.data_parts.insert(part.data);
        self.error_parts.push(part.errors);

        if part.propagated_null_up_to_root {
            self.root = None;
            return PartIngestionResult::SubgraphFailure;
        }

        let mut has_ingested_data = false;
        let ctx = OperationPlanContext::from((self.schema.as_ref(), self.operation.as_ref()));
        for update in part.object_updates {
            match update {
                ObjectUpdate::Fields(id, fields) => {
                    has_ingested_data = true;
                    self.recursive_merge_object_in_place(id, new_part_id, fields);
                }
                ObjectUpdate::Default(id, default_field_ids) => {
                    self.merge_with_default_object(id, default_field_ids.walk(ctx));
                }
            }
        }

        // Must be done after object updates, as it'll add nulls if the field doesn't exist yet.
        // And we don't merge null with something else than null, as we treat it as an indicator
        // that something went wrong.
        for value_id in part.propagated_null_at {
            self.data_parts[value_id.part_id()].make_inaccessible(value_id);
        }

        if has_ingested_data {
            PartIngestionResult::Data {
                response_object_sets: part.object_sets,
            }
        } else {
            debug_assert!(part.object_sets.is_empty());
            PartIngestionResult::SubgraphFailure
        }
    }

    pub fn graphql_status(&self) -> GraphqlResponseStatus {
        if self.errors.is_empty() && self.error_parts.is_empty() {
            GraphqlResponseStatus::Success
        } else {
            GraphqlResponseStatus::FieldError {
                count: (self.errors.len() + self.error_parts.len()) as u64,
                data_is_null: self.root.is_none(),
            }
        }
    }

    pub fn build(mut self, operation_attributes: GraphqlOperationAttributes) -> Response {
        self.error_parts.push(self.errors);

        Response::Executed(ExecutedResponse {
            schema: self.schema.clone(),
            operation: self.operation.clone(),
            operation_attributes,
            data: self.root.map(|(root, _)| ResponseData {
                root,
                parts: self.data_parts,
            }),
            errors: self.error_parts,
            extensions: Default::default(),
        })
    }
}

pub(crate) enum PartIngestionResult {
    Data {
        response_object_sets: Vec<(ResponseObjectSetId, ResponseObjectSet)>,
    },
    SubgraphFailure,
}
