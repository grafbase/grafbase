use schema::ObjectDefinitionId;
use serde::de::{DeserializeSeed, MapAccess, Visitor};
use walker::Walk;

use crate::{
    prepare::{ConcreteShape, ConcreteShapeId, FieldShapeRecord},
    response::{
        GraphqlError, ResponseFieldsSortedByKey, ResponseObject, ResponseObjectId, ResponseObjectRef, ResponseValue,
        write::deserialize::SeedState,
    },
};

use super::{ConcreteShapeFieldsSeed, FieldsDeserializationResult};

pub(crate) struct ConcreteShapeSeed<'ctx, 'parent, 'state> {
    state: &'state SeedState<'ctx, 'parent>,
    parent_field: &'ctx FieldShapeRecord,
    is_required: bool,
    shape_id: ConcreteShapeId,
    known_definition_id: Option<ObjectDefinitionId>,
}

impl<'ctx, 'parent, 'state> ConcreteShapeSeed<'ctx, 'parent, 'state> {
    pub fn new(
        state: &'state SeedState<'ctx, 'parent>,
        parent_field: &'ctx FieldShapeRecord,
        is_required: bool,
        shape_id: ConcreteShapeId,
    ) -> Self {
        Self {
            state,
            parent_field,
            is_required,
            shape_id,
            known_definition_id: None,
        }
    }

    pub fn new_with_known_object_definition_id(
        state: &'state SeedState<'ctx, 'parent>,
        parent_field: &'ctx FieldShapeRecord,
        is_required: bool,
        shape_id: ConcreteShapeId,
        object_definition_id: ObjectDefinitionId,
    ) -> Self {
        Self {
            state,
            parent_field,
            is_required,
            shape_id,
            known_definition_id: Some(object_definition_id),
        }
    }
}

impl<'de> DeserializeSeed<'de> for ConcreteShapeSeed<'_, '_, '_> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.handle(|seed| seed.deserialize(deserializer))
    }
}

impl<'ctx> ConcreteShapeSeed<'ctx, '_, '_> {
    // later we could also support visit_struct by using the schema as the reference structure.
    pub(super) fn visit_map<'de, A>(&self, map: A) -> Result<ResponseValue, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.handle(|seed| seed.visit_map(map))
    }

    fn handle<E>(
        &self,
        ingest: impl FnOnce(ConcreteShapeFieldsSeed<'_, '_, '_, '_>) -> Result<FieldsDeserializationResult, E>,
    ) -> Result<ResponseValue, E> {
        let shape = self.shape_id.walk(self.state);
        let object_id = self
            .state
            .response
            .borrow_mut()
            .data
            .push_empty_object(self.known_definition_id);

        // If there is no set_id it means, no further plan / response modifier has this object as
        // an input so it won't be modified later on with the exception of shared nested roots
        // which isn't that common.
        if shape.set_id.is_none() {
            let (fields_id, mut response_fields) = self.state.response.borrow_mut().data.take_next_shared_fields();
            let offset = response_fields.len();
            let seed = ConcreteShapeFieldsSeed::new(
                self.state,
                shape,
                object_id,
                self.known_definition_id,
                &mut response_fields,
            );

            let serde_result = ingest(seed);

            response_fields[offset..].sort_unstable_by_key(|field| field.key);
            let limit = response_fields.len() - offset;

            self.state
                .response
                .borrow_mut()
                .data
                .restore_shared_fields(fields_id, response_fields);

            Ok(self.ingest_object_fields(
                shape,
                object_id,
                serde_result?,
                ResponseFieldsSortedByKey::Slice {
                    fields_id,
                    offset: offset as u32,
                    limit: limit as u16,
                },
            ))
        } else {
            let mut response_fields = Vec::new();
            let seed = ConcreteShapeFieldsSeed::new(
                self.state,
                shape,
                object_id,
                self.known_definition_id,
                &mut response_fields,
            );

            let result = ingest(seed)?;

            response_fields.sort_unstable_by_key(|field| field.key);
            let fields_id = self
                .state
                .response
                .borrow_mut()
                .data
                .push_owned_sorted_fields_by_key(response_fields);
            Ok(self.ingest_object_fields(shape, object_id, result, fields_id.into()))
        }
    }

    fn ingest_object_fields(
        &self,
        shape: ConcreteShape<'ctx>,
        object_id: ResponseObjectId,
        result: FieldsDeserializationResult,
        fields: ResponseFieldsSortedByKey,
    ) -> ResponseValue {
        match result {
            FieldsDeserializationResult::Some { definition_id } => {
                let mut resp = self.state.response.borrow_mut();
                resp.data
                    .put_object(object_id, ResponseObject::new(definition_id, fields));

                if let Some(definition_id) = definition_id {
                    let path = self.state.path();
                    // If the parent field won't be sent back to the client, there is no need to bother
                    // with inaccessible.
                    if self.state.should_report_error_for(self.parent_field)
                        && definition_id.walk(self.state.schema).is_inaccessible()
                    {
                        resp.propagate_null(&path);
                    }
                    if let Some(set_id) = shape.set_id {
                        let (parent_path, local_path) = path;
                        let mut path = Vec::with_capacity(parent_path.len() + local_path.len());
                        path.extend_from_slice(parent_path);
                        path.extend_from_slice(local_path.as_ref());
                        resp.push_object_ref(
                            set_id,
                            ResponseObjectRef {
                                id: object_id,
                                path,
                                definition_id,
                            },
                        );
                    }
                }

                object_id.into()
            }
            FieldsDeserializationResult::Null => {
                if self.is_required {
                    tracing::error!(
                        "invalid type: null, expected an object at path '{}'",
                        self.state.display_path()
                    );
                    if self.state.should_report_error_for(self.parent_field) {
                        let mut resp = self.state.response.borrow_mut();
                        let path = self.state.path();
                        resp.propagate_null(&path);
                        resp.errors.push(
                            GraphqlError::invalid_subgraph_response()
                                .with_path(path)
                                .with_location(self.parent_field.id.walk(self.state).location()),
                        );
                    }
                    ResponseValue::Unexpected
                } else {
                    ResponseValue::Null
                }
            }
            FieldsDeserializationResult::Error(error) => {
                if self.state.should_report_error_for(self.parent_field) {
                    let mut resp = self.state.response.borrow_mut();
                    let path = self.state.path();
                    // If not required, we don't need to propagate as Unexpected is equivalent to
                    // null for users.
                    if self.is_required {
                        resp.propagate_null(&path);
                    }
                    resp.errors.push(
                        error
                            .with_path(path)
                            .with_location(self.parent_field.id.walk(self.state).location()),
                    );
                }
                ResponseValue::Unexpected
            }
        }
    }
}
