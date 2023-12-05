use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt,
};

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};

use crate::{
    plan::ExpectedGroupedFields,
    response::{
        write::deserialize::{FieldSeed, SeedContext},
        BoundResponseKey, ResponseObject, ResponsePath, ResponseValue,
    },
};

pub struct ObjectFieldsSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContext<'ctx>,
    pub path: &'parent ResponsePath,
    pub expected: &'parent ExpectedGroupedFields,
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for ObjectFieldsSeed<'ctx, 'parent> {
    type Value = ResponseObject;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx, 'parent> Visitor<'de> for ObjectFieldsSeed<'ctx, 'parent> {
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
        let mut fields = BTreeMap::<BoundResponseKey, ResponseValue>::new();
        while let Some(key) = map.next_key::<&str>()? {
            let start = self
                .expected
                .fields
                .partition_point(|field| field.expected_name.as_str() < key);
            // likely we found a matching field
            if start < self.expected.fields.len() {
                let mut end = start + 1;
                // All fields with the same expected_name (when aliases aren't support by upsteam)
                while self
                    .expected
                    .fields
                    .get(end + 1)
                    .map(|field| field.expected_name == key)
                    .unwrap_or_default()
                {
                    end += 1;
                }
                if end - start == 1 {
                    let field = &self.expected.fields[start];
                    let value = map.next_value_seed(FieldSeed {
                        ctx: self.ctx,
                        path: self.path.child(field.bound_response_key),
                        definition_id: field.definition_id,
                        expected_type: &field.ty,
                        wrapping: field.wrapping.clone(),
                    })?;
                    fields.insert(field.bound_response_key, value);
                } else {
                    // if we found more than one field with the same expected_name we need to store the
                    // value first.
                    let stored_value = map.next_value::<serde_value::Value>()?;
                    for field in &self.expected.fields[start..end] {
                        let value = FieldSeed {
                            ctx: self.ctx,
                            path: self.path.child(field.bound_response_key),
                            definition_id: field.definition_id,
                            expected_type: &field.ty,
                            wrapping: field.wrapping.clone(),
                        }
                        .deserialize(serde_value::ValueDeserializer::new(stored_value.clone()))?;
                        fields.insert(field.bound_response_key, value);
                    }
                }
            // This supposes that the discriminant is never part of the schema
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
                if let Entry::Vacant(entry) = fields.entry(field.bound_response_key) {
                    if field.wrapping.is_required() {
                        let missing_key = &self.ctx.walker.operation().response_keys[field.bound_response_key];
                        if field.expected_name == missing_key {
                            return Err(serde::de::Error::custom(format!(
                                "Missing required field named '{missing_key}'"
                            )));
                        }
                        return Err(serde::de::Error::custom(format!(
                            "Missing required field named '{missing_key}' (expected: '{}')",
                            field.expected_name
                        )));
                    }
                    entry.insert(ResponseValue::Null);
                }
            }
        }
        for bound_response_key in &self.expected.typename_fields {
            fields.insert(
                *bound_response_key,
                ResponseValue::StringId {
                    id: self.ctx.walker.schema()[object_id].name,
                    nullable: false,
                },
            );
        }
        Ok(ResponseObject { object_id, fields })
    }
}
