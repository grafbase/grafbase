use std::fmt;

use schema::ObjectId;
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};

use crate::response::{
    value::{ResponseObjectFields, RESPONSE_OBJECT_FIELDS_BINARY_SEARCH_THRESHOLD},
    write::deserialize::{field::FieldSeed, key::Key, SeedContext},
    ConcreteObjectShapeId, FieldError, FieldShape, GraphqlError, ObjectIdentifier, ResponseEdge, ResponseObject,
    ResponseObjectRef, ResponseObjectSetId, ResponseValue,
};

pub(crate) struct ConcreteObjectSeed<'ctx, 'seed> {
    ctx: &'seed SeedContext<'ctx>,
    set_id: Option<ResponseObjectSetId>,
    fields_seed: ConcreteObjectFieldsSeed<'ctx, 'seed>,
}

impl<'ctx, 'seed> ConcreteObjectSeed<'ctx, 'seed> {
    pub fn new(ctx: &'seed SeedContext<'ctx>, shape_id: ConcreteObjectShapeId) -> Self {
        let shape = &ctx.shapes[shape_id];
        Self {
            ctx,
            set_id: shape.set_id,
            fields_seed: ConcreteObjectFieldsSeed {
                ctx,
                object_identifier: shape.identifier,
                fields: &ctx.shapes[shape.field_shape_ids],
                field_errors: &ctx.shapes[shape.field_error_ids],
                typename_response_edges: &shape.typename_response_edges,
            },
        }
    }

    pub fn new_with_object_id(
        ctx: &'seed SeedContext<'ctx>,
        shape_id: ConcreteObjectShapeId,
        object_id: ObjectId,
    ) -> Self {
        let shape = &ctx.shapes[shape_id];
        Self {
            ctx,
            set_id: shape.set_id,
            fields_seed: ConcreteObjectFieldsSeed {
                ctx,
                object_identifier: ObjectIdentifier::Known(object_id),
                fields: &ctx.shapes[shape.field_shape_ids],
                field_errors: &ctx.shapes[shape.field_error_ids],
                typename_response_edges: &shape.typename_response_edges,
            },
        }
    }

    pub(crate) fn into_fields_seed(self) -> ConcreteObjectFieldsSeed<'ctx, 'seed> {
        self.fields_seed
    }
}

pub(crate) struct ConcreteObjectFieldsSeed<'ctx, 'seed> {
    ctx: &'seed SeedContext<'ctx>,
    object_identifier: ObjectIdentifier,
    fields: &'ctx [FieldShape],
    field_errors: &'ctx [FieldError],
    typename_response_edges: &'ctx [ResponseEdge],
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for ConcreteObjectSeed<'ctx, 'parent> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx, 'parent> Visitor<'de> for ConcreteObjectSeed<'ctx, 'parent> {
    type Value = ResponseValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an object")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let (object_id, fields) = self.fields_seed.visit_map(map)?;

        let id = self.ctx.writer.push_object(ResponseObject::new(fields));
        if let Some(set_id) = self.set_id {
            self.ctx.writer.push_response_object(
                set_id,
                ResponseObjectRef {
                    id,
                    path: self.ctx.response_path(),
                    definition_id: object_id.expect("Object id should have been identified"),
                },
            );
        }

        Ok(id.into())
    }
}

impl<'de, 'ctx, 'seed> DeserializeSeed<'de> for ConcreteObjectFieldsSeed<'ctx, 'seed> {
    type Value = (Option<ObjectId>, ResponseObjectFields);

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx, 'seed> Visitor<'de> for ConcreteObjectFieldsSeed<'ctx, 'seed> {
    type Value = (Option<ObjectId>, ResponseObjectFields);

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an object")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let plan = self.ctx.plan;
        let mut response_fields =
            ResponseObjectFields::with_capacity(self.fields.len() + self.typename_response_edges.len());

        let mut required_field_error = false;
        for field_error in self.field_errors {
            let mut path = self.ctx.response_path();
            path.push(field_error.edge);

            for error in &field_error.errors {
                self.ctx.writer.push_error(GraphqlError {
                    path: Some(path.clone()),
                    ..error.clone()
                });
            }

            if field_error.is_required {
                required_field_error = true;
            } else {
                response_fields.push((field_error.edge, ResponseValue::Null));
            }
        }
        if required_field_error {
            // Skipping the rest of the fields
            while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
            return self.ctx.propagate_error();
        }

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
                    &plan.schema()[id].possible_types_ordered_by_typename,
                    &mut response_fields,
                )?);
            }
            ObjectIdentifier::InterfaceTypename(id) => {
                maybe_object_id = Some(self.visit_fields_with_typename_detection(
                    &mut map,
                    &plan.schema()[id].possible_types_ordered_by_typename,
                    &mut response_fields,
                )?);
            }
        }

        self.check_missing_fields::<A>(&mut response_fields)?;

        if !self.typename_response_edges.is_empty() {
            let Some(object_id) = maybe_object_id else {
                return Err(serde::de::Error::custom("Could not determine the "));
            };
            let name_id = plan.schema()[object_id].name;
            for edge in self.typename_response_edges {
                response_fields.push((
                    *edge,
                    ResponseValue::StringId {
                        id: name_id,
                        nullable: false,
                    },
                ));
            }
        }

        Ok((maybe_object_id, response_fields))
    }
}

