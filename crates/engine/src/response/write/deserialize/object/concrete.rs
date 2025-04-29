use id_newtypes::IdRange;
use operation::PositionedResponseKey;
use schema::ObjectDefinitionId;
use serde::{
    Deserializer,
    de::{DeserializeSeed, IgnoredAny, MapAccess, Unexpected, Visitor},
};
use std::fmt;
use walker::Walk;

use crate::{
    prepare::{ConcreteShape, ConcreteShapeId, FieldShapeId, FieldShapeRecord, ObjectIdentifier},
    response::{
        GraphqlError, ResponseObject, ResponseObjectId, ResponseObjectRef, ResponseValue, ResponseValueId,
        value::ResponseObjectField,
        write::deserialize::{SeedContext, field::FieldSeed, key::Key},
    },
};

pub(crate) struct ConcreteShapeSeed<'ctx, 'seed> {
    ctx: &'seed SeedContext<'ctx>,
    parent_field: &'ctx FieldShapeRecord,
    is_required: bool,
    shape_id: ConcreteShapeId,
    known_definition_id: Option<ObjectDefinitionId>,
}

impl<'ctx, 'seed> ConcreteShapeSeed<'ctx, 'seed> {
    pub fn new(
        ctx: &'seed SeedContext<'ctx>,
        parent_field: &'ctx FieldShapeRecord,
        is_required: bool,
        shape_id: ConcreteShapeId,
    ) -> Self {
        Self {
            ctx,
            parent_field,
            is_required,
            shape_id,
            known_definition_id: None,
        }
    }

    pub fn new_with_known_object_definition_id(
        ctx: &'seed SeedContext<'ctx>,
        parent_field: &'ctx FieldShapeRecord,
        is_required: bool,
        shape_id: ConcreteShapeId,
        object_definition_id: ObjectDefinitionId,
    ) -> Self {
        Self {
            ctx,
            parent_field,
            is_required,
            shape_id,
            known_definition_id: Some(object_definition_id),
        }
    }
}

impl<'de> DeserializeSeed<'de> for ConcreteShapeSeed<'_, '_> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let shape = self.shape_id.walk(self.ctx);
        let object_id = self.ctx.subgraph_response.borrow_mut().data.reserve_object_id();

        Ok(self.post_process_fields_seed_result(
            shape,
            object_id,
            ConcreteShapeFieldsSeed::new(self.ctx, shape, object_id, self.known_definition_id)
                .deserialize(deserializer)?,
        ))
    }
}

impl<'ctx> ConcreteShapeSeed<'ctx, '_> {
    // later we could also support visit_struct by using the schema as the reference structure.
    pub(super) fn visit_map<'de, A>(&self, map: A) -> Result<ResponseValue, A::Error>
    where
        A: MapAccess<'de>,
    {
        let shape = self.shape_id.walk(self.ctx);
        let object_id = self.ctx.subgraph_response.borrow_mut().data.reserve_object_id();

        Ok(self.post_process_fields_seed_result(
            shape,
            object_id,
            ConcreteShapeFieldsSeed::new(self.ctx, shape, object_id, self.known_definition_id).visit_map(map)?,
        ))
    }

    fn post_process_fields_seed_result(
        &self,
        shape: ConcreteShape<'ctx>,
        object_id: ResponseObjectId,
        object: ObjectValue,
    ) -> ResponseValue {
        match object {
            ObjectValue::Some { definition_id, fields } => {
                let mut resp = self.ctx.subgraph_response.borrow_mut();
                resp.data
                    .put_object(object_id, ResponseObject::new(definition_id, fields));

                if let Some(definition_id) = definition_id {
                    // If the parent field won't be sent back to the client, there is no need to bother
                    // with inaccessible.
                    if self.parent_field.key.query_position.is_some()
                        && definition_id.walk(self.ctx.schema).is_inaccessible()
                    {
                        resp.propagate_null(&self.ctx.path());
                    }
                    if let Some(set_id) = shape.set_id {
                        resp.push_object_ref(
                            set_id,
                            ResponseObjectRef {
                                id: object_id,
                                path: self.ctx.path().clone(),
                                definition_id,
                            },
                        );
                    }
                }

                object_id.into()
            }
            ObjectValue::Null => {
                if self.is_required {
                    tracing::error!(
                        "invalid type: null, expected an object at path '{}'",
                        self.ctx.display_path()
                    );
                    if self.parent_field.key.query_position.is_some() {
                        let mut resp = self.ctx.subgraph_response.borrow_mut();
                        let path = self.ctx.path();
                        resp.propagate_null(&path);
                        resp.push_error(
                            GraphqlError::invalid_subgraph_response()
                                .with_path(path)
                                .with_location(self.parent_field.id.walk(self.ctx).location()),
                        );
                    }
                    ResponseValue::Unexpected
                } else {
                    ResponseValue::Null
                }
            }
            ObjectValue::Error(error) => {
                if self.parent_field.key.query_position.is_some() {
                    let mut resp = self.ctx.subgraph_response.borrow_mut();
                    let path = self.ctx.path();
                    // If not required, we don't need to propagate as Unexpected is equivalent to
                    // null for users.
                    if self.is_required {
                        resp.propagate_null(&path);
                    }
                    resp.push_error(
                        error
                            .with_path(path)
                            .with_location(self.parent_field.id.walk(self.ctx).location()),
                    );
                }
                ResponseValue::Unexpected
            }
            ObjectValue::Unexpected => ResponseValue::Unexpected,
        }
    }
}

