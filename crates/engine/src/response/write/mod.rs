mod deserialize;
mod merge;
mod part;

use std::sync::Arc;

use grafbase_telemetry::graphql::{GraphqlOperationAttributes, GraphqlResponseStatus};
use operation::PositionedResponseKey;
use schema::{ObjectDefinitionId, Schema};
use walker::Walk;

use super::{
    DataParts, ErrorCodeCounter, ErrorPathSegment, ExecutedResponse, GraphqlError, OutputResponseObjectSets,
    ParentObjectId, ParentObjects, Response, ResponseData, ResponseObject, ResponseObjectField, ResponseObjectId,
    ResponseObjectRef, ResponseValue, ResponseValueId,
};
use crate::{
    execution::ExecutionError,
    prepare::{ObjectIdentifier, Plan, PreparedOperation},
};
pub(crate) use part::*;

pub(crate) struct ResponseBuilder<'ctx> {
    // will be None if an error propagated up to the root.
    pub(in crate::response) schema: &'ctx Arc<Schema>,
    operation: &'ctx PreparedOperation,
    pub(super) root: Option<(ResponseObjectId, ObjectDefinitionId)>,
    pub(super) data_parts: DataParts,
    errors: Vec<GraphqlError>,
}

impl<'ctx> ResponseBuilder<'ctx> {
    pub fn new(schema: &'ctx Arc<Schema>, operation: &'ctx PreparedOperation) -> Self {
        let root_object_definition_id = operation.cached.operation.root_object_id;
        let mut parts = DataParts::default();
        let mut initial_part = parts.new_part();
        let root_id = initial_part.push_object(ResponseObject::new(Some(root_object_definition_id), Vec::new()));
        parts.insert(initial_part);

        Self {
            schema,
            operation,
            root: Some((root_id, root_object_definition_id)),
            data_parts: parts,
            errors: Vec::new(),
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

    pub fn create_root_part(&mut self) -> (ParentObjectId, ResponsePart<'ctx>) {
        let root_parent_objects = Arc::new(
            ParentObjects::default().with_response_objects(Arc::new(self.root_response_object().into_iter().collect())),
        );
        let root_id = root_parent_objects.ids().next().expect("We just added the root object");
        let resp = self.create_part_for(root_parent_objects);
        (root_id, resp)
    }

    pub fn create_part_for(&mut self, parent_objects: Arc<ParentObjects>) -> ResponsePart<'ctx> {
        ResponsePart::new(self.schema, self.operation, self.data_parts.new_part(), parent_objects)
    }

    pub fn root_response_object(&self) -> Option<ResponseObjectRef> {
        self.root.map(|(response_object_id, object_id)| ResponseObjectRef {
            id: response_object_id,
            path: Vec::new(),
            definition_id: object_id,
        })
    }

    pub fn propagate_execution_error(
        &mut self,
        plan: Plan<'_>,
        root_response_object_set: Arc<ParentObjects>,
        error: ExecutionError,
    ) {
        let (any_response_key, default_fields_sorted_by_key) =
            self.extract_any_response_key_and_default_fields_sorted_by_key(plan);
        let error = GraphqlError::from(error);
        if let Some(any_response_key) = any_response_key {
            if let Some(default_fields_sorted_by_key) = &default_fields_sorted_by_key {
                for obj_ref in root_response_object_set.iter() {
                    self.errors
                        .push(error.clone().with_path((&obj_ref.path, any_response_key)));
                    self.recursive_merge_with_default_object(obj_ref.id, default_fields_sorted_by_key);
                }
            } else {
                for obj_ref in root_response_object_set.iter() {
                    self.propagate_null(&obj_ref.path);
                    self.errors
                        .push(error.clone().with_path((&obj_ref.path, any_response_key)));
                }
            }
        }
    }

    pub fn ingest(&mut self, plan: Plan<'ctx>, mut subgraph_response: ResponsePart<'ctx>) -> OutputResponseObjectSets {
        self.data_parts.insert(subgraph_response.data);

        let (any_response_key, default_fields_sorted_by_key) =
            self.extract_any_response_key_and_default_fields_sorted_by_key(plan);
        for (update, obj_ref) in subgraph_response
            .updates
            .into_iter()
            .zip(subgraph_response.parent_objects.iter())
        {
            match update {
                ObjectUpdate::Missing => {
                    if let Some(any_response_key) = any_response_key {
                        if !subgraph_response
                            .subgraph_errors
                            .iter()
                            .any(|subgraph_error| self.sugraph_error_matches_current_object(subgraph_error, obj_ref))
                        {
                            tracing::error!("Missing data from subgraph.");
                            self.errors.push(
                                GraphqlError::invalid_subgraph_response().with_path((&obj_ref.path, any_response_key)),
                            );
                        }
                        if let Some(default_fields_sorted_by_key) = &default_fields_sorted_by_key {
                            self.recursive_merge_with_default_object(obj_ref.id, default_fields_sorted_by_key);
                        } else {
                            self.propagate_null(&obj_ref.path);
                        }
                    }
                }
                ObjectUpdate::Fields(mut fields) => {
                    fields.sort_unstable_by(|a, b| a.key.cmp(&b.key));
                    self.recursive_merge_shared_object(obj_ref.id, fields);
                }
                ObjectUpdate::Error(error) => {
                    if let Some(any_response_key) = any_response_key {
                        self.errors.push(error.with_path((&obj_ref.path, any_response_key)));
                        if let Some(default_fields_sorted_by_key) = &default_fields_sorted_by_key {
                            self.recursive_merge_with_default_object(obj_ref.id, default_fields_sorted_by_key);
                        } else {
                            self.propagate_null(&obj_ref.path);
                        }
                    }
                }
                ObjectUpdate::PropagateNullWithoutError => {
                    if any_response_key.is_some() {
                        if let Some(default_fields_sorted_by_key) = &default_fields_sorted_by_key {
                            self.recursive_merge_with_default_object(obj_ref.id, default_fields_sorted_by_key);
                        } else {
                            self.propagate_null(&obj_ref.path);
                        }
                    }
                }
            }
        }
        self.errors.append(&mut subgraph_response.subgraph_errors);
        self.errors.append(&mut subgraph_response.errors);
        if subgraph_response.propagated_null_up_to_root {
            self.root = None;
        } else {
            for path in subgraph_response.propagated_null_up_to_paths {
                self.propagate_null(&path);
            }
        }

        let (ids, sets) = subgraph_response.object_sets.into_iter().unzip();
        OutputResponseObjectSets { ids, sets }
    }

    fn sugraph_error_matches_current_object(&self, error: &GraphqlError, obj_ref: &ResponseObjectRef) -> bool {
        let Some(parent_path) = &error.path else {
            return true;
        };
        if obj_ref.path.len() > parent_path.len() {
            return false;
        }

        let mut parent_path = parent_path.iter();
        let mut path = obj_ref.path.iter();
        while let Some((parent_segment, child_segment)) = parent_path.next().zip(path.next()) {
            match (parent_segment, child_segment) {
                (ErrorPathSegment::Index(i), ResponseValueId::Index { index, .. }) => {
                    if *i != (*index as usize) {
                        return false;
                    }
                }
                (ErrorPathSegment::UnknownField(name), ResponseValueId::Field { key, .. }) => {
                    if **name != self.operation.cached.operation.response_keys[*key] {
                        return false;
                    }
                }
                (ErrorPathSegment::Field(field), ResponseValueId::Field { key, .. }) => {
                    if field != &key.response_key {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        true
    }

    fn extract_any_response_key_and_default_fields_sorted_by_key(
        &self,
        plan: Plan<'_>,
    ) -> (Option<PositionedResponseKey>, Option<Vec<ResponseObjectField>>) {
        let shape = plan.shape();
        let any_response_key = shape
            .fields()
            .filter(|field| field.key.query_position.is_some())
            .map(|field| field.key)
            .min()
            .or_else(|| shape.typename_response_keys.iter().min().copied());

        let mut fields = Vec::new();
        if !shape.typename_response_keys.is_empty() {
            if let ObjectIdentifier::Known(object_id) = shape.identifier {
                let name: ResponseValue = object_id.walk(self.schema.as_ref()).as_ref().name_id.into();
                fields.extend(shape.typename_response_keys.iter().map(|&key| ResponseObjectField {
                    key,
                    value: name.clone(),
                }))
            } else {
                return (any_response_key, None);
            }
        }
        for field_shape in shape.fields() {
            if field_shape.key.query_position.is_none() {
                continue;
            }
            if field_shape.wrapping.is_required() {
                return (any_response_key, None);
            }
            fields.push(ResponseObjectField {
                key: field_shape.key,
                value: ResponseValue::Null,
            })
        }

        fields.sort_unstable_by(|a, b| a.key.cmp(&b.key));
        (any_response_key, Some(fields))
    }

    pub fn graphql_status(&self) -> GraphqlResponseStatus {
        if self.errors.is_empty() {
            GraphqlResponseStatus::Success
        } else {
            GraphqlResponseStatus::FieldError {
                count: self.errors.len() as u64,
                data_is_null: self.root.is_none(),
            }
        }
    }

    pub fn build<OnOperationResponseHookOutput>(
        self,
        operation_attributes: GraphqlOperationAttributes,
        on_operation_response_output: OnOperationResponseHookOutput,
    ) -> Response<OnOperationResponseHookOutput> {
        let error_code_counter = ErrorCodeCounter::from_errors(&self.errors);
        Response::Executed(ExecutedResponse {
            schema: self.schema.clone(),
            operation: self.operation.cached.clone(),
            operation_attributes,
            data: self.root.map(|(root, _)| ResponseData {
                root,
                parts: self.data_parts,
            }),
            errors: self.errors,
            error_code_counter,
            on_operation_response_output: Some(on_operation_response_output),
            extensions: Default::default(),
        })
    }
}