impl<'de, 'ctx, 'seed> ConcreteObjectFieldsSeed<'ctx, 'seed> {
    fn check_missing_fields<A: MapAccess<'de>>(
        &self,
        response_fields: &mut ResponseObjectFields,
    ) -> Result<(), A::Error> {
        if response_fields.len() < self.fields.len() {
            let n = response_fields.len();
            if n <= RESPONSE_OBJECT_FIELDS_BINARY_SEARCH_THRESHOLD {
                for field in self.fields {
                    if !response_fields[0..n].iter().any(|(e, _)| *e == field.edge) {
                        if field.wrapping.is_required() {
                            return Err(serde::de::Error::custom(self.ctx.missing_field_error_message(field)));
                        }
                        response_fields.push((field.edge, ResponseValue::Null));
                    }
                }
            } else {
                for field in self.fields {
                    if response_fields[0..n]
                        .binary_search_by(|(edge, _)| edge.cmp(&field.edge))
                        .is_err()
                    {
                        if field.wrapping.is_required() {
                            return Err(serde::de::Error::custom(self.ctx.missing_field_error_message(field)));
                        }
                        response_fields.push((field.edge, ResponseValue::Null));
                    }
                }
            }
        }

        Ok(())
    }

    fn visit_fields_with_typename_detection<A: MapAccess<'de>>(
        &self,
        map: &mut A,
        possible_types_ordered_by_typename: &[ObjectId],
        response_fields: &mut ResponseObjectFields,
    ) -> Result<ObjectId, A::Error> {
        let schema = self.ctx.plan.schema();
        let keys = self.ctx.plan.response_keys();
        let mut maybe_object_id = None;
        while let Some(key) = map.next_key::<Key<'_>>()? {
            let key = key.as_ref();
            let start = self.fields.partition_point(|field| &keys[field.expected_key] < key);

            if start < self.fields.len() && &keys[self.fields[start].expected_key] == key {
                self.visit_field(map, start, response_fields)?;
            // This supposes that the discriminant is never part of the schema.
            } else if maybe_object_id.is_none() && key == "__typename" {
                let value = map.next_value::<Key<'_>>()?;
                let typename = value.as_ref();
                maybe_object_id = possible_types_ordered_by_typename
                    .binary_search_by(|probe| schema[schema[*probe].name].as_str().cmp(typename))
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

    fn visit_fields<A: MapAccess<'de>>(
        &self,
        map: &mut A,
        response_fields: &mut ResponseObjectFields,
    ) -> Result<(), A::Error> {
        let keys = self.ctx.plan.response_keys();
        while let Some(key) = map.next_key::<Key<'_>>()? {
            let key = key.as_ref();
            let start = self.fields.partition_point(|field| &keys[field.expected_key] < key);

            if start < self.fields.len() && &keys[self.fields[start].expected_key] == key {
                self.visit_field(map, start, response_fields)?;
            } else {
                // Skipping the value.
                map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(())
    }

    fn visit_field<A: MapAccess<'de>>(
        &self,
        map: &mut A,
        start: usize,
        response_fields: &mut ResponseObjectFields,
    ) -> Result<(), A::Error> {
        let mut end = start + 1;
        let start_key = self.fields[start].expected_key;
        // All fields with the same expected_key (when aliases aren't supported by upsteam)
        while self
            .fields
            .get(end + 1)
            .map(|field| field.expected_key == start_key)
            .unwrap_or_default()
        {
            end += 1;
        }
        if end - start == 1 {
            let field = &self.fields[start];
            self.ctx.push_edge(field.edge);
            let result = map.next_value_seed(FieldSeed {
                ctx: self.ctx,
                field,
                wrapping: field.wrapping,
            });
            self.ctx.pop_edge();
            response_fields.push((field.edge, result?));
        } else {
            // if we found more than one field with the same expected_key we need to store the
            // value first.
            let stored_value = map.next_value::<serde_value::Value>()?;
            for field in &self.fields[start..end] {
                self.ctx.push_edge(field.edge);
                let result = FieldSeed {
                    ctx: self.ctx,
                    field,
                    wrapping: field.wrapping,
                }
                .deserialize(serde_value::ValueDeserializer::new(stored_value.clone()));
                self.ctx.pop_edge();
                response_fields.push((field.edge, result?));
            }
        }
        Ok(())
    }
}