pub(crate) struct ConcreteShapeFieldsSeed<'ctx, 'seed> {
    ctx: &'seed SeedContext<'ctx>,
    object_id: ResponseObjectId,
    has_error: bool,
    object_identifier: ObjectIdentifier,
    field_shape_ids: IdRange<FieldShapeId>,
    typename_response_keys: &'ctx [PositionedResponseKey],
}

impl<'ctx, 'seed> ConcreteShapeFieldsSeed<'ctx, 'seed> {
    pub fn new(
        ctx: &'seed SeedContext<'ctx>,
        shape: ConcreteShape<'ctx>,
        object_id: ResponseObjectId,
        definition_id: Option<ObjectDefinitionId>,
    ) -> Self {
        ConcreteShapeFieldsSeed {
            ctx,
            object_id,
            has_error: shape.has_errors(),
            object_identifier: definition_id.map(ObjectIdentifier::Known).unwrap_or(shape.identifier),
            field_shape_ids: shape.field_shape_ids,
            typename_response_keys: &shape.as_ref().typename_response_keys,
        }
    }
}

pub(crate) enum ObjectValue {
    Null,
    Some {
        definition_id: Option<ObjectDefinitionId>,
        fields: Vec<ResponseObjectField>,
    },
    Error(GraphqlError),
    Unexpected,
}

