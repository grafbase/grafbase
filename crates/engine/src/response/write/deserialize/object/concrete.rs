use id_newtypes::IdRange;
use schema::ObjectDefinitionId;
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};
use std::fmt;
use walker::Walk;

use crate::{
    response::{
        value::ResponseObjectField,
        write::deserialize::{field::FieldSeed, key::Key, SeedContext},
        ConcreteShape, ConcreteShapeId, FieldShapeId, FieldShapeRecord, GraphqlError, ObjectIdentifier,
        PositionedResponseKey, ResponseObject, ResponseObjectId, ResponseObjectRef, ResponseValue, ResponseValueId,
    },
    ErrorCode,
};

pub(crate) struct ConcreteShapeSeed<'ctx, 'seed> {
    ctx: &'seed SeedContext<'ctx>,
    shape_id: ConcreteShapeId,
    known_definition_id: Option<ObjectDefinitionId>,
}

impl<'ctx, 'seed> ConcreteShapeSeed<'ctx, 'seed> {
    pub fn new(ctx: &'seed SeedContext<'ctx>, shape_id: ConcreteShapeId) -> Self {
        Self {
            ctx,
            shape_id,
            known_definition_id: None,
        }
    }

    pub fn new_with_known_object_definition_id(
        ctx: &'seed SeedContext<'ctx>,
        shape_id: ConcreteShapeId,
        object_definition_id: ObjectDefinitionId,
    ) -> Self {
        Self {
            ctx,
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
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for ConcreteShapeSeed<'_, '_> {
    type Value = ResponseValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an object")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let shape = self.shape_id.walk(self.ctx);
        let object_id = self.ctx.writer.data().reserve_object_id();

        let (definition_id, fields) =
            ConcreteShapeFieldsSeed::new(self.ctx, shape, object_id, self.known_definition_id).visit_map(map)?;

        self.ctx
            .writer
            .data()
            .put_object(object_id, ResponseObject::new(fields));

        if let Some(set_id) = shape.set_id {
            self.ctx.writer.push_object_ref(
                set_id,
                ResponseObjectRef {
                    id: object_id,
                    path: self.ctx.path().clone(),
                    definition_id: definition_id.expect("Object id should have been identified"),
                },
            );
        }

        Ok(object_id.into())
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

impl<'de> DeserializeSeed<'de> for ConcreteShapeFieldsSeed<'_, '_> {
    type Value = (Option<ObjectDefinitionId>, Vec<ResponseObjectField>);

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for ConcreteShapeFieldsSeed<'_, '_> {
    type Value = (Option<ObjectDefinitionId>, Vec<ResponseObjectField>);

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an object")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let schema = self.ctx.schema;
        let mut response_fields = Vec::with_capacity(self.field_shape_ids.len() + self.typename_response_keys.len());

        let mut maybe_object_id = None;
        match self.object_identifier {
            ObjectIdentifier::Known(id) => {
                maybe_object_id = Some(id);
                self.visit_fields(&mut map, &mut response_fields)?;
            }
            ObjectIdentifier::Anonymous => {
                self.visit_fields(&mut map, &mut response_fields)?;
            }
            ObjectIdentifier::UnionTypename(id) => {
                maybe_object_id = Some(self.visit_fields_with_typename_detection(
                    &mut map,
                    &schema[id].possible_types_ordered_by_typename_ids,
                    &mut response_fields,
                )?);
            }
            ObjectIdentifier::InterfaceTypename(id) => {
                maybe_object_id = Some(self.visit_fields_with_typename_detection(
                    &mut map,
                    &schema[id].possible_types_ordered_by_typename_ids,
                    &mut response_fields,
                )?);
            }
        }

        self.post_process(&mut response_fields);

        if !self.typename_response_keys.is_empty() {
            let Some(object_id) = maybe_object_id else {
                return Err(serde::de::Error::custom("Could not determine the object type"));
            };
            let name_id = schema[object_id].name_id;
            for key in self.typename_response_keys {
                response_fields.push(ResponseObjectField {
                    key: *key,
                    required_field_id: None,
                    value: name_id.into(),
                });
            }
        }

        Ok((maybe_object_id, response_fields))
    }
}

impl ConcreteShapeFieldsSeed<'_, '_> {
    fn post_process(&self, response_fields: &mut Vec<ResponseObjectField>) {
        if self.has_error {
            let mut must_propagate_null = false;
            for field_shape in self.field_shape_ids.walk(self.ctx) {
                for error in field_shape.errors() {
                    self.ctx.writer.push_error(
                        error
                            .clone()
                            .with_path((self.ctx.path(), field_shape.key))
                            .with_location(field_shape.as_ref().id.walk(self.ctx).location),
                    );

                    if field_shape.wrapping.is_required() {
                        must_propagate_null = true;
                    } else {
                        response_fields.push(ResponseObjectField {
                            key: field_shape.key,
                            required_field_id: field_shape.required_field_id,
                            value: ResponseValue::Null,
                        });
                    }
                }
            }
            if must_propagate_null {
                self.ctx.propagate_null();
                return;
            }
        }

        if response_fields.len() < self.field_shape_ids.len() {
            let n = response_fields.len();
            for field_shape in self.field_shape_ids.walk(self.ctx) {
                if field_shape.is_skipped() {
                    continue;
                }
                if response_fields[0..n]
                    .binary_search_by(|field| field.key.cmp(&field.key))
                    .is_err()
                {
                    if field_shape.wrapping.is_required() {
                        // If part of the query fields the user requested. We don't propagate null
                        // for extra fields.
                        if field_shape.key.query_position.is_some() {
                            self.ctx.propagate_null();
                            let keys = &self.ctx.operation.cached.solved.response_keys;
                            let message = if field_shape.key.response_key == field_shape.expected_key {
                                format!(
                                    "Error decoding response from upstream: Missing required field named '{}'",
                                    &keys[field_shape.expected_key]
                                )
                            } else {
                                format!(
                                    "Error decoding response from upstream: Missing required field named '{}' (expected: '{}')",
                                    &keys[field_shape.key.response_key],
                                    &keys[field_shape.expected_key]
                                )
                            };
                            self.ctx.writer.push_error(
                                GraphqlError::new(message, ErrorCode::SubgraphInvalidResponseError)
                                    .with_path((self.ctx.path(), field_shape.key))
                                    .with_location(field_shape.as_ref().id.walk(self.ctx).location),
                            );

                            return;
                        }
                    } else {
                        response_fields.push(ResponseObjectField {
                            key: field_shape.key,
                            required_field_id: field_shape.required_field_id,
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
    ) -> Result<ObjectDefinitionId, A::Error> {
        let schema = self.ctx.schema;
        let keys = &self.ctx.operation.cached.solved.response_keys;
        let fields = &self.ctx.operation.cached.solved.shapes[self.field_shape_ids];
        let mut maybe_object_id = None;
        while let Some(key) = map.next_key::<Key<'_>>()? {
            let key = key.as_ref();
            let start = fields.partition_point(|field| &keys[field.expected_key] < key);
            let fields = &fields[start..];

            if fields
                .first()
                .map(|field| &keys[field.expected_key] == key)
                .unwrap_or_default()
            {
                self.visit_field(map, fields, response_fields)?;
            // This supposes that the discriminant is never part of the schema.
            } else if maybe_object_id.is_none() && key == "__typename" {
                let value = map.next_value::<Key<'_>>()?;
                let typename = value.as_ref();
                maybe_object_id = possible_types_ordered_by_typename
                    .binary_search_by(|probe| schema[schema[*probe].name_id].as_str().cmp(typename))
                    .ok();
            } else {
                // Skipping the value.
                map.next_value::<IgnoredAny>()?;
            }
        }

        if let Some(i) = maybe_object_id {
            Ok(possible_types_ordered_by_typename[i])
        } else {
            Err(serde::de::Error::custom(
                "Missing __typename field, could not determine object type.",
            ))
        }
    }

    fn visit_fields<'de, A: MapAccess<'de>>(
        &self,
        map: &mut A,
        response_fields: &mut Vec<ResponseObjectField>,
    ) -> Result<(), A::Error> {
        let keys = &self.ctx.operation.cached.solved.response_keys;
        let fields = &self.ctx.operation.cached.solved.shapes[self.field_shape_ids];
        while let Some(key) = map.next_key::<Key<'_>>()? {
            let key = key.as_ref();
            let start = fields.partition_point(|field| &keys[field.expected_key] < key);
            let fields = &fields[start..];

            if fields
                .first()
                .map(|field| &keys[field.expected_key] == key)
                .unwrap_or_default()
            {
                self.visit_field(map, fields, response_fields)?;
            } else {
                // Skipping the value.
                map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(())
    }

    fn visit_field<'de, A: MapAccess<'de>>(
        &self,
        map: &mut A,
        field_shapes: &[FieldShapeRecord],
        response_fields: &mut Vec<ResponseObjectField>,
    ) -> Result<(), A::Error> {
        let mut end = 1;
        let start_key = field_shapes[0].expected_key;
        // All fields with the same expected_key (when aliases aren't supported by upsteam)
        while field_shapes
            .get(end + 1)
            .map(|field| field.expected_key == start_key)
            .unwrap_or_default()
        {
            end += 1;
        }
        if end == 1 {
            let field = &field_shapes[0];
            self.ctx.path().push(ResponseValueId::Field {
                object_id: self.object_id,
                key: field.key.response_key,
                nullable: field.wrapping.is_nullable(),
            });
            let result = map.next_value_seed(FieldSeed {
                ctx: self.ctx,
                field,
                wrapping: field.wrapping,
            });
            self.ctx.path().pop();
            response_fields.push(ResponseObjectField {
                key: field.key,
                required_field_id: field.required_field_id,
                value: result?,
            });
        } else {
            // if we found more than one field with the same expected_key we need to store the
            // value first.
            let stored_value = map.next_value::<serde_value::Value>()?;
            for field in &field_shapes[..end] {
                self.ctx.path().push(ResponseValueId::Field {
                    object_id: self.object_id,
                    key: field.key.response_key,
                    nullable: field.wrapping.is_nullable(),
                });
                let result = FieldSeed {
                    ctx: self.ctx,
                    field,
                    wrapping: field.wrapping,
                }
                .deserialize(serde_value::ValueDeserializer::new(stored_value.clone()));
                self.ctx.path().pop();
                response_fields.push(ResponseObjectField {
                    key: field.key,
                    required_field_id: field.required_field_id,
                    value: result?,
                });
            }
        }
        Ok(())
    }
}
