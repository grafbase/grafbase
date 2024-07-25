use std::fmt;

use id_newtypes::IdRange;
use schema::ObjectId;
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};

use crate::response::{
    value::ResponseObjectField,
    write::deserialize::{field::FieldSeed, key::Key, SeedContext},
    ConcreteObjectShapeId, FieldShape, FieldShapeId, GraphqlError, ObjectIdentifier, ResponseEdge, ResponseObject,
    ResponseObjectRef, ResponseObjectSetId, ResponseValue,
};

pub(crate) struct ConcreteObjectSeed<'ctx, 'seed> {
    ctx: &'seed SeedContext<'ctx>,
    set_id: Option<ResponseObjectSetId>,
    fields_seed: ConcreteObjectFieldsSeed<'ctx, 'seed>,
}

impl<'ctx, 'seed> ConcreteObjectSeed<'ctx, 'seed> {
    pub fn new(ctx: &'seed SeedContext<'ctx>, shape_id: ConcreteObjectShapeId) -> Self {
        let shape = &ctx.operation.response_blueprint[shape_id];
        Self {
            ctx,
            set_id: shape.set_id,
            fields_seed: ConcreteObjectFieldsSeed {
                ctx,
                has_error: ctx.operation.query_modifications.concrete_shape_has_error[shape_id],
                object_identifier: shape.identifier,
                field_shape_ids: shape.field_shape_ids,
                typename_response_edges: &shape.typename_response_edges,
            },
        }
    }

    pub fn new_with_object_id(
        ctx: &'seed SeedContext<'ctx>,
        shape_id: ConcreteObjectShapeId,
        object_id: ObjectId,
    ) -> Self {
        let shape = &ctx.operation.response_blueprint[shape_id];
        Self {
            ctx,
            set_id: shape.set_id,
            fields_seed: ConcreteObjectFieldsSeed {
                ctx,
                has_error: ctx.operation.query_modifications.concrete_shape_has_error[shape_id],
                object_identifier: ObjectIdentifier::Known(object_id),
                field_shape_ids: shape.field_shape_ids,
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
    has_error: bool,
    object_identifier: ObjectIdentifier,
    field_shape_ids: IdRange<FieldShapeId>,
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
    type Value = (Option<ObjectId>, Vec<ResponseObjectField>);

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx, 'seed> Visitor<'de> for ConcreteObjectFieldsSeed<'ctx, 'seed> {
    type Value = (Option<ObjectId>, Vec<ResponseObjectField>);

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an object")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let plan = self.ctx.plan;
        let mut response_fields = Vec::with_capacity(self.field_shape_ids.len() + self.typename_response_edges.len());

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

        self.post_process::<A>(&mut response_fields)?;

        if !self.typename_response_edges.is_empty() {
            let Some(object_id) = maybe_object_id else {
                return Err(serde::de::Error::custom("Could not determine the "));
            };
            let name_id = plan.schema()[object_id].name;
            for edge in self.typename_response_edges {
                response_fields.push(ResponseObjectField {
                    edge: *edge,
                    required_field_id: None,
                    value: ResponseValue::StringId {
                        id: name_id,
                        nullable: false,
                    },
                });
            }
        }

        Ok((maybe_object_id, response_fields))
    }
}

impl<'de, 'ctx, 'seed> ConcreteObjectFieldsSeed<'ctx, 'seed> {
    fn post_process<A: MapAccess<'de>>(&self, response_fields: &mut Vec<ResponseObjectField>) -> Result<(), A::Error> {
        if self.has_error {
            let mut required_field_error = false;
            for id in self.field_shape_ids {
                for error_id in self
                    .ctx
                    .operation
                    .query_modifications
                    .field_shape_id_to_error_ids
                    .find_all(id)
                    .copied()
                {
                    let field_shape = &self.ctx.operation.response_blueprint[id];
                    let mut path = self.ctx.response_path();
                    path.push(field_shape.edge);

                    self.ctx.writer.push_error(GraphqlError {
                        path: Some(path),
                        ..self.ctx.operation[error_id].clone()
                    });

                    if field_shape.wrapping.is_required() {
                        required_field_error = true;
                    } else {
                        response_fields.push(ResponseObjectField {
                            edge: field_shape.edge,
                            required_field_id: field_shape.required_field_id,
                            value: ResponseValue::Null,
                        });
                    }
                }
            }
            if required_field_error {
                return self.ctx.propagate_error();
            }
        }

        if response_fields.len() < self.field_shape_ids.len() {
            let field_shapes = &self.ctx.operation.response_blueprint[self.field_shape_ids];
            let n = response_fields.len();
            for field_shape in field_shapes {
                if response_fields[0..n]
                    .binary_search_by(|field| field.edge.cmp(&field.edge))
                    .is_err()
                {
                    if field_shape.wrapping.is_required() {
                        return Err(serde::de::Error::custom(
                            self.ctx.missing_field_error_message(field_shape),
                        ));
                    }
                    response_fields.push(ResponseObjectField {
                        edge: field_shape.edge,
                        required_field_id: field_shape.required_field_id,
                        value: ResponseValue::Null,
                    });
                }
            }
        }

        Ok(())
    }

    fn visit_fields_with_typename_detection<A: MapAccess<'de>>(
        &self,
        map: &mut A,
        possible_types_ordered_by_typename: &[ObjectId],
        response_fields: &mut Vec<ResponseObjectField>,
    ) -> Result<ObjectId, A::Error> {
        let schema = self.ctx.plan.schema();
        let keys = self.ctx.plan.response_keys();
        let fields = &self.ctx.operation.response_blueprint[self.field_shape_ids];
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
        response_fields: &mut Vec<ResponseObjectField>,
    ) -> Result<(), A::Error> {
        let keys = self.ctx.plan.response_keys();
        let fields = &self.ctx.operation.response_blueprint[self.field_shape_ids];
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

    fn visit_field<A: MapAccess<'de>>(
        &self,
        map: &mut A,
        field_shapes: &[FieldShape],
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
            self.ctx.push_edge(field.edge);
            let result = map.next_value_seed(FieldSeed {
                ctx: self.ctx,
                field,
                wrapping: field.wrapping,
            });
            self.ctx.pop_edge();
            response_fields.push(ResponseObjectField {
                edge: field.edge,
                required_field_id: field.required_field_id,
                value: result?,
            });
        } else {
            // if we found more than one field with the same expected_key we need to store the
            // value first.
            let stored_value = map.next_value::<serde_value::Value>()?;
            for field in &field_shapes[..end] {
                self.ctx.push_edge(field.edge);
                let result = FieldSeed {
                    ctx: self.ctx,
                    field,
                    wrapping: field.wrapping,
                }
                .deserialize(serde_value::ValueDeserializer::new(stored_value.clone()));
                self.ctx.pop_edge();
                response_fields.push(ResponseObjectField {
                    edge: field.edge,
                    required_field_id: field.required_field_id,
                    value: result?,
                });
            }
        }
        Ok(())
    }
}