impl<'de> DeserializeSeed<'de> for ConcreteShapeFieldsSeed<'_, '_> {
    type Value = ObjectValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl ConcreteShapeFieldsSeed<'_, '_> {
    fn unexpected_type(&self, value: Unexpected<'_>) -> <Self as Visitor<'_>>::Value {
        tracing::error!(
            "invalid type: {}, expected an object at path '{}'",
            value,
            self.ctx.display_path()
        );
        ObjectValue::Error(GraphqlError::invalid_subgraph_response().with_path(self.ctx.path().as_ref()))
    }
}

impl<'de> Visitor<'de> for ConcreteShapeFieldsSeed<'_, '_> {
    type Value = ObjectValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("any value?")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let schema = self.ctx.schema;
        let mut response_fields = Vec::with_capacity(self.field_shape_ids.len() + self.typename_response_keys.len());
        let mut maybe_object_definition_id = None;

        match self.object_identifier {
            ObjectIdentifier::Known(id) => {
                maybe_object_definition_id = Some(id);
                self.visit_fields(&mut map, &mut response_fields)?;
            }
            ObjectIdentifier::Anonymous => {
                self.visit_fields(&mut map, &mut response_fields)?;
            }
            ObjectIdentifier::UnionTypename(id) => {
                if let Some(definition_id) = self.visit_fields_with_typename_detection(
                    &mut map,
                    &schema[id].possible_types_ordered_by_typename_ids,
                    &mut response_fields,
                )? {
                    maybe_object_definition_id = Some(definition_id);
                } else {
                    return Ok(ObjectValue::Error(GraphqlError::invalid_subgraph_response()));
                }
            }
            ObjectIdentifier::InterfaceTypename(id) => {
                if let Some(definition_id) = self.visit_fields_with_typename_detection(
                    &mut map,
                    &schema[id].possible_types_ordered_by_typename_ids,
                    &mut response_fields,
                )? {
                    maybe_object_definition_id = Some(definition_id);
                } else {
                    return Ok(ObjectValue::Error(GraphqlError::invalid_subgraph_response()));
                }
            }
        }

        self.post_process(&mut response_fields);

        if !self.typename_response_keys.is_empty() {
            let Some(object_id) = maybe_object_definition_id else {
                tracing::error!(
                    "Expected to have the object definition id to generate __typename at path '{}'",
                    self.ctx.display_path()
                );
                return Ok(ObjectValue::Error(GraphqlError::invalid_subgraph_response()));
            };
            let name_id = schema[object_id].name_id;
            for key in self.typename_response_keys {
                response_fields.push(ResponseObjectField {
                    key: *key,
                    value: name_id.into(),
                });
            }
        }

        Ok(ObjectValue::Some {
            definition_id: maybe_object_definition_id,
            fields: response_fields,
        })
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Bool(v)))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Signed(v)))
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Other(&format!("integer {v}"))))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Unsigned(v)))
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Other(&format!("integer {v}"))))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Float(v)))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v.encode_utf8(&mut [0u8; 4]))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Str(v)))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Bytes(v)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ObjectValue::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ObjectValue::Null)
    }

    fn visit_newtype_struct<D>(self, _: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // newtype_struct are used by custom deserializers to indicate that an error happened, but
        // was already treated.
        Ok(ObjectValue::Unexpected)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        // Try discarding the rest of the list, we might be able to use other parts of
        // the response.
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(self.unexpected_type(Unexpected::Seq))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let _ = data.variant::<IgnoredAny>()?;
        Ok(self.unexpected_type(Unexpected::Enum))
    }
}

