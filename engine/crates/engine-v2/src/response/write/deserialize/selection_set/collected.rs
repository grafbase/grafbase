use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt,
};

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};

use crate::{
    plan::ConcreteSelectionSetId,
    response::{
        write::deserialize::{key::Key, FieldSeed, SeedContextInner},
        ResponseBoundaryItem, ResponseEdge, ResponseObject, ResponsePath, ResponseValue,
    },
};

pub(crate) struct ConcreteCollectionSetSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContextInner<'ctx>,
    pub path: &'parent ResponsePath,
    pub id: ConcreteSelectionSetId,
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for ConcreteCollectionSetSeed<'ctx, 'parent> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx, 'parent> Visitor<'de> for ConcreteCollectionSetSeed<'ctx, 'parent> {
    type Value = ResponseValue;

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
        let selection_set = &plan[self.id];
        let mut identifier = super::ObjectIdentifier::new(self.ctx, selection_set.ty);
        let mut response_fields = BTreeMap::<ResponseEdge, ResponseValue>::new();
        let fields = &plan[selection_set.fields];

        while let Some(key) = map.next_key::<Key<'_>>()? {
            let key = key.as_ref();
            let start = fields.partition_point(|field| &keys[field.expected_key] < key);

            if start < fields.len() && &keys[fields[start].expected_key] == key {
                let mut end = start + 1;
                // All fields with the same expected_key (when aliases aren't support by upsteam)
                while fields
                    .get(end + 1)
                    .map(|field| &keys[field.expected_key] == key)
                    .unwrap_or_default()
                {
                    end += 1;
                }
                if end - start == 1 {
                    let field = &fields[start];
                    let value = map.next_value_seed(FieldSeed {
                        ctx: self.ctx,
                        path: self.path.child(field.edge),
                        bound_field_id: field.bound_field_id,
                        ty: &field.ty,
                        wrapping: field.wrapping.clone(),
                    })?;
                    response_fields.insert(field.edge, value);
                } else {
                    // if we found more than one field with the same expected_key we need to store the
                    // value first.
                    let stored_value = map.next_value::<serde_value::Value>()?;
                    for field in &fields[start..end] {
                        let value = FieldSeed {
                            ctx: self.ctx,
                            path: self.path.child(field.edge),
                            bound_field_id: field.bound_field_id,
                            ty: &field.ty,
                            wrapping: field.wrapping.clone(),
                        }
                        .deserialize(serde_value::ValueDeserializer::new(stored_value.clone()))?;
                        response_fields.insert(field.edge, value);
                    }
                }
            // This supposes that the discriminant is never part of the schema.
            } else if identifier.discriminant_key_matches(key) {
                identifier.determine_object_id_from_discriminant(map.next_value()?);
            } else {
                // Skipping the value.
                map.next_value::<IgnoredAny>()?;
            }
        }

        // Ensuring we did find the object_id if the root was an interface.
        let object_id = identifier.try_into_object_id()?;

        // Checking if we're missing fields
        if response_fields.len() < fields.len() {
            for field in fields {
                if let Entry::Vacant(entry) = response_fields.entry(field.edge) {
                    if field.wrapping.is_required() {
                        return Err(serde::de::Error::custom(self.ctx.missing_field_error_message(field)));
                    }
                    entry.insert(ResponseValue::Null);
                }
            }
        }

        let name_id = plan.schema()[object_id].name;
        for edge in &selection_set.typename_fields {
            response_fields.insert(
                *edge,
                ResponseValue::StringId {
                    id: name_id,
                    nullable: false,
                },
            );
        }

        let mut data = self.ctx.response_part.borrow_mut();
        let id = data.push_object(ResponseObject {
            object_id,
            fields: response_fields,
        });
        if let Some(boundary_id) = selection_set.maybe_boundary_id {
            data[boundary_id].push(ResponseBoundaryItem {
                response_object_id: id,
                response_path: self.path.clone(),
                object_id,
            });
        }

        Ok(ResponseValue::Object { id, nullable: false })
    }
}
