use std::fmt;

use schema::ObjectId;
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};

use crate::{
    plan::{CollectedField, CollectedSelectionSetId, PlanBoundaryId, RuntimeCollectedSelectionSet},
    request::SelectionSetType,
    response::{
        value::ResponseObjectFields,
        write::deserialize::{key::Key, FieldSeed, SeedContextInner},
        ResponseBoundaryItem, ResponseEdge, ResponseObject, ResponseValue,
    },
};

/// Seed for a collected selection set, meaning we know exactly which fields should be present
/// or not. There is no field with type conditions anymore.
pub(crate) struct CollectedSelectionSetSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContextInner<'ctx>,
    pub boundary_ids: &'parent [PlanBoundaryId],
    pub fields_seed: CollectedFieldsSeed<'ctx, 'parent>,
}

pub(crate) struct CollectedFieldsSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContextInner<'ctx>,
    pub selection_set_ty: SelectionSetType,
    pub fields: &'parent [CollectedField],
    pub typename_fields: &'parent [ResponseEdge],
}

impl<'ctx, 'parent> CollectedSelectionSetSeed<'ctx, 'parent> {
    pub fn new_from_id(ctx: &'parent SeedContextInner<'ctx>, id: CollectedSelectionSetId) -> Self {
        let selection_set = &ctx.plan[id];
        Self {
            ctx,
            boundary_ids: if let Some(ref id) = selection_set.maybe_boundary_id {
                std::array::from_ref(id)
            } else {
                &[]
            },
            fields_seed: CollectedFieldsSeed {
                ctx,
                selection_set_ty: selection_set.ty,

                fields: &ctx.plan[selection_set.fields],
                typename_fields: &selection_set.typename_fields,
            },
        }
    }

    pub fn new(ctx: &'parent SeedContextInner<'ctx>, selection_set: &'parent RuntimeCollectedSelectionSet) -> Self {
        Self {
            ctx,
            boundary_ids: &selection_set.boundary_ids,
            fields_seed: CollectedFieldsSeed {
                ctx,
                selection_set_ty: SelectionSetType::Object(selection_set.object_id),
                fields: &selection_set.fields,
                typename_fields: &selection_set.typename_fields,
            },
        }
    }
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for CollectedSelectionSetSeed<'ctx, 'parent> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx, 'parent> Visitor<'de> for CollectedSelectionSetSeed<'ctx, 'parent> {
    type Value = ResponseValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an object")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let (maybe_object_id, fields) = self.fields_seed.visit_map(map)?;
        let mut data = self.ctx.response_part.borrow_mut();

        let id = data.push_object(ResponseObject::new(fields));
        if !self.boundary_ids.is_empty() {
            let Some(object_id) = maybe_object_id else {
                return Err(serde::de::Error::custom("Could not determine the __typename"));
            };
            for boundary_id in self.boundary_ids {
                data[*boundary_id].push(ResponseBoundaryItem {
                    response_object_id: id,
                    response_path: self.ctx.response_path(),
                    object_id,
                });
            }
        }

        Ok(id.into())
    }
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for CollectedFieldsSeed<'ctx, 'parent> {
    type Value = (Option<ObjectId>, ResponseObjectFields);

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx, 'parent> Visitor<'de> for CollectedFieldsSeed<'ctx, 'parent> {
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
        let keys = plan.response_keys();
        let mut response_fields = ResponseObjectFields::with_capacity(self.fields.len() + self.typename_fields.len());
        let mut maybe_object_id = None;
        if let SelectionSetType::Object(object_id) = self.selection_set_ty {
            maybe_object_id = Some(object_id);
            while let Some(key) = map.next_key::<Key<'_>>()? {
                let key = key.as_ref();
                let start = self.fields.partition_point(|field| &keys[field.expected_key] < key);

                if start < self.fields.len() && &keys[self.fields[start].expected_key] == key {
                    self.visit_field(&mut map, start, &mut response_fields)?;
                } else {
                    // Skipping the value.
                    map.next_value::<IgnoredAny>()?;
                }
            }
        } else {
            let mut identifier = super::ObjectIdentifier::new(self.ctx, self.selection_set_ty);
            while let Some(key) = map.next_key::<Key<'_>>()? {
                let key = key.as_ref();
                let start = self.fields.partition_point(|field| &keys[field.expected_key] < key);

                if start < self.fields.len() && &keys[self.fields[start].expected_key] == key {
                    self.visit_field(&mut map, start, &mut response_fields)?;
                // This supposes that the discriminant is never part of the schema.
                } else if maybe_object_id.is_none() && identifier.discriminant_key_matches(key) {
                    maybe_object_id = identifier.determine_object_id_from_discriminant(map.next_value()?)
                } else {
                    // Skipping the value.
                    map.next_value::<IgnoredAny>()?;
                }
            }
        };

        // Checking if we're missing fields
        if response_fields.len() < self.fields.len() {
            for field in self.fields {
                if !response_fields.iter().any(|(e, _)| *e == field.edge) {
                    if field.wrapping.is_required() {
                        return Err(serde::de::Error::custom(self.ctx.missing_field_error_message(field)));
                    }
                    response_fields.push((field.edge, ResponseValue::Null));
                }
            }
        }

        if !self.typename_fields.is_empty() {
            let Some(object_id) = maybe_object_id else {
                return Err(serde::de::Error::custom("Could not determine the "));
            };
            let name_id = plan.schema()[object_id].name;
            for edge in self.typename_fields {
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

impl<'ctx, 'parent> CollectedFieldsSeed<'ctx, 'parent> {
    fn visit_field<'de, A: MapAccess<'de>>(
        &self,
        map: &mut A,
        start: usize,
        response_fields: &mut ResponseObjectFields,
    ) -> Result<(), A::Error> {
        let mut end = start + 1;
        // All fields with the same expected_key (when aliases aren't supported by upsteam)
        while self
            .fields
            .get(end + 1)
            .map(|field| field.expected_key == self.fields[start].expected_key)
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
