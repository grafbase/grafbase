use schema::ObjectDefinitionId;
use serde::de::{DeserializeSeed, MapAccess, Visitor};
use walker::Walk;

use crate::{
    prepare::{ConcreteShape, ConcreteShapeId, FieldShapeRecord},
    response::{
        GraphqlError, ResponseObject, ResponseObjectId, ResponseObjectRef, ResponseValue, write::deserialize::SeedState,
    },
};

use super::{ConcreteShapeFieldsSeed, ObjectFields};

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
        let shape = self.shape_id.walk(self.state);
        let object_id = self.state.response.borrow_mut().data.reserve_object_id();

        Ok(self.ingest_object_fields(
            shape,
            object_id,
            ConcreteShapeFieldsSeed::new(self.state, shape, object_id, self.known_definition_id)
                .deserialize(deserializer)?,
        ))
    }
}

impl<'ctx> ConcreteShapeSeed<'ctx, '_, '_> {
    // later we could also support visit_struct by using the schema as the reference structure.
    pub(super) fn visit_map<'de, A>(&self, map: A) -> Result<ResponseValue, A::Error>
    where
        A: MapAccess<'de>,
    {
        let shape = self.shape_id.walk(self.state);
        let object_id = self.state.response.borrow_mut().data.reserve_object_id();
        let fields =
            ConcreteShapeFieldsSeed::new(self.state, shape, object_id, self.known_definition_id).visit_map(map)?;
        Ok(self.ingest_object_fields(shape, object_id, fields))
    }

    fn ingest_object_fields(
        &self,
        shape: ConcreteShape<'ctx>,
        object_id: ResponseObjectId,
        fields: ObjectFields,
    ) -> ResponseValue {
        match fields {
            ObjectFields::Some { definition_id, fields } => {
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
            ObjectFields::Null => {
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
            ObjectFields::Error(error) => {
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