impl<'ctx> ConcreteShapeFieldsSeed<'ctx, '_> {
    fn post_process(&self, response_fields: &mut Vec<ResponseObjectField>) {
        if self.has_error {
            let mut must_propagate_null = false;
            let mut resp = self.ctx.subgraph_response.borrow_mut();
            for field_shape in self.field_shape_ids.walk(self.ctx) {
                for error in field_shape.errors() {
                    resp.push_error(
                        error
                            .clone()
                            .with_path((self.ctx.path(), field_shape.key))
                            .with_location(field_shape.as_ref().id.walk(self.ctx).location()),
                    );

                    if field_shape.wrapping.is_required() {
                        must_propagate_null = true;
                    } else {
                        response_fields.push(ResponseObjectField {
                            key: field_shape.key,
                            value: ResponseValue::Null,
                        });
                    }
                }
            }
            if must_propagate_null {
                resp.propagate_null(&self.ctx.path());
                return;
            }
        }

        if response_fields.len() < self.field_shape_ids.len() {
            let n = response_fields.len();
            let keys = self.ctx.response_keys();
            for field_shape in self.field_shape_ids.walk(self.ctx) {
                if field_shape.is_skipped() {
                    continue;
                }
                if !response_fields[0..n].iter().any(|field| field.key == field_shape.key) {
                    if field_shape.wrapping.is_required() {
                        // If part of the query fields the user requested. We don't propagate null
                        // for extra fields.
                        if field_shape.key.query_position.is_some() {
                            if field_shape.key.response_key == field_shape.expected_key {
                                tracing::error!(
                                    "Error decoding response from upstream: Missing required field named '{}' at path '{}'",
                                    &keys[field_shape.expected_key],
                                    self.ctx.display_path()
                                )
                            } else {
                                tracing::error!(
                                    "Error decoding response from upstream: Missing required field named '{}' (expected: '{}') at path '{}'",
                                    &keys[field_shape.key.response_key],
                                    &keys[field_shape.expected_key],
                                    self.ctx.display_path()
                                )
                            }
                            let mut resp = self.ctx.subgraph_response.borrow_mut();
                            let path = self.ctx.path();
                            resp.propagate_null(&path);
                            resp.push_error(
                                GraphqlError::invalid_subgraph_response()
                                    .with_path((path, field_shape.key))
                                    .with_location(field_shape.as_ref().id.walk(self.ctx).location()),
                            );

                            return;
                        }
                    } else {
                        response_fields.push(ResponseObjectField {
                            key: field_shape.key,
                            value: ResponseValue::Null,
                        });
                    }
                }
            }
        }
    }

    fn visit_fields_with_typename_detection<'de, A: MapAccess<'de>>(
        &self,
        map: &mut A,
        possible_types_ordered_by_typename: &[ObjectDefinitionId],
        response_fields: &mut Vec<ResponseObjectField>,
    ) -> Result<Option<ObjectDefinitionId>, A::Error> {
        let schema = self.ctx.schema;
        let keys = self.ctx.response_keys();
        let fields = &self.ctx.prepared_operation.cached.shapes[self.field_shape_ids];
        let mut offset = 0;
        let mut maybe_object_definition_id: Option<ObjectDefinitionId> = None;
        while let Some(key) = map.next_key::<Key<'_>>()? {
            let key = key.as_ref();
            // Improves significantly (a few %) the performance to use the unchecked version.
            // SAFETY: offset is initialized 0 which always work. Later on it's only incremented by
            //         at most 1 if we find an element within [offset..]. So offset + 1 is still equal or
            //         lower than the fields length.
            if let Some(pos) = unsafe { fields.get_unchecked(offset..) }
                .iter()
                .position(|field| &keys[field.expected_key] == key)
            {
                self.visit_field(map, &fields[offset + pos], response_fields)?;
                // Each key in the JSON is unique, it's an object. So if we found it once, we won't
                // re-find it. This means that if the found field is the first one, we can increase
                // the offset to ignore for the next key.
                // Worst-case scenario if the field re-appears, we'll ignore the data.
                offset += (pos == 0) as usize;
            // This supposes that the discriminant is never part of the schema.
            } else if maybe_object_definition_id.is_none() && key == "__typename" {
                let value = map.next_value::<Key<'_>>()?;
                let typename = value.as_ref();
                maybe_object_definition_id = possible_types_ordered_by_typename
                    .binary_search_by(|probe| schema[schema[*probe].name_id].as_str().cmp(typename))
                    .map(|i| possible_types_ordered_by_typename[i])
                    .ok();
            } else {
                // Try discarding the next value, we might be able to use other parts of
                // the response.
                map.next_value::<IgnoredAny>()?;
            }
        }

        Ok(maybe_object_definition_id)
    }

    fn visit_fields<'de, A: MapAccess<'de>>(
        &self,
        map: &mut A,
        response_fields: &mut Vec<ResponseObjectField>,
    ) -> Result<(), A::Error> {
        let keys = self.ctx.response_keys();
        let fields = &self.ctx.prepared_operation.cached.shapes[self.field_shape_ids];
        let mut offset = 0;
        while let Some(key) = map.next_key::<Key<'_>>()? {
            let key = key.as_ref();
            // Improves significantly (a few %) the performance to use the unchecked version.
            // SAFETY: offset is initialized 0 which always work. Later on it's only incremented by
            //         at most 1 if we find an element within [offset..]. So offset + 1 is still equal or
            //         lower than the fields length.
            if let Some(pos) = unsafe { fields.get_unchecked(offset..) }
                .iter()
                .position(|field| &keys[field.expected_key] == key)
            {
                self.visit_field(map, &fields[offset + pos], response_fields)?;
                // Each key in the JSON is unique, it's an object. So if we found it once, we won't
                // re-find it. This means that if the found field is the first one, we can increase
                // the offset to ignore for the next key.
                // Worst-case scenario if the field re-appears, we'll ignore the data.
                offset += (pos == 0) as usize;
            } else {
                // Try discarding the next value, we might be able to use other parts of
                // the response.
                map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(())
    }

    fn visit_field<'de, A: MapAccess<'de>>(
        &self,
        map: &mut A,
        field: &'ctx FieldShapeRecord,
        response_fields: &mut Vec<ResponseObjectField>,
    ) -> Result<(), A::Error> {
        self.ctx.path_mut().push(ResponseValueId::Field {
            object_id: self.object_id,
            key: field.key,
            nullable: field.wrapping.is_nullable(),
        });
        let result = map.next_value_seed(FieldSeed {
            ctx: self.ctx,
            field,
            wrapping: field.wrapping.to_mutable(),
        });
        self.ctx.path_mut().pop();
        let value = result?;

        response_fields.push(ResponseObjectField { key: field.key, value });

        Ok(())
    }
}
