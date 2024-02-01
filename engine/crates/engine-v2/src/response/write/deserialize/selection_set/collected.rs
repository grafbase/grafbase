use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt,
};

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};

use crate::{
    plan::CollectedSelectionSet,
    response::{
        write::deserialize::{key::Key, FieldSeed, SeedContextInner},
        ResponseEdge, ResponseObject, ResponsePath, ResponseValue,
    },
};

pub(crate) struct CollectedFieldsSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContextInner<'ctx>,
    pub path: &'parent ResponsePath,
    pub expected: &'parent CollectedSelectionSet,
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for CollectedFieldsSeed<'ctx, 'parent> {
    type Value = ResponseObject;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx, 'parent> Visitor<'de> for CollectedFieldsSeed<'ctx, 'parent> {
    type Value = ResponseObject;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an object")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut identifier = super::ObjectIdentifier::new(self.ctx, self.expected.ty);
        let mut fields = BTreeMap::<ResponseEdge, ResponseValue>::new();
        while let Some(key) = map.next_key::<Key<'_>>()? {
            let key = key.as_ref();
            let start = self
                .expected
                .fields
                .partition_point(|field| field.expected_key.as_str() < key);

            if start < self.expected.fields.len() && self.expected.fields[start].expected_key == key {
                let mut end = start + 1;
                // All fields with the same expected_key (when aliases aren't support by upsteam)
                while self
                    .expected
                    .fields
                    .get(end + 1)
                    .map(|field| field.expected_key == key)
                    .unwrap_or_default()
                {
                    end += 1;
                }
                if end - start == 1 {
                    let field = &self.expected.fields[start];
                    let value = map.next_value_seed(FieldSeed {
                        ctx: self.ctx,
                        path: self.path.child(field.edge),
                        bound_field_id: field.bound_field_id,
                        expected_type: &field.ty,
                        wrapping: field.wrapping.clone(),
                    })?;
                    fields.insert(field.edge, value);
                } else {
                    // if we found more than one field with the same expected_key we need to store the
                    // value first.
                    let stored_value = map.next_value::<serde_value::Value>()?;
                    for field in &self.expected.fields[start..end] {
                        let value = FieldSeed {
                            ctx: self.ctx,
                            path: self.path.child(field.edge),
                            bound_field_id: field.bound_field_id,
                            expected_type: &field.ty,
                            wrapping: field.wrapping.clone(),
                        }
                        .deserialize(serde_value::ValueDeserializer::new(stored_value.clone()))?;
                        fields.insert(field.edge, value);
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
        if fields.len() < self.expected.fields.len() {
            for field in &self.expected.fields {
                if let Entry::Vacant(entry) = fields.entry(field.edge) {
                    if field.wrapping.is_required() {
                        return Err(serde::de::Error::custom(self.ctx.missing_field_error_message(field)));
                    }
                    entry.insert(ResponseValue::Null);
                }
            }
        }

        for edge in &self.expected.typename_fields {
            fields.insert(
                *edge,
                ResponseValue::StringId {
                    id: self.ctx.walker.schema()[object_id].name,
                    nullable: false,
                },
            );
        }

        Ok(ResponseObject { object_id, fields })
    }
}
